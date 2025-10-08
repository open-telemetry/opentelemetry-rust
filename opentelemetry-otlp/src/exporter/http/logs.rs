use super::OtlpHttpClient;
use http::header::CONTENT_ENCODING;
use http::{header::CONTENT_TYPE, Method};
use opentelemetry::otel_debug;
use opentelemetry_sdk::error::{OTelSdkError, OTelSdkResult};
use opentelemetry_sdk::logs::{LogBatch, LogExporter};
#[cfg(feature = "http-retry")]
use std::sync::Arc;
use std::time;

#[cfg(feature = "http-retry")]
use super::{classify_http_export_error, HttpExportError, HttpRetryData};
#[cfg(feature = "http-retry")]
use opentelemetry_sdk::retry::{retry_with_backoff, RetryPolicy};
#[cfg(feature = "http-retry")]
use opentelemetry_sdk::runtime::Tokio;

impl LogExporter for OtlpHttpClient {
    #[cfg(feature = "http-retry")]
    async fn export(&self, batch: LogBatch<'_>) -> OTelSdkResult {
        let policy = RetryPolicy {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 1600,
            jitter_ms: 100,
        };

        // Build request body once before retry loop since LogBatch contains borrowed data
        let (body, content_type, content_encoding) = self
            .build_logs_export_body(batch)
            .map_err(OTelSdkError::InternalFailure)?;

        let retry_data = Arc::new(HttpRetryData {
            body,
            headers: self.headers.clone(),
            endpoint: self.collector_endpoint.to_string(),
        });

        retry_with_backoff(
            Tokio,
            policy,
            classify_http_export_error,
            "HttpLogsClient.Export",
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
                    request_builder = request_builder.header(CONTENT_ENCODING, encoding);
                }

                let mut request = request_builder
                    .body(retry_data.body.clone().into())
                    .map_err(|e| HttpExportError {
                        status_code: 400,
                        retry_after: None,
                        message: format!("Failed to build HTTP request: {e}"),
                    })?;

                for (k, v) in &retry_data.headers {
                    request.headers_mut().insert(k.clone(), v.clone());
                }

                let request_uri = request.uri().to_string();
                otel_debug!(name: "HttpLogsClient.ExportStarted");

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

                otel_debug!(name: "HttpLogsClient.ExportSucceeded");
                Ok(())
            },
        )
        .await
        .map_err(|e| OTelSdkError::InternalFailure(e.message))
    }

    #[cfg(not(feature = "http-retry"))]
    async fn export(&self, batch: LogBatch<'_>) -> OTelSdkResult {
        let client = self
            .client
            .lock()
            .map_err(|e| OTelSdkError::InternalFailure(format!("Mutex lock failed: {e}")))?
            .clone()
            .ok_or(OTelSdkError::AlreadyShutdown)?;

        let (body, content_type, content_encoding) = self
            .build_logs_export_body(batch)
            .map_err(OTelSdkError::InternalFailure)?;

        let mut request_builder = http::Request::builder()
            .method(Method::POST)
            .uri(&self.collector_endpoint)
            .header(CONTENT_TYPE, content_type);

        if let Some(encoding) = content_encoding {
            request_builder = request_builder.header(CONTENT_ENCODING, encoding);
        }

        let mut request = request_builder
            .body(body.into())
            .map_err(|e| OTelSdkError::InternalFailure(e.to_string()))?;

        for (k, v) in &self.headers {
            request.headers_mut().insert(k.clone(), v.clone());
        }

        let request_uri = request.uri().to_string();
        otel_debug!(name: "HttpLogsClient.ExportStarted");
        let response = client
            .send_bytes(request)
            .await
            .map_err(|e| OTelSdkError::InternalFailure(format!("{e:?}")))?;

        if !response.status().is_success() {
            let error = format!(
                "OpenTelemetry logs export failed. Url: {}, Status Code: {}, Response: {:?}",
                request_uri,
                response.status().as_u16(),
                response.body()
            );
            otel_debug!(name: "HttpLogsClient.ExportFailed", error = &error);
            return Err(OTelSdkError::InternalFailure(error));
        }

        otel_debug!(name: "HttpLogsClient.ExportSucceeded");
        Ok(())
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
