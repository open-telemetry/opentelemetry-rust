use std::sync::Arc;

use futures_core::future::BoxFuture;
use http::{header::CONTENT_TYPE, Method};
use opentelemetry::logs::{LogError, LogResult};
use opentelemetry::InstrumentationLibrary;
use opentelemetry_sdk::export::logs::LogExporter;
use opentelemetry_sdk::logs::LogRecord;

use super::OtlpHttpClient;

impl LogExporter for OtlpHttpClient {
    fn export(
        &mut self,
        batch: Vec<(&LogRecord, &InstrumentationLibrary)>,
    ) -> BoxFuture<'static, LogResult<()>> {
        let client = match self
            .client
            .lock()
            .map_err(|e| LogError::Other(e.to_string().into()))
            .and_then(|g| match &*g {
                Some(client) => Ok(Arc::clone(client)),
                _ => Err(LogError::Other("exporter is already shut down".into())),
            }) {
            Ok(client) => client,
            Err(err) => return Box::pin(std::future::ready(Err(err))),
        };

        let (body, content_type) = match self.build_logs_export_body(batch) {
            Ok(body) => body,
            Err(err) => return Box::pin(std::future::ready(Err(err))),
        };
        let mut request = match http::Request::builder()
            .method(Method::POST)
            .uri(&self.collector_endpoint)
            .header(CONTENT_TYPE, content_type)
            .body(body)
        {
            Ok(req) => req,
            Err(e) => {
                return Box::pin(std::future::ready(Err(crate::Error::RequestFailed(
                    Box::new(e),
                )
                .into())))
            }
        };

        for (k, v) in &self.headers {
            request.headers_mut().insert(k.clone(), v.clone());
        }

        let request_uri = request.uri().to_string();
        Box::pin(async move {
            let response = client.send(request).await?;

            if !response.status().is_success() {
                let error = format!(
                    "OpenTelemetry logs export failed. Url: {}, Status Code: {}, Response: {:?}",
                    response.status().as_u16(),
                    request_uri,
                    response.body()
                );
                return Err(LogError::Other(error.into()));
            }
            Ok(())
        })
    }

    fn shutdown(&mut self) {
        let _ = self.client.lock().map(|mut c| c.take());
    }

    fn set_resource(&mut self, resource: &opentelemetry_sdk::Resource) {
        self.resource = resource.into();
    }
}
