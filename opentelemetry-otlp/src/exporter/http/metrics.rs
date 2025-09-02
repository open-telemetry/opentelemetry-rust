use crate::metric::MetricsClient;
use opentelemetry_sdk::error::{OTelSdkError, OTelSdkResult};
use opentelemetry_sdk::metrics::data::ResourceMetrics;

use super::OtlpHttpClient;

impl MetricsClient for OtlpHttpClient {
    async fn export(&self, metrics: &ResourceMetrics) -> OTelSdkResult {
        let build_body_wrapper = |client: &OtlpHttpClient, metrics: &ResourceMetrics| {
            client
                .build_metrics_export_body(metrics)
                .ok_or_else(|| "Failed to serialize metrics".to_string())
        };

        self.export_http_with_retry(metrics, build_body_wrapper, "HttpMetricsClient.Export")
            .await
    }

    fn shutdown(&self) -> OTelSdkResult {
        self.client
            .lock()
            .map_err(|e| OTelSdkError::InternalFailure(format!("Failed to acquire lock: {e}")))?
            .take();

        Ok(())
    }
}
