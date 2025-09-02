use std::sync::Arc;

use crate::metric::MetricsClient;
use http::{header::CONTENT_TYPE, Method};
use opentelemetry::otel_debug;
use opentelemetry_sdk::error::{OTelSdkError, OTelSdkResult};
use opentelemetry_sdk::metrics::data::ResourceMetrics;

use super::OtlpHttpClient;

#[cfg(feature = "http-retry")]
use super::{classify_http_export_error, HttpExportError, HttpRetryData};
#[cfg(feature = "http-retry")]
use opentelemetry_sdk::retry::{retry_with_backoff, RetryPolicy};
#[cfg(feature = "http-retry")]
use opentelemetry_sdk::runtime::Tokio;

impl MetricsClient for OtlpHttpClient {
    #[cfg(feature = "http-retry")]
    async fn export(&self, metrics: &ResourceMetrics) -> OTelSdkResult {
        let policy = RetryPolicy {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 1600,
            jitter_ms: 100,
        };

        // Build request body once before retry loop
        let (body, content_type, content_encoding) =
            self.build_metrics_export_body(metrics).ok_or_else(|| {
                OTelSdkError::InternalFailure("Failed to serialize metrics".to_string())
            })?;

        let retry_data = Arc::new(HttpRetryData {
            body,
            headers: self.headers.clone(),
            endpoint: self.collector_endpoint.to_string(),
        });

        retry_with_backoff(
            Tokio,
            policy,
            classify_http_export_error,
            "HttpMetricsClient.Export",
            || async {
                // Get client
                let client = self
                    .client
                    .lock()
                    .map_err(|e| HttpExportError {
                        status_code: 500,
                        retry_after: None,
                        message: format!("Mutex lock failed: {e}"),
                    })?
                    .as_ref()
                    .ok_or_else(|| HttpExportError {
                        status_code: 500,
                        retry_after: None,
                        message: "Exporter already shutdown".to_string(),
                    })?
                    .clone();

                // Build HTTP request
                let mut request_builder = http::Request::builder()
                    .method(Method::POST)
                    .uri(&retry_data.endpoint)
                    .header(CONTENT_TYPE, content_type);

                if let Some(encoding) = content_encoding {
                    request_builder = request_builder.header("Content-Encoding", encoding);
                }

                let mut request = request_builder
                    .body(retry_data.body.clone().into())
                    .map_err(|e| HttpExportError {
                        status_code: 400,
                        retry_after: None,
                        message: format!("Failed to build HTTP request: {e}"),
                    })?;

                for (k, v) in retry_data.headers.iter() {
                    request.headers_mut().insert(k.clone(), v.clone());
                }

                let request_uri = request.uri().to_string();
                otel_debug!(name: "HttpMetricsClient.ExportStarted");

                // Send request
                let response = client.send_bytes(request).await.map_err(|e| {
                    HttpExportError {
                        status_code: 0, // Network error
                        retry_after: None,
                        message: format!("Network error: {e:?}"),
                    }
                })?;

                let status_code = response.status().as_u16();
                let retry_after = response
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string());

                if !response.status().is_success() {
                    return Err(HttpExportError {
                        status_code,
                        retry_after,
                        message: format!(
                            "HTTP export failed. Url: {}, Status: {}, Response: {:?}",
                            request_uri,
                            status_code,
                            response.body()
                        ),
                    });
                }

                otel_debug!(name: "HttpMetricsClient.ExportSucceeded");
                Ok(())
            },
        )
        .await
        .map_err(|e| OTelSdkError::InternalFailure(e.message))
    }

    #[cfg(not(feature = "http-retry"))]
    async fn export(&self, metrics: &ResourceMetrics) -> OTelSdkResult {
        let client = self
            .client
            .lock()
            .map_err(|e| OTelSdkError::InternalFailure(format!("Failed to acquire lock: {e:?}")))
            .and_then(|g| match &*g {
                Some(client) => Ok(Arc::clone(client)),
                _ => Err(OTelSdkError::AlreadyShutdown),
            })?;

        let (body, content_type, content_encoding) =
            self.build_metrics_export_body(metrics).ok_or_else(|| {
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

        for (k, v) in self.headers.iter() {
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
