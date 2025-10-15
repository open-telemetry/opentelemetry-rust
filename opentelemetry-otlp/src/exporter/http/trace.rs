use super::OtlpHttpClient;
use opentelemetry_sdk::{
    error::{OTelSdkError, OTelSdkResult},
    trace::{SpanData, SpanExporter},
};

impl SpanExporter for OtlpHttpClient {
    async fn export(&self, batch: Vec<SpanData>) -> OTelSdkResult {
        self.export_http_with_retry(
            batch,
            OtlpHttpClient::build_trace_export_body,
            "HttpTracesClient.Export",
        )
        .await
    }

    fn shutdown(&mut self) -> OTelSdkResult {
        let mut client_guard = self.client.lock().map_err(|e| {
            OTelSdkError::InternalFailure(format!("Failed to acquire client lock: {e}"))
        })?;

        if client_guard.take().is_none() {
            return Err(OTelSdkError::AlreadyShutdown);
        }

        Ok(())
    }

    fn set_resource(&mut self, resource: &opentelemetry_sdk::Resource) {
        self.resource = resource.into();
    }
}
