use std::sync::Arc;

use crate::metric::MetricsClient;
use http::{header::CONTENT_TYPE, Method};
use opentelemetry::otel_debug;
use opentelemetry_sdk::error::{OTelSdkError, OTelSdkResult};
use opentelemetry_sdk::metrics::data::ResourceMetrics;

use super::OtlpHttpClient;

impl MetricsClient for OtlpHttpClient {
    async fn export(&self, metrics: &ResourceMetrics) -> OTelSdkResult {
        let client = self
            .client
            .lock()
            .map_err(|e| OTelSdkError::InternalFailure(format!("Failed to acquire lock: {e:?}")))
            .and_then(|g| match &*g {
                Some(client) => Ok(Arc::clone(client)),
                _ => Err(OTelSdkError::AlreadyShutdown),
            })?;

        let (body, content_type, content_encoding) = self.build_metrics_export_body(metrics).ok_or_else(|| {
            OTelSdkError::InternalFailure("Failed to serialize metrics".to_string())
        })?;
        
        let mut request_builder = http::Request::builder()
            .method(Method::POST)
            .uri(&self.collector_endpoint)
            .header(CONTENT_TYPE, content_type);
            
        if let Some(encoding) = content_encoding {
            request_builder = request_builder.header("Content-Encoding", encoding);
        }

        let mut request = request_builder
            .body(body.into())
            .map_err(|e| OTelSdkError::InternalFailure(format!("{e:?}")))?;

        for (k, v) in &self.headers {
            request.headers_mut().insert(k.clone(), v.clone());
        }

        otel_debug!(name: "HttpMetricsClient.ExportStarted");
        let result = client.send_bytes(request).await;

        match result {
            Ok(response) => {
                if response.status().is_success() {
                    otel_debug!(name: "HttpMetricsClient.ExportSucceeded");
                    Ok(())
                } else {
                    let error = format!(
                        "OpenTelemetry metrics export failed. Status Code: {}, Response: {:?}",
                        response.status().as_u16(),
                        response.body()
                    );
                    otel_debug!(name: "HttpMetricsClient.ExportFailed", error = &error);
                    Err(OTelSdkError::InternalFailure(error))
                }
            }
            Err(e) => {
                let error = format!("{e:?}");
                otel_debug!(name: "HttpMetricsClient.ExportFailed", error = &error);
                Err(OTelSdkError::InternalFailure(error))
            }
        }
    }

    fn shutdown(&self) -> OTelSdkResult {
        self.client
            .lock()
            .map_err(|e| OTelSdkError::InternalFailure(format!("Failed to acquire lock: {e}")))?
            .take();

        Ok(())
    }
}
