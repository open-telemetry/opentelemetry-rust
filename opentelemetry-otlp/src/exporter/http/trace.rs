use std::sync::Arc;

use super::OtlpHttpClient;
use http::{header::CONTENT_TYPE, Method};
use opentelemetry::otel_debug;
use opentelemetry_sdk::{
    error::{OTelSdkError, OTelSdkResult},
    trace::{SpanData, SpanExporter},
};

#[cfg(feature = "http-retry")]
use super::{classify_http_export_error, HttpExportError, HttpRetryData};
#[cfg(feature = "http-retry")]
use opentelemetry_sdk::retry::{retry_with_backoff, RetryPolicy};
#[cfg(feature = "http-retry")]
use opentelemetry_sdk::runtime::Tokio;

impl SpanExporter for OtlpHttpClient {
    #[cfg(feature = "http-retry")]
    async fn export(&self, batch: Vec<SpanData>) -> OTelSdkResult {
        let policy = RetryPolicy {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 1600,
            jitter_ms: 100,
        };

        // Build request body once before retry loop
        let (body, content_type, content_encoding) =
            self.build_trace_export_body(batch).map_err(|e| {
                OTelSdkError::InternalFailure(format!("Failed to build request body: {e}"))
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
            "HttpTracesClient.Export",
            || async {
                // Get client
                let client = self
                    .client
                    .lock()
                    .map_err(|e| HttpExportError::new(500, format!("Mutex lock failed: {e}")))?
                    .as_ref()
                    .ok_or_else(|| {
                        HttpExportError::new(500, "Exporter already shutdown".to_string())
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
                    .map_err(|e| {
                        HttpExportError::new(400, format!("Failed to build HTTP request: {e}"))
                    })?;

                for (k, v) in retry_data.headers.iter() {
                    request.headers_mut().insert(k.clone(), v.clone());
                }

                let request_uri = request.uri().to_string();
                otel_debug!(name: "HttpTracesClient.ExportStarted");

                // Send request
                let response = client.send_bytes(request).await.map_err(|e| {
                    HttpExportError::new(0, format!("Network error: {e:?}")) // Network error
                })?;

                let status_code = response.status().as_u16();
                let retry_after = response
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string());

                if !response.status().is_success() {
                    let message = format!(
                        "HTTP export failed. Url: {}, Status: {}, Response: {:?}",
                        request_uri,
                        status_code,
                        response.body()
                    );
                    return Err(match retry_after {
                        Some(retry_after) => {
                            HttpExportError::with_retry_after(status_code, retry_after, message)
                        }
                        None => HttpExportError::new(status_code, message),
                    });
                }

                otel_debug!(name: "HttpTracesClient.ExportSucceeded");
                Ok(())
            },
        )
        .await
        .map_err(|e| OTelSdkError::InternalFailure(e.message))
    }

    #[cfg(not(feature = "http-retry"))]
    async fn export(&self, batch: Vec<SpanData>) -> OTelSdkResult {
        let client = match self
            .client
            .lock()
            .map_err(|e| OTelSdkError::InternalFailure(format!("Mutex lock failed: {e}")))
            .and_then(|g| match &*g {
                Some(client) => Ok(Arc::clone(client)),
                _ => Err(OTelSdkError::AlreadyShutdown),
            }) {
            Ok(client) => client,
            Err(err) => return Err(err),
        };

        let (body, content_type, content_encoding) = match self.build_trace_export_body(batch) {
            Ok(result) => result,
            Err(e) => return Err(OTelSdkError::InternalFailure(e.to_string())),
        };

        let mut request_builder = http::Request::builder()
            .method(Method::POST)
            .uri(&self.collector_endpoint)
            .header(CONTENT_TYPE, content_type);

        if let Some(encoding) = content_encoding {
            request_builder = request_builder.header("Content-Encoding", encoding);
        }

        let mut request = match request_builder.body(body.into()) {
            Ok(req) => req,
            Err(e) => return Err(OTelSdkError::InternalFailure(e.to_string())),
        };

        for (k, v) in self.headers.iter() {
            request.headers_mut().insert(k.clone(), v.clone());
        }

        let request_uri = request.uri().to_string();
        otel_debug!(name: "HttpTracesClient.ExportStarted");
        let response = client
            .send_bytes(request)
            .await
            .map_err(|e| OTelSdkError::InternalFailure(format!("{e:?}")))?;

        if !response.status().is_success() {
            let error = format!(
                "OpenTelemetry trace export failed. Url: {}, Status Code: {}, Response: {:?}",
                request_uri,
                response.status().as_u16(),
                response.body()
            );
            otel_debug!(name: "HttpTracesClient.ExportFailed", error = &error);
            return Err(OTelSdkError::InternalFailure(error));
        }

        otel_debug!(name: "HttpTracesClient.ExportSucceeded");
        Ok(())
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
