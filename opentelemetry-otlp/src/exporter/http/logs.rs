use std::sync::Arc;

use http::{header::CONTENT_TYPE, Method};
use opentelemetry::otel_debug;
use opentelemetry_sdk::error::{OTelSdkError, OTelSdkResult};
use opentelemetry_sdk::logs::{LogBatch, LogExporter};

use super::OtlpHttpClient;

impl LogExporter for OtlpHttpClient {
    #[allow(clippy::manual_async_fn)]
    fn export(
        &self,
        batch: LogBatch<'_>,
    ) -> impl std::future::Future<Output = OTelSdkResult> + Send {
        async move {
            let client = self
                .client
                .lock()
                .map_err(|e| OTelSdkError::InternalFailure(e.to_string()))
                .and_then(|g| match &*g {
                    Some(client) => Ok(Arc::clone(client)),
                    _ => Err(OTelSdkError::AlreadyShutdown),
                })?;

            let (body, content_type) = { self.build_logs_export_body(batch)? };
            let mut request = http::Request::builder()
                .method(Method::POST)
                .uri(&self.collector_endpoint)
                .header(CONTENT_TYPE, content_type)
                .body(body.into())
                .map_err(|e| OTelSdkError::InternalFailure(format!("Request failure: {}", e)))?;

            for (k, v) in &self.headers {
                request.headers_mut().insert(k.clone(), v.clone());
            }

            let request_uri = request.uri().to_string();
            otel_debug!(name: "HttpLogsClient.CallingExport");
            let response = client
                .send_bytes(request)
                .await
                .map_err(|e| OTelSdkError::InternalFailure(format!("Failed exporting: {}", e)))?;

            if !response.status().is_success() {
                let error = format!(
                    "OpenTelemetry logs export failed. Url: {}, Status Code: {}, Response: {:?}",
                    response.status().as_u16(),
                    request_uri,
                    response.body()
                );
                return Err(OTelSdkError::InternalFailure(error.to_string()));
            }

            Ok(())
        }
    }

    fn shutdown(&mut self) -> OTelSdkResult {
        self.client
            .lock()
            .map_err(|e| OTelSdkError::InternalFailure(format!("Failed to acquire lock: {}", e)))?
            .take();
        Ok(())
    }

    fn set_resource(&mut self, resource: &opentelemetry_sdk::Resource) {
        self.resource = resource.into();
    }
}
