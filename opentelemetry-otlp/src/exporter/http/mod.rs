use super::{default_headers, default_protocol, parse_header_string};
use crate::{
    ExportConfig, Protocol, OTEL_EXPORTER_OTLP_ENDPOINT, OTEL_EXPORTER_OTLP_HEADERS,
    OTEL_EXPORTER_OTLP_TIMEOUT,
};
use http::{HeaderName, HeaderValue, Uri};
use opentelemetry_http::HttpClient;
#[cfg(feature = "logs")]
use opentelemetry_sdk::export::logs::LogData;
#[cfg(feature = "trace")]
use opentelemetry_sdk::export::trace::SpanData;
#[cfg(feature = "metrics")]
use opentelemetry_sdk::metrics::data::ResourceMetrics;
use prost::Message;
use std::collections::HashMap;
use std::env;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[cfg(feature = "metrics")]
mod metrics;

#[cfg(feature = "logs")]
mod logs;

#[cfg(feature = "trace")]
mod trace;

/// Configuration of the http transport
#[cfg(any(feature = "http-proto", feature = "http-json"))]
#[derive(Debug)]
#[cfg_attr(
    all(
        not(feature = "reqwest-client"),
        not(feature = "reqwest-blocking-client")
    ),
    derive(Default)
)]
pub(crate) struct HttpConfig {
    /// Select the HTTP client
    client: Option<Arc<dyn HttpClient>>,

    /// Additional headers to send to the collector.
    headers: Option<HashMap<String, String>>,
}

#[cfg(any(feature = "reqwest-blocking-client", feature = "reqwest-client",))]
impl Default for HttpConfig {
    fn default() -> Self {
        HttpConfig {
            #[cfg(feature = "reqwest-blocking-client")]
            client: Some(Arc::new(reqwest::blocking::Client::new())),
            #[cfg(all(not(feature = "reqwest-blocking-client"), feature = "reqwest-client"))]
            client: Some(Arc::new(reqwest::Client::new())),
            #[cfg(all(
                not(feature = "reqwest-client"),
                not(feature = "reqwest-blocking-client")
            ))]
            client: None,
            headers: None,
        }
    }
}

/// Configuration for the OTLP HTTP exporter.
///
/// ## Examples
///
/// ```
/// # #[cfg(feature="metrics")]
/// use opentelemetry_sdk::metrics::reader::{
///     DefaultAggregationSelector, DefaultTemporalitySelector,
/// };
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a span exporter you can use to when configuring tracer providers
/// # #[cfg(feature="trace")]
/// let span_exporter = opentelemetry_otlp::new_exporter().http().build_span_exporter()?;
///
/// // Create a metrics exporter you can use when configuring meter providers
/// # #[cfg(feature="metrics")]
/// let metrics_exporter = opentelemetry_otlp::new_exporter()
///     .http()
///     .build_metrics_exporter(
///         Box::new(DefaultAggregationSelector::new()),
///         Box::new(DefaultTemporalitySelector::new()),
///     )?;
///
/// // Create a log exporter you can use when configuring logger providers
/// # #[cfg(feature="logs")]
/// let log_exporter = opentelemetry_otlp::new_exporter().http().build_log_exporter()?;
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
            exporter_config: ExportConfig {
                protocol: default_protocol(),
                ..ExportConfig::default()
            },
            http_config: HttpConfig {
                headers: Some(default_headers()),
                ..HttpConfig::default()
            },
        }
    }
}

impl HttpExporterBuilder {
    /// Specify the OTLP protocol to be used by the exporter
    pub fn with_protocol(mut self, protocol: Protocol) -> Self {
        self.exporter_config.protocol = protocol;
        self
    }

    /// Assign client implementation
    pub fn with_http_client<T: HttpClient + 'static>(mut self, client: T) -> Self {
        self.http_config.client = Some(Arc::new(client));
        self
    }

    /// Set additional headers to send to the collector.
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        // headers will be wrapped, so we must do some logic to unwrap first.
        let mut inst_headers = self.http_config.headers.unwrap_or_default();
        inst_headers.extend(
            headers
                .into_iter()
                .map(|(key, value)| (key, super::url_decode(&value).unwrap_or(value))),
        );
        self.http_config.headers = Some(inst_headers);
        self
    }

    fn build_client(
        &mut self,
        signal_endpoint_var: &str,
        signal_endpoint_path: &str,
        signal_timeout_var: &str,
        signal_http_headers_var: &str,
    ) -> Result<OtlpHttpClient, crate::Error> {
        let endpoint = resolve_endpoint(
            signal_endpoint_var,
            signal_endpoint_path,
            self.exporter_config.endpoint.as_str(),
        )?;

        let timeout = match env::var(signal_timeout_var)
            .ok()
            .or(env::var(OTEL_EXPORTER_OTLP_TIMEOUT).ok())
        {
            Some(val) => match val.parse() {
                Ok(seconds) => Duration::from_secs(seconds),
                Err(_) => self.exporter_config.timeout,
            },
            None => self.exporter_config.timeout,
        };

        let http_client = self
            .http_config
            .client
            .take()
            .ok_or(crate::Error::NoHttpClient)?;

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

        Ok(OtlpHttpClient::new(
            http_client,
            endpoint,
            headers,
            self.exporter_config.protocol,
            timeout,
        ))
    }

    /// Create a log exporter with the current configuration
    #[cfg(feature = "trace")]
    pub fn build_span_exporter(
        mut self,
    ) -> Result<crate::SpanExporter, opentelemetry::trace::TraceError> {
        use crate::{
            OTEL_EXPORTER_OTLP_TRACES_ENDPOINT, OTEL_EXPORTER_OTLP_TRACES_HEADERS,
            OTEL_EXPORTER_OTLP_TRACES_TIMEOUT,
        };

        let client = self.build_client(
            OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
            "/v1/traces",
            OTEL_EXPORTER_OTLP_TRACES_TIMEOUT,
            OTEL_EXPORTER_OTLP_TRACES_HEADERS,
        )?;

        Ok(crate::SpanExporter::new(client))
    }

    /// Create a log exporter with the current configuration
    #[cfg(feature = "logs")]
    pub fn build_log_exporter(mut self) -> opentelemetry::logs::LogResult<crate::LogExporter> {
        use crate::{
            OTEL_EXPORTER_OTLP_LOGS_ENDPOINT, OTEL_EXPORTER_OTLP_LOGS_HEADERS,
            OTEL_EXPORTER_OTLP_LOGS_TIMEOUT,
        };

        let client = self.build_client(
            OTEL_EXPORTER_OTLP_LOGS_ENDPOINT,
            "/v1/logs",
            OTEL_EXPORTER_OTLP_LOGS_TIMEOUT,
            OTEL_EXPORTER_OTLP_LOGS_HEADERS,
        )?;

        Ok(crate::LogExporter::new(client))
    }

    /// Create a metrics exporter with the current configuration
    #[cfg(feature = "metrics")]
    pub fn build_metrics_exporter(
        mut self,
        aggregation_selector: Box<dyn opentelemetry_sdk::metrics::reader::AggregationSelector>,
        temporality_selector: Box<dyn opentelemetry_sdk::metrics::reader::TemporalitySelector>,
    ) -> opentelemetry::metrics::Result<crate::MetricsExporter> {
        use crate::{
            OTEL_EXPORTER_OTLP_METRICS_ENDPOINT, OTEL_EXPORTER_OTLP_METRICS_HEADERS,
            OTEL_EXPORTER_OTLP_METRICS_TIMEOUT,
        };

        let client = self.build_client(
            OTEL_EXPORTER_OTLP_METRICS_ENDPOINT,
            "/v1/metrics",
            OTEL_EXPORTER_OTLP_METRICS_TIMEOUT,
            OTEL_EXPORTER_OTLP_METRICS_HEADERS,
        )?;

        Ok(crate::MetricsExporter::new(
            client,
            temporality_selector,
            aggregation_selector,
        ))
    }
}

#[derive(Debug)]
struct OtlpHttpClient {
    client: Mutex<Option<Arc<dyn HttpClient>>>,
    collector_endpoint: Uri,
    headers: HashMap<HeaderName, HeaderValue>,
    protocol: Protocol,
    _timeout: Duration,
}

impl OtlpHttpClient {
    #[allow(clippy::mutable_key_type)] // http headers are not mutated
    fn new(
        client: Arc<dyn HttpClient>,
        collector_endpoint: Uri,
        headers: HashMap<HeaderName, HeaderValue>,
        protocol: Protocol,
        timeout: Duration,
    ) -> Self {
        OtlpHttpClient {
            client: Mutex::new(Some(client)),
            collector_endpoint,
            headers,
            protocol,
            _timeout: timeout,
        }
    }

    #[cfg(feature = "trace")]
    fn build_trace_export_body(
        &self,
        spans: Vec<SpanData>,
    ) -> opentelemetry::trace::TraceResult<(Vec<u8>, &'static str)> {
        use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;

        let req = ExportTraceServiceRequest {
            resource_spans: spans.into_iter().map(Into::into).collect(),
        };
        match self.protocol {
            #[cfg(feature = "http-json")]
            Protocol::HttpJson => match serde_json::to_string_pretty(&req) {
                Ok(json) => Ok((json.into(), "application/json")),
                Err(e) => Err(opentelemetry::trace::TraceError::from(e.to_string())),
            },
            _ => Ok((req.encode_to_vec(), "application/x-protobuf")),
        }
    }

    #[cfg(feature = "logs")]
    fn build_logs_export_body(
        &self,
        logs: Vec<LogData>,
    ) -> opentelemetry::logs::LogResult<(Vec<u8>, &'static str)> {
        use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;

        let req = ExportLogsServiceRequest {
            resource_logs: logs.into_iter().map(Into::into).collect(),
        };
        match self.protocol {
            #[cfg(feature = "http-json")]
            Protocol::HttpJson => match serde_json::to_string_pretty(&req) {
                Ok(json) => Ok((json.into(), "application/json")),
                Err(e) => Err(opentelemetry::logs::LogError::from(e.to_string())),
            },
            _ => Ok((req.encode_to_vec(), "application/x-protobuf")),
        }
    }

    #[cfg(feature = "metrics")]
    fn build_metrics_export_body(
        &self,
        metrics: &mut ResourceMetrics,
    ) -> opentelemetry::metrics::Result<(Vec<u8>, &'static str)> {
        use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;

        let req: ExportMetricsServiceRequest = (&*metrics).into();

        match self.protocol {
            #[cfg(feature = "http-json")]
            Protocol::HttpJson => match serde_json::to_string_pretty(&req) {
                Ok(json) => Ok((json.into(), "application/json")),
                Err(e) => Err(opentelemetry::metrics::MetricsError::Other(e.to_string())),
            },
            _ => Ok((req.encode_to_vec(), "application/x-protobuf")),
        }
    }
}

fn build_endpoint_uri(endpoint: &str, path: &str) -> Result<Uri, crate::Error> {
    let path = if endpoint.ends_with('/') && path.starts_with('/') {
        path.strip_prefix('/').unwrap()
    } else {
        path
    };
    format!("{endpoint}{path}").parse().map_err(From::from)
}

// see https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/protocol/exporter.md#endpoint-urls-for-otlphttp
fn resolve_endpoint(
    signal_endpoint_var: &str,
    signal_endpoint_path: &str,
    provided_or_default_endpoint: &str,
) -> Result<Uri, crate::Error> {
    // per signal env var is not modified
    if let Some(endpoint) = env::var(signal_endpoint_var)
        .ok()
        .and_then(|s| s.parse().ok())
    {
        return Ok(endpoint);
    }

    // if signal env var is not set, then we check if the OTEL_EXPORTER_OTLP_ENDPOINT is set
    if let Some(endpoint) = env::var(OTEL_EXPORTER_OTLP_ENDPOINT)
        .ok()
        .and_then(|s| build_endpoint_uri(&s, signal_endpoint_path).ok())
    {
        return Ok(endpoint);
    }

    // if neither works, we use the one provided in pipeline. If user never provides one, we will use the default one
    build_endpoint_uri(provided_or_default_endpoint, signal_endpoint_path)
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

#[cfg(test)]
mod tests {
    use crate::exporter::tests::run_env_test;
    use crate::{OTEL_EXPORTER_OTLP_ENDPOINT, OTEL_EXPORTER_OTLP_TRACES_ENDPOINT};

    use super::build_endpoint_uri;

    #[test]
    fn test_append_signal_path_to_generic_env() {
        run_env_test(
            vec![(OTEL_EXPORTER_OTLP_ENDPOINT, "http://example.com")],
            || {
                let endpoint = super::resolve_endpoint(
                    OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
                    "/v1/traces",
                    "http://localhost:4317",
                )
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
                let endpoint = super::resolve_endpoint(
                    OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
                    "/v1/traces",
                    "http://localhost:4317",
                )
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
                let endpoint = super::resolve_endpoint(
                    OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
                    "/v1/traces",
                    "http://localhost:4317",
                )
                .unwrap();
                assert_eq!(endpoint, "http://example.com");
            },
        );
    }

    #[test]
    fn test_use_provided_or_default_when_others_missing() {
        run_env_test(vec![], || {
            let endpoint =
                super::resolve_endpoint("NON_EXISTENT_VAR", "/v1/traces", "http://localhost:4317")
                    .unwrap();
            assert_eq!(endpoint, "http://localhost:4317/v1/traces");
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
    fn test_invalid_uri_in_signal_env_falls_back_to_generic_env() {
        run_env_test(
            vec![
                (
                    OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
                    "-*/*-/*-//-/-/invalid-uri",
                ),
                (OTEL_EXPORTER_OTLP_ENDPOINT, "http://example.com"),
            ],
            || {
                let endpoint = super::resolve_endpoint(
                    OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
                    "/v1/traces",
                    "http://localhost:4317",
                )
                .unwrap();
                assert_eq!(endpoint, "http://example.com/v1/traces");
            },
        );
    }

    #[test]
    fn test_all_invalid_urls_falls_back_to_error() {
        run_env_test(vec![], || {
            let result = super::resolve_endpoint(
                OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
                "/v1/traces",
                "-*/*-/*-//-/-/yet-another-invalid-uri",
            );
            assert!(result.is_err());
            // You may also want to assert on the specific error type if applicable
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
                "Failed on input: {}",
                input_str
            );

            for (expected_key, expected_value) in expected_headers {
                assert_eq!(
                    headers.get(&HeaderName::from_static(expected_key)),
                    Some(&HeaderValue::from_static(expected_value)),
                    "Failed on key: {} with input: {}",
                    expected_key,
                    input_str
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
                "Failed on input: {}",
                input_str
            );

            for (expected_key, expected_value) in expected_headers {
                assert_eq!(
                    headers.get(&HeaderName::from_static(expected_key)),
                    Some(&HeaderValue::from_static(expected_value)),
                    "Failed on key: {} with input: {}",
                    expected_key,
                    input_str
                );
            }
        }
    }
}
