use std::sync::Arc;

use async_trait::async_trait;
use http::{header::CONTENT_TYPE, Method};
use opentelemetry::logs::{LogError, LogResult};
use opentelemetry_sdk::export::logs::{LogData, LogExporter};

use super::OtlpHttpClient;

#[async_trait]
impl LogExporter for OtlpHttpClient {
    async fn export(&mut self, batch: Vec<LogData>) -> LogResult<()> {
        let client = self
            .client
            .lock()
            .map_err(|e| LogError::Other(e.to_string().into()))
            .and_then(|g| match &*g {
                Some(client) => Ok(Arc::clone(client)),
                _ => Err(LogError::Other("exporter is already shut down".into())),
            })?;

        let (body, content_type) = build_body(batch)?;
        let mut request = http::Request::builder()
            .method(Method::POST)
            .uri(&self.collector_endpoint)
            .header(CONTENT_TYPE, content_type)
            .body(body)
            .map_err(|e| crate::Error::RequestFailed(Box::new(e)))?;

        for (k, v) in &self.headers {
            request.headers_mut().insert(k.clone(), v.clone());
        }

        let request_uri = request.uri().to_string();
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
    }

    fn shutdown(&mut self) {
        let _ = self.client.lock().map(|mut c| c.take());
    }
}

#[cfg(feature = "http-proto")]
fn build_body(logs: Vec<LogData>) -> LogResult<(Vec<u8>, &'static str)> {
    use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
    use prost::Message;

    let req = ExportLogsServiceRequest {
        resource_logs: logs.into_iter().map(Into::into).collect(),
    };
    let mut buf = vec![];
    req.encode(&mut buf).map_err(crate::Error::from)?;

    Ok((buf, "application/x-protobuf"))
}

#[cfg(not(feature = "http-proto"))]
fn build_body(logs: Vec<LogData>) -> LogResult<(Vec<u8>, &'static str)> {
    Err(LogsError::Other(
        "No http protocol configured. Enable one via `http-proto`".into(),
    ))
}
