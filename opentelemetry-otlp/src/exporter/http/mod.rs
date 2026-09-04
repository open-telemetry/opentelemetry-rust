use super::{
    default_headers, parse_header_string, read_env_var, resolve_timeout, ExporterBuildError,
    OTEL_EXPORTER_OTLP_HTTP_ENDPOINT_DEFAULT,
};
use crate::{
    exporter::ExportConfig, Protocol, OTEL_EXPORTER_OTLP_ENDPOINT, OTEL_EXPORTER_OTLP_HEADERS,
};
use http::{HeaderName, HeaderValue, Uri};
use opentelemetry::otel_debug;
use opentelemetry_http::{Bytes, HttpClient, ResponseBodyTooLarge};
use opentelemetry_proto::transform::common::tonic::ResourceAttributesWithSchema;
#[cfg(feature = "logs")]
use opentelemetry_proto::transform::logs::tonic::group_logs_by_resource_and_scope;
#[cfg(feature = "trace")]
use opentelemetry_proto::transform::trace::tonic::group_spans_by_resource_and_scope;
#[cfg(feature = "logs")]
use opentelemetry_sdk::logs::LogBatch;
#[cfg(feature = "trace")]
use opentelemetry_sdk::trace::SpanData;
#[cfg(feature = "http-proto")]
use prost::Message;
use std::collections::HashMap;
use std::env;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::retry::RetryErrorType;
use crate::retry_classification::http::classify_http_error;
use crate::RetryPolicy;

// Recommended by the OTLP/HTTP specification:
// https://github.com/open-telemetry/opentelemetry-proto/blob/main/docs/specification.md#otlphttp-request
const DEFAULT_MAX_REQUEST_BODY_SIZE: usize = 64 * 1024 * 1024;

// Shared HTTP retry functionality
/// HTTP-specific error wrapper for retry classification
#[derive(Debug)]
pub(crate) enum HttpExportError {
    /// An error with an HTTP status code and optional Retry-After header
    HttpStatus {
        status_code: u16,
        retry_after: Option<String>,
        message: String,
    },
    /// An HTTP response body exceeded the client's configured limit
    ResponseBodyTooLarge { message: String },
}

impl HttpExportError {
    /// Create a new HttpExportError without retry-after header
    pub(crate) fn new(status_code: u16, message: String) -> Self {
        Self::HttpStatus {
            status_code,
            retry_after: None,
            message,
        }
    }

    /// Create a new HttpExportError with retry-after header
    pub(crate) fn with_retry_after(status_code: u16, retry_after: String, message: String) -> Self {
        Self::HttpStatus {
            status_code,
            retry_after: Some(retry_after),
            message,
        }
    }

    /// Create an error for an oversized HTTP response body
    fn response_body_too_large(message: String) -> Self {
        Self::ResponseBodyTooLarge { message }
    }
}

/// Classify HTTP export errors for retry decisions
pub(crate) fn classify_http_export_error(error: &HttpExportError) -> RetryErrorType {
    match error {
        HttpExportError::HttpStatus {
            status_code,
            retry_after,
            ..
        } => classify_http_error(*status_code, retry_after.as_deref()),
        HttpExportError::ResponseBodyTooLarge { .. } => RetryErrorType::NonRetryable,
    }
}

/// Shared HTTP request data for retry attempts - optimizes Arc usage by bundling all data
/// we need to pass into the retry handler
#[derive(Debug)]
pub(crate) struct HttpRetryData {
    pub body: Vec<u8>,
    pub headers: Arc<HashMap<HeaderName, HeaderValue>>,
    pub endpoint: String,
}

#[cfg(feature = "metrics")]
mod metrics;

#[cfg(feature = "metrics")]
use opentelemetry_sdk::metrics::data::ResourceMetrics;

#[cfg(feature = "logs")]
pub(crate) mod logs;

#[cfg(feature = "trace")]
mod trace;

#[cfg(all(
    not(feature = "reqwest-client"),
    not(feature = "reqwest-blocking-client"),
    feature = "hyper-client"
))]
use opentelemetry_http::hyper::HyperClient;

/// Configuration of the http transport
#[derive(Debug, Default)]
pub(crate) struct HttpConfig {
    /// Select the HTTP client
    client: Option<Arc<dyn HttpClient>>,

    /// Additional headers to send to the OTLP endpoint.
    headers: Option<HashMap<String, String>>,

    /// The compression algorithm to use when communicating with the OTLP endpoint.
    compression: Option<crate::Compression>,

    /// The retry policy to use for HTTP requests.
    retry_policy: Option<RetryPolicy>,

    /// Maximum HTTP request body size, before and after compression.
    max_request_body_size: Option<usize>,
}

/// Configuration for the OTLP HTTP exporter.
///
/// ## Examples
///
/// ```no_run
/// # #[cfg(feature="metrics")]
/// use opentelemetry_sdk::metrics::Temporality;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a span exporter you can use when configuring tracer providers
/// # #[cfg(feature="trace")]
/// let span_exporter = opentelemetry_otlp::SpanExporter::builder().with_http().build()?;
///
/// // Create a metrics exporter you can use when configuring meter providers
/// # #[cfg(feature="metrics")]
/// let metrics_exporter = opentelemetry_otlp::MetricExporter::builder()
///     .with_http()
///     .with_temporality(Temporality::default())
///     .build()?;
///
/// // Create a log exporter you can use when configuring logger providers
/// # #[cfg(feature="logs")]
/// let log_exporter = opentelemetry_otlp::LogExporter::builder().with_http().build()?;
/// # Ok(())
/// # }
/// ```
///
#[derive(Debug)]
pub struct HttpExporterBuilder {
    pub(crate) exporter_config: ExportConfig,
    pub(crate) http_config: HttpConfig,
}

impl Default for HttpExporterBuilder {
    fn default() -> Self {
        HttpExporterBuilder {
            exporter_config: ExportConfig::default(),
            http_config: HttpConfig {
                headers: Some(default_headers()),
                ..HttpConfig::default()
            },
        }
    }
}

impl HttpExporterBuilder {
    fn build_client(
        &mut self,
        signal_endpoint_var: &str,
        signal_endpoint_path: &str,
        signal_timeout_var: &str,
        signal_http_headers_var: &str,
        signal_compression_var: &str,
        signal_protocol_var: &str,
    ) -> Result<OtlpHttpClient, ExporterBuildError> {
        let protocol = super::resolve_protocol(signal_protocol_var, self.exporter_config.protocol)?;

        // Validate protocol is compatible with HTTP transport
        #[cfg(feature = "grpc-tonic")]
        if matches!(protocol, Protocol::Grpc) {
            return Err(ExporterBuildError::invalid_configuration(
                "protocol",
                "gRPC protocol is not compatible with HTTP transport; use `.with_tonic()` instead",
            ));
        }

        let endpoint = resolve_http_endpoint(
            signal_endpoint_var,
            signal_endpoint_path,
            self.exporter_config.endpoint.as_deref(),
        )?;

        let compression = self.resolve_compression(signal_compression_var)?;

        // Validate compression is supported at build time
        if let Some(compression_alg) = &compression {
            match compression_alg {
                crate::Compression::Gzip => {
                    #[cfg(not(feature = "gzip-http"))]
                    {
                        return Err(ExporterBuildError::invalid_configuration(
                            "compression",
                            "feature 'gzip-http' is required to use the compression algorithm 'gzip'",
                        ));
                    }
                }
                crate::Compression::Zstd => {
                    #[cfg(not(feature = "zstd-http"))]
                    {
                        return Err(ExporterBuildError::invalid_configuration(
                            "compression",
                            "feature 'zstd-http' is required to use the compression algorithm 'zstd'",
                        ));
                    }
                }
            }
        }

        let timeout = resolve_timeout(signal_timeout_var, self.exporter_config.timeout.as_ref());

        #[allow(unused_mut)] // TODO - clippy thinks mut is not needed, but it is
        let mut http_client = self.http_config.client.take();

        // When multiple HTTP client features are enabled, we use a priority order
        // to select the client. This follows Rust's feature unification principle
        // where features should be additive. Priority (highest to lowest):
        // 1. reqwest-client (async)
        // 2. hyper-client
        // 3. reqwest-blocking-client (default)
        if http_client.is_none() {
            #[cfg(feature = "reqwest-client")]
            {
                let client = reqwest::Client::builder()
                    .timeout(timeout)
                    .build()
                    .map_err(|error| {
                        ExporterBuildError::internal_failure(format!(
                            "failed to build the reqwest HTTP client: {error}"
                        ))
                    })?;
                http_client = Some(Arc::new(client) as Arc<dyn HttpClient>);
            }
            #[cfg(all(not(feature = "reqwest-client"), feature = "hyper-client"))]
            {
                // TODO - support configuring custom connector and executor
                http_client = Some(Arc::new(HyperClient::with_default_connector(timeout, None))
                    as Arc<dyn HttpClient>);
            }
            #[cfg(all(
                not(feature = "reqwest-client"),
                not(feature = "hyper-client"),
                feature = "reqwest-blocking-client"
            ))]
            {
                let timeout_clone = timeout;
                let client = std::thread::Builder::new()
                    .spawn(move || {
                        reqwest::blocking::Client::builder()
                            .timeout(timeout_clone)
                            .build()
                    })
                    .map_err(|error| {
                        ExporterBuildError::internal_failure(format!(
                            "failed to spawn thread for the blocking HTTP client: {error}"
                        ))
                    })?
                    .join()
                    .map_err(|_| {
                        ExporterBuildError::internal_failure(
                            "thread creating the blocking HTTP client panicked",
                        )
                    })?
                    .map_err(|error| {
                        ExporterBuildError::internal_failure(format!(
                            "failed to build the blocking reqwest HTTP client: {error}"
                        ))
                    })?;
                http_client = Some(Arc::new(client) as Arc<dyn HttpClient>);
            }
        }

        let http_client = http_client.ok_or_else(|| {
            ExporterBuildError::invalid_configuration(
                "http_client",
                "no HTTP client is configured; enable an HTTP client feature or provide one with `.with_http_client()`",
            )
        })?;

        #[allow(clippy::mutable_key_type)] // http headers are not mutated
        let mut headers: HashMap<HeaderName, HeaderValue> = self
            .http_config
            .headers
            .take()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(k, v)| {
                Some((
                    HeaderName::from_str(&k).ok()?,
                    HeaderValue::from_str(&v).ok()?,
                ))
            })
            .collect();

        // read headers from env var - signal specific env var is preferred over general
        if let Ok(input) =
            env::var(signal_http_headers_var).or_else(|_| env::var(OTEL_EXPORTER_OTLP_HEADERS))
        {
            add_header_from_string(&input, &mut headers);
        }

        let mut client = OtlpHttpClient::new(
            http_client,
            endpoint,
            headers,
            protocol,
            timeout,
            compression,
            self.http_config.retry_policy.take(),
        );
        if let Some(max_request_body_size) = self.http_config.max_request_body_size {
            client.max_request_body_size = max_request_body_size;
        }
        Ok(client)
    }

    fn resolve_compression(
        &self,
        env_override: &str,
    ) -> Result<Option<crate::Compression>, super::ExporterBuildError> {
        super::resolve_compression_from_env(self.http_config.compression, env_override)
    }

    /// Create a span exporter with the current configuration
    #[cfg(feature = "trace")]
    pub fn build_span_exporter(mut self) -> Result<crate::SpanExporter, ExporterBuildError> {
        use crate::{
            OTEL_EXPORTER_OTLP_TRACES_COMPRESSION, OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
            OTEL_EXPORTER_OTLP_TRACES_HEADERS, OTEL_EXPORTER_OTLP_TRACES_PROTOCOL,
            OTEL_EXPORTER_OTLP_TRACES_TIMEOUT,
        };

        let client = self.build_client(
            OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
            "/v1/traces",
            OTEL_EXPORTER_OTLP_TRACES_TIMEOUT,
            OTEL_EXPORTER_OTLP_TRACES_HEADERS,
            OTEL_EXPORTER_OTLP_TRACES_COMPRESSION,
            OTEL_EXPORTER_OTLP_TRACES_PROTOCOL,
        )?;

        Ok(crate::SpanExporter::from_http(client))
    }

    /// Create a log exporter with the current configuration
    #[cfg(feature = "logs")]
    pub fn build_log_exporter(mut self) -> Result<crate::LogExporter, ExporterBuildError> {
        use crate::{
            OTEL_EXPORTER_OTLP_LOGS_COMPRESSION, OTEL_EXPORTER_OTLP_LOGS_ENDPOINT,
            OTEL_EXPORTER_OTLP_LOGS_HEADERS, OTEL_EXPORTER_OTLP_LOGS_PROTOCOL,
            OTEL_EXPORTER_OTLP_LOGS_TIMEOUT,
        };

        let client = self.build_client(
            OTEL_EXPORTER_OTLP_LOGS_ENDPOINT,
            "/v1/logs",
            OTEL_EXPORTER_OTLP_LOGS_TIMEOUT,
            OTEL_EXPORTER_OTLP_LOGS_HEADERS,
            OTEL_EXPORTER_OTLP_LOGS_COMPRESSION,
            OTEL_EXPORTER_OTLP_LOGS_PROTOCOL,
        )?;

        Ok(crate::LogExporter::from_http(client))
    }

    /// Create a metrics exporter with the current configuration
    #[cfg(feature = "metrics")]
    pub fn build_metrics_exporter(
        mut self,
        temporality: opentelemetry_sdk::metrics::Temporality,
    ) -> Result<crate::MetricExporter, ExporterBuildError> {
        use crate::{
            OTEL_EXPORTER_OTLP_METRICS_COMPRESSION, OTEL_EXPORTER_OTLP_METRICS_ENDPOINT,
            OTEL_EXPORTER_OTLP_METRICS_HEADERS, OTEL_EXPORTER_OTLP_METRICS_PROTOCOL,
            OTEL_EXPORTER_OTLP_METRICS_TIMEOUT,
        };

        let client = self.build_client(
            OTEL_EXPORTER_OTLP_METRICS_ENDPOINT,
            "/v1/metrics",
            OTEL_EXPORTER_OTLP_METRICS_TIMEOUT,
            OTEL_EXPORTER_OTLP_METRICS_HEADERS,
            OTEL_EXPORTER_OTLP_METRICS_COMPRESSION,
            OTEL_EXPORTER_OTLP_METRICS_PROTOCOL,
        )?;

        Ok(crate::MetricExporter::from_http(client, temporality))
    }
}

#[derive(Debug)]
pub(crate) struct OtlpHttpClient {
    client: Mutex<Option<Arc<dyn HttpClient>>>,
    collector_endpoint: Uri,
    headers: Arc<HashMap<HeaderName, HeaderValue>>,
    protocol: Protocol,
    timeout: Duration,
    compression: Option<crate::Compression>,
    retry_policy: RetryPolicy,
    max_request_body_size: usize,
    #[allow(dead_code)]
    // <allow dead> would be removed once we support set_resource for metrics and traces.
    resource: opentelemetry_proto::transform::common::tonic::ResourceAttributesWithSchema,
}

impl OtlpHttpClient {
    /// Shared HTTP export logic used by all exporters with retry support.
    ///
    /// Uses the configured retry policy with the exporter timeout as the deadline.
    /// Delays between retries adapt to the calling context: cooperative
    /// `tokio::time::sleep` inside a Tokio runtime, or `std::thread::sleep`
    /// on bare OS threads (the SDK's default batch processors).
    async fn export_http_with_retry<F, T>(
        &self,
        data: T,
        build_body_fn: F,
        operation_name: &'static str,
    ) -> Result<Bytes, opentelemetry_sdk::error::OTelSdkError>
    where
        F: Fn(&Self, T) -> Result<(Vec<u8>, &'static str, Option<&'static str>), String>,
    {
        use crate::retry::retry_with_backoff;

        // Build request body once before retry loop
        let (body, content_type, content_encoding) = build_body_fn(self, data)
            .map_err(opentelemetry_sdk::error::OTelSdkError::InternalFailure)?;

        let retry_data = Arc::new(HttpRetryData {
            body,
            headers: self.headers.clone(),
            endpoint: self.collector_endpoint.to_string(),
        });

        let response_body = retry_with_backoff(
            &self.retry_policy,
            self.timeout,
            classify_http_export_error,
            operation_name,
            || async {
                self.export_http_once(&retry_data, content_type, content_encoding, operation_name)
                    .await
            },
        )
        .await
        .map_err(|e| {
            let message = match e {
                HttpExportError::HttpStatus { message, .. }
                | HttpExportError::ResponseBodyTooLarge { message } => message,
            };
            opentelemetry_sdk::error::OTelSdkError::InternalFailure(message)
        })?;

        Ok(response_body)
    }

    /// Single HTTP export attempt - shared between retry and no-retry paths
    async fn export_http_once(
        &self,
        retry_data: &HttpRetryData,
        content_type: &'static str,
        content_encoding: Option<&'static str>,
        _operation_name: &'static str,
    ) -> Result<Bytes, HttpExportError> {
        // Get client
        let client = self
            .client
            .lock()
            .map_err(|e| HttpExportError::new(500, format!("Mutex lock failed: {e}")))?
            .as_ref()
            .ok_or_else(|| HttpExportError::new(500, "Exporter already shutdown".to_string()))?
            .clone();

        // Build HTTP request
        let mut request_builder = http::Request::builder()
            .method(http::Method::POST)
            .uri(&retry_data.endpoint)
            .header(http::header::CONTENT_TYPE, content_type);

        if let Some(encoding) = content_encoding {
            request_builder = request_builder.header("Content-Encoding", encoding);
        }

        let mut request = request_builder
            .body(retry_data.body.clone().into())
            .map_err(|e| HttpExportError::new(400, format!("Failed to build HTTP request: {e}")))?;

        for (k, v) in retry_data.headers.iter() {
            request.headers_mut().insert(k.clone(), v.clone());
        }

        let request_uri = request.uri().to_string();
        otel_debug!(name: "HttpClient.ExportStarted");

        // Send request
        let response = client.send_bytes(request).await.map_err(|e| {
            if e.downcast_ref::<ResponseBodyTooLarge>().is_some() {
                let message = e.to_string();
                otel_debug!(
                    name: "HttpClient.ResponseBodyTooLarge",
                    url = request_uri.as_str(),
                    error = message.as_str()
                );
                return HttpExportError::response_body_too_large(message);
            }

            // Connection errors (e.g., "Connection refused", DNS failures) typically
            // indicate user-side misconfigurations and don't contain sensitive data.
            // We don't log at WARN here because SDK processors (BatchLogProcessor,
            // BatchSpanProcessor, PeriodicReader) already log the returned error
            // via otel_error!.
            otel_debug!(
                name: "HttpClient.NetworkError",
                url = request_uri.as_str(),
                error = format!("{e}")
            );
            HttpExportError::new(0, "HTTP export failed: network error".to_string())
        })?;

        let status_code = response.status().as_u16();
        let retry_after = response
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        if !response.status().is_success() {
            // We don't log at WARN here because SDK processors (BatchLogProcessor,
            // BatchSpanProcessor, PeriodicReader) already log the returned error
            // via otel_error!. Response body may contain sensitive information
            // (e.g., auth tokens echoed back by the server), so log it at DEBUG
            // level only.
            otel_debug!(
                name: "HttpClient.StatusError",
                status_code = status_code,
                url = request_uri.as_str(),
                response_body = format!("{:?}", response.body())
            );
            let message = format!("HTTP export failed with status code: {status_code}");
            return Err(match retry_after {
                Some(retry_after) => {
                    HttpExportError::with_retry_after(status_code, retry_after, message)
                }
                None => HttpExportError::new(status_code, message),
            });
        }

        otel_debug!(name: "HttpClient.ExportSucceeded");

        // Return the response, consuming the body to save a copy
        Ok(response.into_body())
    }

    /// Compress data using gzip or zstd if the user has requested it and the relevant feature
    /// has been enabled. If the user has requested it but the feature has not been enabled,
    /// we should catch this at exporter build time and never get here.
    fn process_body(&self, body: Vec<u8>) -> Result<(Vec<u8>, Option<&'static str>), String> {
        self.validate_request_body_size(&body, "uncompressed")?;

        let (processed_body, content_encoding) = match self.compression {
            #[cfg(feature = "gzip-http")]
            Some(crate::Compression::Gzip) => {
                use flate2::{write::GzEncoder, Compression};
                use std::io::Write;

                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(&body).map_err(|e| e.to_string())?;
                let compressed = encoder.finish().map_err(|e| e.to_string())?;
                (compressed, Some("gzip"))
            }
            #[cfg(not(feature = "gzip-http"))]
            Some(crate::Compression::Gzip) => {
                return Err(
                    "gzip compression requested but gzip-http feature not enabled".to_string(),
                );
            }
            #[cfg(feature = "zstd-http")]
            Some(crate::Compression::Zstd) => {
                let compressed = zstd::bulk::compress(&body, 0).map_err(|e| e.to_string())?;
                (compressed, Some("zstd"))
            }
            #[cfg(not(feature = "zstd-http"))]
            Some(crate::Compression::Zstd) => {
                return Err(
                    "zstd compression requested but zstd-http feature not enabled".to_string(),
                );
            }
            None => (body, None),
        };

        if content_encoding.is_some() {
            self.validate_request_body_size(&processed_body, "compressed")?;
        }

        Ok((processed_body, content_encoding))
    }

    fn validate_request_body_size(
        &self,
        body: &[u8],
        representation: &'static str,
    ) -> Result<(), String> {
        if body.len() > self.max_request_body_size {
            return Err(format!(
                "OTLP HTTP {representation} request body is {} bytes, exceeding the configured limit of {} bytes; request will not be sent and telemetry data will be discarded; reduce the batch size, telemetry item size, or configure a larger limit",
                body.len(), self.max_request_body_size
            ));
        }
        Ok(())
    }

    #[allow(clippy::mutable_key_type)] // http headers are not mutated
    fn new(
        client: Arc<dyn HttpClient>,
        collector_endpoint: Uri,
        headers: HashMap<HeaderName, HeaderValue>,
        protocol: Protocol,
        timeout: Duration,
        compression: Option<crate::Compression>,
        retry_policy: Option<RetryPolicy>,
    ) -> Self {
        OtlpHttpClient {
            client: Mutex::new(Some(client)),
            collector_endpoint,
            headers: Arc::new(headers),
            protocol,
            timeout,
            compression,
            retry_policy: retry_policy.unwrap_or_default(),
            max_request_body_size: DEFAULT_MAX_REQUEST_BODY_SIZE,
            resource: ResourceAttributesWithSchema::default(),
        }
    }

    #[cfg(feature = "trace")]
    fn build_trace_export_body(
        &self,
        spans: Vec<SpanData>,
    ) -> Result<(Vec<u8>, &'static str, Option<&'static str>), String> {
        use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
        let resource_spans = group_spans_by_resource_and_scope(spans, &self.resource);

        let req = ExportTraceServiceRequest { resource_spans };
        let (body, content_type) = match self.protocol {
            #[cfg(feature = "http-json")]
            Protocol::HttpJson => match serde_json::to_string_pretty(&req) {
                Ok(json) => (json.into_bytes(), "application/json"),
                Err(e) => {
                    return Err(format!("failed to serialize traces to OTLP/HTTP JSON: {e}"));
                }
            },
            #[cfg(feature = "http-proto")]
            Protocol::HttpBinary => (req.encode_to_vec(), "application/x-protobuf"),
            #[cfg(feature = "grpc-tonic")]
            Protocol::Grpc => {
                unreachable!("HTTP client should not receive Grpc protocol")
            }
        };

        let (processed_body, content_encoding) = self.process_body(body)?;
        Ok((processed_body, content_type, content_encoding))
    }

    #[cfg(feature = "logs")]
    fn build_logs_export_body(
        &self,
        logs: LogBatch<'_>,
    ) -> Result<(Vec<u8>, &'static str, Option<&'static str>), String> {
        use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
        let resource_logs = group_logs_by_resource_and_scope(&logs, &self.resource);
        let req = ExportLogsServiceRequest { resource_logs };

        let (body, content_type) = match self.protocol {
            #[cfg(feature = "http-json")]
            Protocol::HttpJson => match serde_json::to_string_pretty(&req) {
                Ok(json) => (json.into_bytes(), "application/json"),
                Err(e) => {
                    return Err(format!("failed to serialize logs to OTLP/HTTP JSON: {e}"));
                }
            },
            #[cfg(feature = "http-proto")]
            Protocol::HttpBinary => (req.encode_to_vec(), "application/x-protobuf"),
            #[cfg(feature = "grpc-tonic")]
            Protocol::Grpc => {
                unreachable!("HTTP client should not receive Grpc protocol")
            }
        };

        let (processed_body, content_encoding) = self.process_body(body)?;
        Ok((processed_body, content_type, content_encoding))
    }

    #[cfg(feature = "metrics")]
    fn build_metrics_export_body(
        &self,
        metrics: &ResourceMetrics,
    ) -> Result<(Vec<u8>, &'static str, Option<&'static str>), String> {
        use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;

        let req: ExportMetricsServiceRequest = metrics.into();

        let (body, content_type) = match self.protocol {
            #[cfg(feature = "http-json")]
            Protocol::HttpJson => match serde_json::to_string_pretty(&req) {
                Ok(json) => (json.into_bytes(), "application/json"),
                Err(e) => {
                    return Err(format!(
                        "failed to serialize metrics to OTLP/HTTP JSON: {e}"
                    ));
                }
            },
            #[cfg(feature = "http-proto")]
            Protocol::HttpBinary => (req.encode_to_vec(), "application/x-protobuf"),
            #[cfg(feature = "grpc-tonic")]
            Protocol::Grpc => {
                unreachable!("HTTP client should not receive Grpc protocol")
            }
        };

        let (processed_body, content_encoding) = self.process_body(body)?;
        Ok((processed_body, content_type, content_encoding))
    }
}

fn build_endpoint_uri(endpoint: &str, path: &str) -> Result<Uri, http::uri::InvalidUri> {
    let path = if endpoint.ends_with('/') && path.starts_with('/') {
        path.strip_prefix('/').unwrap()
    } else {
        path
    };
    let endpoint = format!("{endpoint}{path}");
    endpoint.parse()
}

fn invalid_endpoint_env(
    variable: &str,
    value: &str,
    error: http::uri::InvalidUri,
) -> ExporterBuildError {
    ExporterBuildError::invalid_configuration(
        variable,
        format!("invalid endpoint '{value}': {error}"),
    )
}

// see https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/protocol/exporter.md#endpoint-urls-for-otlphttp
fn resolve_http_endpoint(
    signal_endpoint_var: &str,
    signal_endpoint_path: &str,
    provided_endpoint: Option<&str>,
) -> Result<Uri, ExporterBuildError> {
    // programmatic configuration overrides any value set via environment variables
    if let Some(provider_endpoint) = provided_endpoint.filter(|s| !s.is_empty()) {
        provider_endpoint.parse().map_err(|error| {
            ExporterBuildError::invalid_configuration(
                "endpoint",
                format!("invalid endpoint '{provider_endpoint}': {error}"),
            )
        })
    } else if let Some(endpoint) = read_env_var(signal_endpoint_var)? {
        // per signal env var is not modified
        endpoint
            .parse()
            .map_err(|error| invalid_endpoint_env(signal_endpoint_var, &endpoint, error))
    } else if let Some(endpoint) = read_env_var(OTEL_EXPORTER_OTLP_ENDPOINT)? {
        // if signal env var is not set, then we check if the OTEL_EXPORTER_OTLP_ENDPOINT env var is set
        build_endpoint_uri(&endpoint, signal_endpoint_path)
            .map_err(|error| invalid_endpoint_env(OTEL_EXPORTER_OTLP_ENDPOINT, &endpoint, error))
    } else {
        build_endpoint_uri(
            OTEL_EXPORTER_OTLP_HTTP_ENDPOINT_DEFAULT,
            signal_endpoint_path,
        )
        .map_err(|error| {
            ExporterBuildError::internal_failure(format!(
                "the default HTTP endpoint is invalid: {error}"
            ))
        })
    }
}

#[allow(clippy::mutable_key_type)] // http headers are not mutated
fn add_header_from_string(input: &str, headers: &mut HashMap<HeaderName, HeaderValue>) {
    headers.extend(parse_header_string(input).filter_map(|(key, value)| {
        Some((
            HeaderName::from_str(key).ok()?,
            HeaderValue::from_str(&value).ok()?,
        ))
    }));
}

/// Expose interface for modifying builder config.
pub(crate) trait HasHttpConfig {
    /// Return a mutable reference to the config within the exporter builders.
    fn http_client_config(&mut self) -> &mut HttpConfig;
}

/// Expose interface for modifying builder config.
impl HasHttpConfig for HttpExporterBuilder {
    fn http_client_config(&mut self) -> &mut HttpConfig {
        &mut self.http_config
    }
}

/// Expose methods to override HTTP-specific configuration.
///
/// ## Examples
/// ```
/// # #[cfg(all(feature = "trace", feature = "grpc-tonic"))]
/// # {
/// use crate::opentelemetry_otlp::WithHttpConfig;
/// let exporter_builder = opentelemetry_otlp::SpanExporter::builder()
///     .with_http()
///     .with_headers(std::collections::HashMap::new());
/// # }
/// ```
pub trait WithHttpConfig {
    /// Assign client implementation
    fn with_http_client<T: HttpClient + 'static>(self, client: T) -> Self;

    /// Set additional headers to send to the collector.
    fn with_headers(self, headers: HashMap<String, String>) -> Self;

    /// Set the compression algorithm to use when communicating with the collector.
    fn with_compression(self, compression: crate::Compression) -> Self;

    /// Set the retry policy for HTTP requests.
    fn with_retry_policy(self, policy: RetryPolicy) -> Self;

    /// Set the maximum HTTP request body size in bytes.
    ///
    /// The limit is enforced both before and after compression. Requests that
    /// exceed it are discarded without being sent or retried. Reduce the batch
    /// size or telemetry item size if requests exceed the configured limit.
    /// The default is 64 MiB.
    fn with_max_request_body_size(self, max_size: usize) -> Self;
}

impl<B: HasHttpConfig> WithHttpConfig for B {
    fn with_http_client<T: HttpClient + 'static>(mut self, client: T) -> Self {
        self.http_client_config().client = Some(Arc::new(client));
        self
    }

    fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        // headers will be wrapped, so we must do some logic to unwrap first.
        let http_client_headers = self
            .http_client_config()
            .headers
            .get_or_insert(HashMap::new());
        headers.into_iter().for_each(|(key, value)| {
            http_client_headers.insert(key, super::url_decode(&value).unwrap_or(value));
        });
        self
    }

    fn with_compression(mut self, compression: crate::Compression) -> Self {
        self.http_client_config().compression = Some(compression);
        self
    }

    fn with_retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.http_client_config().retry_policy = Some(policy);
        self
    }

    fn with_max_request_body_size(mut self, max_size: usize) -> Self {
        self.http_client_config().max_request_body_size = Some(max_size);
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::exporter::http::HttpConfig;
    use crate::exporter::tests::run_env_test;
    use crate::{
        HttpExporterBuilder, WithExportConfig, WithHttpConfig, OTEL_EXPORTER_OTLP_ENDPOINT,
        OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
    };

    use super::{build_endpoint_uri, resolve_http_endpoint};

    #[test]
    fn test_append_signal_path_to_generic_env() {
        run_env_test(
            vec![(OTEL_EXPORTER_OTLP_ENDPOINT, "http://example.com")],
            || {
                let endpoint =
                    resolve_http_endpoint(OTEL_EXPORTER_OTLP_TRACES_ENDPOINT, "/v1/traces", None)
                        .unwrap();
                assert_eq!(endpoint, "http://example.com/v1/traces");
            },
        )
    }

    #[test]
    fn test_not_append_signal_path_to_signal_env() {
        run_env_test(
            vec![(OTEL_EXPORTER_OTLP_TRACES_ENDPOINT, "http://example.com")],
            || {
                let endpoint =
                    resolve_http_endpoint(OTEL_EXPORTER_OTLP_TRACES_ENDPOINT, "/v1/traces", None)
                        .unwrap();
                assert_eq!(endpoint, "http://example.com");
            },
        )
    }

    #[test]
    fn test_priority_of_signal_env_over_generic_env() {
        run_env_test(
            vec![
                (OTEL_EXPORTER_OTLP_TRACES_ENDPOINT, "http://example.com"),
                (OTEL_EXPORTER_OTLP_ENDPOINT, "http://wrong.com"),
            ],
            || {
                let endpoint = super::resolve_http_endpoint(
                    OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
                    "/v1/traces",
                    None,
                )
                .unwrap();
                assert_eq!(endpoint, "http://example.com");
            },
        );
    }

    #[test]
    fn test_priority_of_code_based_config_over_envs() {
        run_env_test(
            vec![
                (OTEL_EXPORTER_OTLP_TRACES_ENDPOINT, "http://example.com"),
                (OTEL_EXPORTER_OTLP_ENDPOINT, "http://wrong.com"),
            ],
            || {
                let endpoint = super::resolve_http_endpoint(
                    OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
                    "/v1/traces",
                    Some("http://localhost:4317"),
                )
                .unwrap();
                assert_eq!(endpoint, "http://localhost:4317");
            },
        );
    }

    #[test]
    fn test_use_default_when_empty_string_for_option() {
        run_env_test(vec![], || {
            let endpoint =
                super::resolve_http_endpoint("non_existent_var", "/v1/traces", Some("")).unwrap();
            assert_eq!(endpoint, "http://localhost:4318/v1/traces");
        });
    }

    #[test]
    fn test_use_default_when_others_missing() {
        run_env_test(vec![], || {
            let endpoint =
                super::resolve_http_endpoint("NON_EXISTENT_VAR", "/v1/traces", None).unwrap();
            assert_eq!(endpoint, "http://localhost:4318/v1/traces");
        });
    }

    #[test]
    fn test_build_endpoint_uri() {
        let uri = build_endpoint_uri("https://example.com", "/v1/traces").unwrap();
        assert_eq!(uri, "https://example.com/v1/traces");

        // Should be no duplicate slahes:
        let uri = build_endpoint_uri("https://example.com/", "/v1/traces").unwrap();
        assert_eq!(uri, "https://example.com/v1/traces");

        // Append paths properly:
        let uri = build_endpoint_uri("https://example.com/additional/path/", "/v1/traces").unwrap();
        assert_eq!(uri, "https://example.com/additional/path/v1/traces");
    }

    #[test]
    fn test_invalid_uri_in_signal_env_returns_error() {
        run_env_test(
            vec![
                (
                    OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
                    "-*/*-/*-//-/-/invalid-uri",
                ),
                (OTEL_EXPORTER_OTLP_ENDPOINT, "http://example.com"),
            ],
            || {
                let endpoint = super::resolve_http_endpoint(
                    OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
                    "/v1/traces",
                    None,
                );
                assert!(matches!(
                    endpoint,
                    Err(crate::exporter::ExporterBuildError::InvalidConfiguration(message))
                        if message.contains(OTEL_EXPORTER_OTLP_TRACES_ENDPOINT)
                            && message.matches("-*/*-/*-//-/-/invalid-uri").count() == 1
                ));
            },
        );
    }

    #[test]
    fn test_invalid_uri_in_generic_env_returns_error() {
        run_env_test(
            vec![(OTEL_EXPORTER_OTLP_ENDPOINT, "-*/*-/*-//-/-/invalid-uri")],
            || {
                let endpoint = super::resolve_http_endpoint(
                    OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
                    "/v1/traces",
                    None,
                );
                assert!(matches!(
                    endpoint,
                    Err(crate::exporter::ExporterBuildError::InvalidConfiguration(message))
                        if message.contains(OTEL_EXPORTER_OTLP_ENDPOINT)
                            && message.matches("-*/*-/*-//-/-/invalid-uri").count() == 1
                ));
            },
        );
    }

    #[test]
    fn test_empty_endpoint_envs_are_treated_as_unset() {
        run_env_test(
            vec![
                (OTEL_EXPORTER_OTLP_TRACES_ENDPOINT, ""),
                (OTEL_EXPORTER_OTLP_ENDPOINT, ""),
            ],
            || {
                let endpoint = super::resolve_http_endpoint(
                    OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
                    "/v1/traces",
                    None,
                )
                .unwrap();
                assert_eq!(endpoint, "http://localhost:4318/v1/traces");
            },
        );
    }

    #[cfg(unix)]
    #[test]
    fn test_non_unicode_endpoint_env_returns_error() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        temp_env::with_var(
            OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
            Some(OsStr::from_bytes(b"http://example.com/\x80")),
            || {
                let endpoint = super::resolve_http_endpoint(
                    OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
                    "/v1/traces",
                    None,
                );
                assert!(matches!(
                    endpoint,
                    Err(crate::exporter::ExporterBuildError::InvalidConfiguration(message))
                        if message.contains(OTEL_EXPORTER_OTLP_TRACES_ENDPOINT)
                            && message.contains("not valid Unicode")
                ));
            },
        );
    }

    #[test]
    fn test_invalid_programmatic_endpoint_returns_error() {
        run_env_test(vec![], || {
            let result = super::resolve_http_endpoint(
                OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
                "/v1/traces",
                Some("-*/*-/*-//-/-/yet-another-invalid-uri"),
            );
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_add_header_from_string() {
        use http::{HeaderName, HeaderValue};
        use std::collections::HashMap;
        let test_cases = vec![
            // Format: (input_str, expected_headers)
            ("k1=v1", vec![("k1", "v1")]),
            ("k1=v1,k2=v2", vec![("k1", "v1"), ("k2", "v2")]),
            ("k1=v1=10,k2,k3", vec![("k1", "v1=10")]),
            ("k1=v1,,,k2,k3=10", vec![("k1", "v1"), ("k3", "10")]),
        ];

        for (input_str, expected_headers) in test_cases {
            #[allow(clippy::mutable_key_type)] // http headers are not mutated
            let mut headers: HashMap<HeaderName, HeaderValue> = HashMap::new();
            super::add_header_from_string(input_str, &mut headers);

            assert_eq!(
                headers.len(),
                expected_headers.len(),
                "Failed on input: {input_str}"
            );

            for (expected_key, expected_value) in expected_headers {
                assert_eq!(
                    headers.get(&HeaderName::from_static(expected_key)),
                    Some(&HeaderValue::from_static(expected_value)),
                    "Failed on key: {expected_key} with input: {input_str}"
                );
            }
        }
    }

    #[test]
    fn test_merge_header_from_string() {
        use http::{HeaderName, HeaderValue};
        use std::collections::HashMap;
        #[allow(clippy::mutable_key_type)] // http headers are not mutated
        let mut headers: HashMap<HeaderName, HeaderValue> = std::collections::HashMap::new();
        headers.insert(
            HeaderName::from_static("k1"),
            HeaderValue::from_static("v1"),
        );
        headers.insert(
            HeaderName::from_static("k2"),
            HeaderValue::from_static("v2"),
        );
        let test_cases = vec![
            // Format: (input_str, expected_headers)
            ("k1=v1_new", vec![("k1", "v1_new"), ("k2", "v2")]),
            (
                "k3=val=10,22,34,k4=,k5=10",
                vec![
                    ("k1", "v1_new"),
                    ("k2", "v2"),
                    ("k3", "val=10"),
                    ("k5", "10"),
                ],
            ),
        ];

        for (input_str, expected_headers) in test_cases {
            super::add_header_from_string(input_str, &mut headers);

            assert_eq!(
                headers.len(),
                expected_headers.len(),
                "Failed on input: {input_str}"
            );

            for (expected_key, expected_value) in expected_headers {
                assert_eq!(
                    headers.get(&HeaderName::from_static(expected_key)),
                    Some(&HeaderValue::from_static(expected_value)),
                    "Failed on key: {expected_key} with input: {input_str}"
                );
            }
        }
    }

    #[test]
    fn test_http_exporter_builder_with_headers() {
        use std::collections::HashMap;
        // Arrange
        let initial_headers = HashMap::from([("k1".to_string(), "v1".to_string())]);
        let extra_headers = HashMap::from([
            ("k2".to_string(), "v2".to_string()),
            ("k3".to_string(), "v3".to_string()),
        ]);
        let expected_headers = initial_headers.iter().chain(extra_headers.iter()).fold(
            HashMap::new(),
            |mut acc, (k, v)| {
                acc.insert(k.clone(), v.clone());
                acc
            },
        );
        let builder = HttpExporterBuilder {
            http_config: HttpConfig {
                client: None,
                headers: Some(initial_headers),
                compression: None,
                retry_policy: None,
                max_request_body_size: None,
            },
            exporter_config: crate::exporter::ExportConfig::default(),
        };

        // Act
        let builder = builder.with_headers(extra_headers);

        // Assert
        assert_eq!(
            builder
                .http_config
                .headers
                .clone()
                .expect("headers should always be Some"),
            expected_headers,
        );
    }

    #[test]
    fn test_http_exporter_endpoint() {
        // default endpoint should add signal path
        run_env_test(vec![], || {
            let exporter = HttpExporterBuilder::default();

            let url = resolve_http_endpoint(
                OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
                "/v1/traces",
                exporter.exporter_config.endpoint.as_deref(),
            )
            .unwrap();

            assert_eq!(url, "http://localhost:4318/v1/traces");
        });

        // if builder endpoint is set, it should not add signal path
        run_env_test(vec![], || {
            let exporter = HttpExporterBuilder::default()
                .with_endpoint("http://localhost:4318/v1/tracesbutnotreally");

            let url = resolve_http_endpoint(
                OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
                "/v1/traces",
                exporter.exporter_config.endpoint.as_deref(),
            )
            .unwrap();

            assert_eq!(url, "http://localhost:4318/v1/tracesbutnotreally");
        });
    }

    #[cfg(feature = "gzip-http")]
    mod compression_tests {
        use super::super::OtlpHttpClient;
        use flate2::read::GzDecoder;
        use opentelemetry_http::{Bytes, HttpClient};
        use std::io::Read;

        #[cfg(feature = "http-proto")]
        #[test]
        fn test_gzip_compression_and_decompression() {
            let client = OtlpHttpClient::new(
                std::sync::Arc::new(MockHttpClient),
                "http://localhost:4318".parse().unwrap(),
                std::collections::HashMap::new(),
                crate::Protocol::HttpBinary,
                std::time::Duration::from_secs(10),
                Some(crate::Compression::Gzip),
                None,
            );

            // Test with some sample data
            let test_data = b"Hello, world! This is test data for compression.";
            let result = client.process_body(test_data.to_vec()).unwrap();
            let (compressed_body, content_encoding) = result;

            // Verify encoding header is set
            assert_eq!(content_encoding, Some("gzip"));

            // Verify we can decompress the body
            let mut decoder = GzDecoder::new(&compressed_body[..]);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed).unwrap();

            // Verify decompressed data matches original
            assert_eq!(decompressed, test_data);
            // Verify compression actually happened (compressed should be different)
            assert_ne!(compressed_body, test_data.to_vec());
        }

        #[cfg(all(feature = "http-proto", feature = "gzip-http"))]
        #[test]
        fn request_body_limit_applies_before_and_after_compression() {
            let mut client = OtlpHttpClient::new(
                std::sync::Arc::new(MockHttpClient),
                "http://localhost:4318".parse().unwrap(),
                std::collections::HashMap::new(),
                crate::Protocol::HttpBinary,
                std::time::Duration::from_secs(10),
                Some(crate::Compression::Gzip),
                None,
            );
            client.max_request_body_size = 1;

            let uncompressed_error = client.process_body(vec![0; 2]).unwrap_err();
            assert!(uncompressed_error.contains("uncompressed request body is 2 bytes"));

            let compressed_error = client.process_body(vec![0]).unwrap_err();
            assert!(compressed_error.contains("compressed request body is"));
        }

        #[cfg(all(feature = "http-proto", feature = "zstd-http"))]
        #[test]
        fn test_zstd_compression_and_decompression() {
            let client = OtlpHttpClient::new(
                std::sync::Arc::new(MockHttpClient),
                "http://localhost:4318".parse().unwrap(),
                std::collections::HashMap::new(),
                crate::Protocol::HttpBinary,
                std::time::Duration::from_secs(10),
                Some(crate::Compression::Zstd),
                None,
            );

            // Test with some sample data
            let test_data = b"Hello, world! This is test data for zstd compression.";
            let result = client.process_body(test_data.to_vec()).unwrap();
            let (compressed_body, content_encoding) = result;

            // Verify encoding header is set
            assert_eq!(content_encoding, Some("zstd"));

            // Verify we can decompress the body
            let decompressed = zstd::bulk::decompress(&compressed_body, test_data.len()).unwrap();

            // Verify decompressed data matches original
            assert_eq!(decompressed, test_data);
            // Verify compression actually happened (compressed should be different)
            assert_ne!(compressed_body, test_data.to_vec());
        }

        #[cfg(feature = "http-proto")]
        #[test]
        fn test_no_compression_when_disabled() {
            let client = OtlpHttpClient::new(
                std::sync::Arc::new(MockHttpClient),
                "http://localhost:4318".parse().unwrap(),
                std::collections::HashMap::new(),
                crate::Protocol::HttpBinary,
                std::time::Duration::from_secs(10),
                None, // No compression
                None,
            );

            let body = vec![1, 2, 3, 4];
            let result = client.process_body(body.clone()).unwrap();
            let (result_body, content_encoding) = result;

            // Body should be unchanged and no encoding header
            assert_eq!(result_body, body);
            assert_eq!(content_encoding, None);
        }

        #[cfg(all(feature = "http-proto", not(feature = "gzip-http")))]
        #[test]
        fn test_gzip_error_when_feature_disabled() {
            let client = OtlpHttpClient::new(
                std::sync::Arc::new(MockHttpClient),
                "http://localhost:4318".parse().unwrap(),
                std::collections::HashMap::new(),
                crate::Protocol::HttpBinary,
                std::time::Duration::from_secs(10),
                Some(crate::Compression::Gzip),
                None,
            );

            let body = vec![1, 2, 3, 4];
            let result = client.process_body(body);

            // Should return error when gzip requested but feature not enabled
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .contains("gzip-http feature not enabled"));
        }

        #[cfg(all(feature = "http-proto", not(feature = "zstd-http")))]
        #[test]
        fn test_zstd_error_when_feature_disabled() {
            let client = OtlpHttpClient::new(
                std::sync::Arc::new(MockHttpClient),
                "http://localhost:4318".parse().unwrap(),
                std::collections::HashMap::new(),
                crate::Protocol::HttpBinary,
                std::time::Duration::from_secs(10),
                Some(crate::Compression::Zstd),
                None,
            );

            let body = vec![1, 2, 3, 4];
            let result = client.process_body(body);

            // Should return error when zstd requested but feature not enabled
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .contains("zstd-http feature not enabled"));
        }

        // Mock HTTP client for testing
        #[derive(Debug)]
        struct MockHttpClient;

        #[async_trait::async_trait]
        impl HttpClient for MockHttpClient {
            async fn send_bytes(
                &self,
                _request: http::Request<Bytes>,
            ) -> Result<http::Response<Bytes>, opentelemetry_http::HttpError> {
                Ok(http::Response::builder()
                    .status(200)
                    .body(Bytes::new())
                    .unwrap())
            }
        }
    }

    mod export_body_tests {
        use super::super::OtlpHttpClient;
        use opentelemetry_http::{Bytes, HttpClient};
        use std::collections::HashMap;
        use std::time::Duration;

        #[derive(Debug)]
        struct MockHttpClient;

        #[async_trait::async_trait]
        impl HttpClient for MockHttpClient {
            async fn send_bytes(
                &self,
                _request: http::Request<Bytes>,
            ) -> Result<http::Response<Bytes>, opentelemetry_http::HttpError> {
                Ok(http::Response::builder()
                    .status(200)
                    .body(Bytes::new())
                    .unwrap())
            }
        }

        fn create_test_client(
            protocol: crate::Protocol,
            compression: Option<crate::Compression>,
        ) -> OtlpHttpClient {
            OtlpHttpClient::new(
                std::sync::Arc::new(MockHttpClient),
                "http://localhost:4318".parse().unwrap(),
                HashMap::new(),
                protocol,
                std::time::Duration::from_secs(10),
                compression,
                None,
            )
        }

        #[cfg(feature = "http-proto")]
        #[test]
        fn request_body_at_configured_limit_is_accepted() {
            let mut client = create_test_client(crate::Protocol::HttpBinary, None);
            client.max_request_body_size = 4;

            let (body, content_encoding) = client.process_body(vec![0; 4]).unwrap();

            assert_eq!(body.len(), 4);
            assert_eq!(content_encoding, None);
        }

        #[cfg(feature = "http-proto")]
        #[test]
        fn request_body_over_configured_limit_is_rejected() {
            let mut client = create_test_client(crate::Protocol::HttpBinary, None);
            client.max_request_body_size = 4;

            let error = client.process_body(vec![0; 5]).unwrap_err();

            assert!(error.contains("uncompressed request body is 5 bytes"));
            assert!(error.contains("configured limit of 4 bytes"));
            assert!(error.contains("request will not be sent"));
        }

        #[cfg(feature = "http-proto")]
        #[test]
        fn default_request_body_limit_is_64_mib() {
            let client = create_test_client(crate::Protocol::HttpBinary, None);

            assert_eq!(
                client.max_request_body_size,
                super::super::DEFAULT_MAX_REQUEST_BODY_SIZE
            );
        }

        fn create_test_span_data() -> opentelemetry_sdk::trace::SpanData {
            use opentelemetry::trace::Status;
            use opentelemetry::trace::{
                SpanContext, SpanId, SpanKind, TraceFlags, TraceId, TraceState,
            };
            use opentelemetry_sdk::trace::{SpanData, SpanEvents, SpanLinks};
            use std::borrow::Cow;
            use std::time::{Duration, SystemTime};

            let span_context = SpanContext::new(
                TraceId::from(123),
                SpanId::from(456),
                TraceFlags::default(),
                false,
                TraceState::default(),
            );
            SpanData {
                span_context,
                parent_span_id: SpanId::from(0),
                parent_span_is_remote: false,
                span_kind: SpanKind::Internal,
                name: Cow::Borrowed("test_span"),
                start_time: SystemTime::UNIX_EPOCH,
                end_time: SystemTime::UNIX_EPOCH + Duration::from_secs(1),
                attributes: vec![],
                dropped_attributes_count: 0,
                events: SpanEvents::default(),
                links: SpanLinks::default(),
                status: Status::Unset,
                instrumentation_scope: opentelemetry::InstrumentationScope::default(),
            }
        }

        #[cfg(all(feature = "trace", feature = "http-proto"))]
        #[test]
        fn test_build_trace_export_body_binary_protocol() {
            let client = create_test_client(crate::Protocol::HttpBinary, None);
            let span_data = create_test_span_data();

            let result = client.build_trace_export_body(vec![span_data]).unwrap();
            let (_body, content_type, content_encoding) = result;

            assert_eq!(content_type, "application/x-protobuf");
            assert_eq!(content_encoding, None);
        }

        #[cfg(all(feature = "trace", feature = "http-json"))]
        #[test]
        fn test_build_trace_export_body_json_protocol() {
            let client = create_test_client(crate::Protocol::HttpJson, None);
            let span_data = create_test_span_data();

            let result = client.build_trace_export_body(vec![span_data]).unwrap();
            let (_body, content_type, content_encoding) = result;

            assert_eq!(content_type, "application/json");
            assert_eq!(content_encoding, None);
        }

        #[cfg(all(feature = "http-proto", feature = "trace", feature = "gzip-http"))]
        #[test]
        fn test_build_trace_export_body_with_compression() {
            let client =
                create_test_client(crate::Protocol::HttpBinary, Some(crate::Compression::Gzip));
            let span_data = create_test_span_data();

            let result = client.build_trace_export_body(vec![span_data]).unwrap();
            let (_body, content_type, content_encoding) = result;

            assert_eq!(content_type, "application/x-protobuf");
            assert_eq!(content_encoding, Some("gzip"));
        }

        #[cfg(feature = "logs")]
        fn create_test_log_batch() -> opentelemetry_sdk::logs::LogBatch<'static> {
            use opentelemetry_sdk::logs::LogBatch;

            // Use empty batch for simplicity - the method should still handle protocol/compression correctly
            LogBatch::new(&[])
        }

        #[cfg(all(feature = "http-proto", feature = "logs"))]
        #[test]
        fn test_build_logs_export_body_binary_protocol() {
            let client = create_test_client(crate::Protocol::HttpBinary, None);
            let batch = create_test_log_batch();

            let result = client.build_logs_export_body(batch).unwrap();
            let (_body, content_type, content_encoding) = result;

            assert_eq!(content_type, "application/x-protobuf");
            assert_eq!(content_encoding, None);
        }

        #[cfg(all(feature = "logs", feature = "http-json"))]
        #[test]
        fn test_build_logs_export_body_json_protocol() {
            let client = create_test_client(crate::Protocol::HttpJson, None);
            let batch = create_test_log_batch();

            let result = client.build_logs_export_body(batch).unwrap();
            let (_body, content_type, content_encoding) = result;

            assert_eq!(content_type, "application/json");
            assert_eq!(content_encoding, None);
        }

        #[cfg(all(feature = "http-proto", feature = "logs", feature = "gzip-http"))]
        #[test]
        fn test_build_logs_export_body_with_compression() {
            let client =
                create_test_client(crate::Protocol::HttpBinary, Some(crate::Compression::Gzip));
            let batch = create_test_log_batch();

            let result = client.build_logs_export_body(batch).unwrap();
            let (_body, content_type, content_encoding) = result;

            assert_eq!(content_type, "application/x-protobuf");
            assert_eq!(content_encoding, Some("gzip"));
        }

        #[cfg(all(feature = "http-proto", feature = "metrics"))]
        #[test]
        fn test_build_metrics_export_body_binary_protocol() {
            use opentelemetry_sdk::metrics::data::ResourceMetrics;

            let client = create_test_client(crate::Protocol::HttpBinary, None);
            let metrics = ResourceMetrics::default();

            let result = client.build_metrics_export_body(&metrics).unwrap();
            let (_body, content_type, content_encoding) = result;

            assert_eq!(content_type, "application/x-protobuf");
            assert_eq!(content_encoding, None);
        }

        #[cfg(all(feature = "metrics", feature = "http-json"))]
        #[test]
        fn test_build_metrics_export_body_json_protocol() {
            use opentelemetry_sdk::metrics::data::ResourceMetrics;

            let client = create_test_client(crate::Protocol::HttpJson, None);
            let metrics = ResourceMetrics::default();

            let result = client.build_metrics_export_body(&metrics).unwrap();
            let (_body, content_type, content_encoding) = result;

            assert_eq!(content_type, "application/json");
            assert_eq!(content_encoding, None);
        }

        #[cfg(all(feature = "http-proto", feature = "metrics", feature = "gzip-http"))]
        #[test]
        fn test_build_metrics_export_body_with_compression() {
            use opentelemetry_sdk::metrics::data::ResourceMetrics;

            let client =
                create_test_client(crate::Protocol::HttpBinary, Some(crate::Compression::Gzip));
            let metrics = ResourceMetrics::default();

            let result = client.build_metrics_export_body(&metrics).unwrap();
            let (_body, content_type, content_encoding) = result;

            assert_eq!(content_type, "application/x-protobuf");
            assert_eq!(content_encoding, Some("gzip"));
        }

        #[cfg(all(
            feature = "http-proto",
            feature = "metrics",
            not(feature = "gzip-http")
        ))]
        #[test]
        fn test_build_metrics_export_body_returns_compression_error() {
            use opentelemetry_sdk::metrics::data::ResourceMetrics;

            let client =
                create_test_client(crate::Protocol::HttpBinary, Some(crate::Compression::Gzip));
            let metrics = ResourceMetrics::default();

            let error = client.build_metrics_export_body(&metrics).unwrap_err();
            assert!(error.contains("gzip-http feature not enabled"));
        }

        #[test]
        fn test_resolve_compression_uses_generic_env_fallback() {
            use super::super::HttpExporterBuilder;
            use crate::exporter::tests::run_env_test;

            // Test that generic OTEL_EXPORTER_OTLP_COMPRESSION is used when signal-specific env var is not set
            run_env_test(
                vec![(crate::OTEL_EXPORTER_OTLP_COMPRESSION, "gzip")],
                || {
                    let builder = HttpExporterBuilder::default();
                    let result = builder
                        .resolve_compression("NONEXISTENT_SIGNAL_COMPRESSION")
                        .unwrap();
                    assert_eq!(result, Some(crate::Compression::Gzip));
                },
            );
        }

        #[cfg(all(feature = "trace", not(feature = "gzip-http")))]
        #[test]
        fn test_build_span_exporter_with_gzip_without_feature() {
            use super::super::HttpExporterBuilder;
            use crate::{ExporterBuildError, WithHttpConfig};

            let builder = HttpExporterBuilder::default().with_compression(crate::Compression::Gzip);

            let result = builder.build_span_exporter();
            assert!(matches!(
                result,
                Err(ExporterBuildError::InvalidConfiguration(message))
                    if message.contains("gzip-http")
            ));
        }

        #[cfg(all(feature = "trace", not(feature = "zstd-http")))]
        #[test]
        fn test_build_span_exporter_with_zstd_without_feature() {
            use super::super::HttpExporterBuilder;
            use crate::{ExporterBuildError, WithHttpConfig};

            let builder = HttpExporterBuilder::default().with_compression(crate::Compression::Zstd);

            let result = builder.build_span_exporter();
            assert!(matches!(
                result,
                Err(ExporterBuildError::InvalidConfiguration(message))
                    if message.contains("zstd-http")
            ));
        }

        #[test]
        fn test_with_max_request_body_size() {
            use super::super::HttpExporterBuilder;
            use crate::WithHttpConfig;

            let builder = HttpExporterBuilder::default().with_max_request_body_size(1024);

            assert_eq!(builder.http_config.max_request_body_size, Some(1024));
        }

        #[test]
        fn test_with_retry_policy() {
            use super::super::HttpExporterBuilder;
            use crate::RetryPolicy;
            use crate::WithHttpConfig;

            let custom_policy = RetryPolicy::default()
                .with_max_retries(5)
                .with_initial_delay(Duration::from_millis(200))
                .with_max_delay(Duration::from_millis(3200))
                .with_max_jitter(Duration::from_millis(50));

            let builder = HttpExporterBuilder::default().with_retry_policy(custom_policy);

            // Verify the retry policy was set
            let retry_policy = builder.http_config.retry_policy.as_ref().unwrap();
            assert_eq!(retry_policy.max_retries, 5);
            assert_eq!(retry_policy.initial_delay, Duration::from_millis(200));
            assert_eq!(retry_policy.max_delay, Duration::from_millis(3200));
            assert_eq!(retry_policy.max_jitter, Duration::from_millis(50));
        }

        #[cfg(feature = "http-proto")]
        #[test]
        fn test_default_retry_policy_when_none_configured() {
            let client = create_test_client(crate::Protocol::HttpBinary, None);

            // Verify the recommended default values are used.
            assert_eq!(client.retry_policy.max_retries, 3);
            assert_eq!(
                client.retry_policy.initial_delay,
                Duration::from_millis(100)
            );
            assert_eq!(client.retry_policy.max_delay, Duration::from_millis(1600));
            assert_eq!(client.retry_policy.max_jitter, Duration::from_millis(100));
        }

        #[cfg(feature = "http-proto")]
        #[test]
        fn test_custom_retry_policy_used() {
            use crate::RetryPolicy;

            let custom_policy = RetryPolicy::default()
                .with_max_retries(7)
                .with_initial_delay(Duration::from_millis(500))
                .with_max_delay(Duration::from_millis(5000))
                .with_max_jitter(Duration::from_millis(200));

            let client = OtlpHttpClient::new(
                std::sync::Arc::new(MockHttpClient),
                "http://localhost:4318".parse().unwrap(),
                HashMap::new(),
                crate::Protocol::HttpBinary,
                std::time::Duration::from_secs(10),
                None,
                Some(custom_policy),
            );

            // Verify custom values are used
            assert_eq!(client.retry_policy.max_retries, 7);
            assert_eq!(
                client.retry_policy.initial_delay,
                Duration::from_millis(500)
            );
            assert_eq!(client.retry_policy.max_delay, Duration::from_millis(5000));
            assert_eq!(client.retry_policy.max_jitter, Duration::from_millis(200));
        }
    }

    /// Integration tests verifying retry behavior end-to-end through the HTTP exporter.
    /// These test the full path: HttpClient response -> HttpExportError -> classification -> retry loop.
    #[cfg(feature = "http-proto")]
    mod retry_integration_tests {
        use super::super::OtlpHttpClient;
        use crate::RetryPolicy;
        use opentelemetry_http::{Bytes, HttpClient};
        use std::collections::HashMap;
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;
        use std::time::Duration;

        /// Mock HTTP client that returns a sequence of responses controlled by an attempt counter.
        #[derive(Debug)]
        struct SequencedMockClient {
            attempts: AtomicUsize,
            responses: Vec<http::Response<Bytes>>,
        }

        impl SequencedMockClient {
            fn new(responses: Vec<http::Response<Bytes>>) -> Self {
                assert!(
                    !responses.is_empty(),
                    "SequencedMockClient requires at least one response",
                );
                Self {
                    attempts: AtomicUsize::new(0),
                    responses,
                }
            }

            fn attempt_count(&self) -> usize {
                self.attempts.load(Ordering::SeqCst)
            }
        }

        #[async_trait::async_trait]
        impl HttpClient for SequencedMockClient {
            async fn send_bytes(
                &self,
                _request: http::Request<Bytes>,
            ) -> Result<http::Response<Bytes>, opentelemetry_http::HttpError> {
                let attempt = self.attempts.fetch_add(1, Ordering::SeqCst);
                let idx = attempt.min(self.responses.len() - 1);
                let resp = &self.responses[idx];
                // Clone the response
                let mut builder = http::Response::builder().status(resp.status());
                for (k, v) in resp.headers() {
                    builder = builder.header(k, v);
                }
                Ok(builder.body(resp.body().clone()).unwrap())
            }
        }

        /// Mock that returns Err (simulating network failure) for the first N attempts,
        /// then Ok(200).
        #[derive(Debug)]
        struct NetworkFailureMockClient {
            attempts: AtomicUsize,
            fail_count: usize,
        }

        impl NetworkFailureMockClient {
            fn new(fail_count: usize) -> Self {
                Self {
                    attempts: AtomicUsize::new(0),
                    fail_count,
                }
            }

            fn attempt_count(&self) -> usize {
                self.attempts.load(Ordering::SeqCst)
            }
        }

        #[async_trait::async_trait]
        impl HttpClient for NetworkFailureMockClient {
            async fn send_bytes(
                &self,
                _request: http::Request<Bytes>,
            ) -> Result<http::Response<Bytes>, opentelemetry_http::HttpError> {
                let attempt = self.attempts.fetch_add(1, Ordering::SeqCst);
                if attempt < self.fail_count {
                    Err("connection refused".into())
                } else {
                    Ok(http::Response::builder()
                        .status(200)
                        .body(Bytes::new())
                        .unwrap())
                }
            }
        }

        #[derive(Debug, Default)]
        struct OversizedResponseMockClient {
            attempts: AtomicUsize,
        }

        impl OversizedResponseMockClient {
            fn attempt_count(&self) -> usize {
                self.attempts.load(Ordering::SeqCst)
            }
        }

        #[async_trait::async_trait]
        impl HttpClient for OversizedResponseMockClient {
            async fn send_bytes(
                &self,
                _request: http::Request<Bytes>,
            ) -> Result<http::Response<Bytes>, opentelemetry_http::HttpError> {
                self.attempts.fetch_add(1, Ordering::SeqCst);
                Err(Box::new(opentelemetry_http::ResponseBodyTooLarge))
            }
        }

        fn build_test_body(
            _client: &OtlpHttpClient,
            _data: (),
        ) -> Result<(Vec<u8>, &'static str, Option<&'static str>), String> {
            Ok((vec![1, 2, 3], "application/x-protobuf", None))
        }

        fn build_processed_test_body(
            client: &OtlpHttpClient,
            _data: (),
        ) -> Result<(Vec<u8>, &'static str, Option<&'static str>), String> {
            let (body, content_encoding) = client.process_body(vec![1, 2, 3])?;
            Ok((body, "application/x-protobuf", content_encoding))
        }

        fn make_client(mock: Arc<dyn HttpClient>, retry_policy: RetryPolicy) -> OtlpHttpClient {
            OtlpHttpClient::new(
                mock,
                "http://localhost:4318/v1/traces".parse().unwrap(),
                HashMap::new(),
                crate::Protocol::HttpBinary,
                std::time::Duration::from_secs(5),
                None,
                Some(retry_policy),
            )
        }

        fn retry_policy() -> RetryPolicy {
            RetryPolicy::default()
                .with_max_retries(3)
                .with_initial_delay(Duration::from_millis(1))
                .with_max_delay(Duration::from_millis(10))
                .with_max_jitter(Duration::ZERO)
        }

        #[test]
        fn oversized_request_is_not_sent_or_retried() {
            let mock = Arc::new(SequencedMockClient::new(vec![http::Response::builder()
                .status(200)
                .body(Bytes::new())
                .unwrap()]));
            let mut client = make_client(mock.clone(), retry_policy());
            client.max_request_body_size = 2;

            let result = futures_executor::block_on(client.export_http_with_retry(
                (),
                build_processed_test_body,
                "test",
            ));

            let error = result.unwrap_err().to_string();
            assert!(error.contains("request will not be sent"));
            assert_eq!(mock.attempt_count(), 0);
        }

        #[test]
        fn retries_on_503_then_succeeds() {
            let mock = Arc::new(SequencedMockClient::new(vec![
                http::Response::builder()
                    .status(503)
                    .body(Bytes::new())
                    .unwrap(),
                http::Response::builder()
                    .status(200)
                    .body(Bytes::new())
                    .unwrap(),
            ]));
            let client = make_client(mock.clone(), retry_policy());

            let result = futures_executor::block_on(client.export_http_with_retry(
                (),
                build_test_body,
                "test",
            ));

            assert!(result.is_ok());
            assert_eq!(mock.attempt_count(), 2);
        }

        #[test]
        fn does_not_retry_on_400() {
            let mock = Arc::new(SequencedMockClient::new(vec![http::Response::builder()
                .status(400)
                .body(Bytes::new())
                .unwrap()]));
            let client = make_client(mock.clone(), retry_policy());

            let result = futures_executor::block_on(client.export_http_with_retry(
                (),
                build_test_body,
                "test",
            ));

            assert!(result.is_err());
            assert_eq!(mock.attempt_count(), 1);
        }

        #[test]
        fn does_not_retry_when_response_body_is_too_large() {
            let mock = Arc::new(OversizedResponseMockClient::default());
            let client = make_client(mock.clone(), retry_policy());

            let error = futures_executor::block_on(client.export_http_with_retry(
                (),
                build_test_body,
                "test",
            ))
            .unwrap_err();

            assert_eq!(mock.attempt_count(), 1);
            assert!(error
                .to_string()
                .contains("response body exceeded maximum allowed 4 MiB limit"));
        }

        #[test]
        fn does_not_retry_on_unlisted_server_error() {
            let mock = Arc::new(SequencedMockClient::new(vec![http::Response::builder()
                .status(500)
                .body(Bytes::new())
                .unwrap()]));
            let client = make_client(mock.clone(), retry_policy());

            let result = futures_executor::block_on(client.export_http_with_retry(
                (),
                build_test_body,
                "test",
            ));

            assert!(result.is_err());
            assert_eq!(mock.attempt_count(), 1);
        }

        #[test]
        fn retries_on_429_with_retry_after() {
            let mock = Arc::new(SequencedMockClient::new(vec![
                http::Response::builder()
                    .status(429)
                    .header("retry-after", "1")
                    .body(Bytes::new())
                    .unwrap(),
                http::Response::builder()
                    .status(200)
                    .body(Bytes::new())
                    .unwrap(),
            ]));
            let client = make_client(mock.clone(), retry_policy());

            let start = std::time::Instant::now();
            let result = futures_executor::block_on(client.export_http_with_retry(
                (),
                build_test_body,
                "test",
            ));

            assert!(result.is_ok());
            assert_eq!(mock.attempt_count(), 2);
            // Should have honored the 1s Retry-After
            assert!(start.elapsed() >= std::time::Duration::from_millis(900));
        }

        #[test]
        fn retries_on_503_with_retry_after() {
            let mock = Arc::new(SequencedMockClient::new(vec![
                http::Response::builder()
                    .status(503)
                    .header("retry-after", "1")
                    .body(Bytes::new())
                    .unwrap(),
                http::Response::builder()
                    .status(200)
                    .body(Bytes::new())
                    .unwrap(),
            ]));
            let client = make_client(mock.clone(), retry_policy());

            let start = std::time::Instant::now();
            let result = futures_executor::block_on(client.export_http_with_retry(
                (),
                build_test_body,
                "test",
            ));

            assert!(result.is_ok());
            assert_eq!(mock.attempt_count(), 2);
            assert!(start.elapsed() >= std::time::Duration::from_millis(900));
        }

        #[test]
        fn retries_on_network_error() {
            let mock = Arc::new(NetworkFailureMockClient::new(2));
            let client = make_client(mock.clone(), retry_policy());

            let result = futures_executor::block_on(client.export_http_with_retry(
                (),
                build_test_body,
                "test",
            ));

            assert!(result.is_ok());
            assert_eq!(mock.attempt_count(), 3);
        }
    }
}
