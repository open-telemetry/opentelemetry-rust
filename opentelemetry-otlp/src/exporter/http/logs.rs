use super::OtlpHttpClient;
use opentelemetry_sdk::error::{OTelSdkError, OTelSdkResult};
use opentelemetry_sdk::logs::{LogBatch, LogExporter};
use std::time;

impl LogExporter for OtlpHttpClient {
    async fn export(&self, batch: LogBatch<'_>) -> OTelSdkResult {
        self.export_http_with_retry(
            batch,
            OtlpHttpClient::build_logs_export_body,
            "HttpLogsClient.Export",
        )
        .await
    }

    fn shutdown_with_timeout(&self, _timeout: time::Duration) -> OTelSdkResult {
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
