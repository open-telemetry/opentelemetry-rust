use super::{
    default_headers, default_protocol, parse_header_string,
    OTEL_EXPORTER_OTLP_HTTP_ENDPOINT_DEFAULT,
};
use crate::{
    ExportConfig, Protocol, OTEL_EXPORTER_OTLP_ENDPOINT, OTEL_EXPORTER_OTLP_HEADERS,
    OTEL_EXPORTER_OTLP_TIMEOUT,
};
use http::{HeaderName, HeaderValue, Uri};
use opentelemetry_http::HttpClient;
use opentelemetry_proto::transform::common::tonic::ResourceAttributesWithSchema;
#[cfg(feature = "logs")]
use opentelemetry_proto::transform::logs::tonic::group_logs_by_resource_and_scope;
#[cfg(feature = "trace")]
use opentelemetry_proto::transform::trace::tonic::group_spans_by_resource_and_scope;
#[cfg(feature = "logs")]
use opentelemetry_sdk::logs::LogBatch;
#[cfg(feature = "trace")]
use opentelemetry_sdk::trace::SpanData;
use prost::Message;
use std::collections::HashMap;
use std::env;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

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
#[derive(Debug)]
#[cfg_attr(
    all(
        not(feature = "reqwest-client"),
        not(feature = "reqwest-blocking-client"),
        not(feature = "hyper-client")
    ),
    derive(Default)
)]
pub struct HttpConfig {
    /// Select the HTTP client
    client: Option<Arc<dyn HttpClient>>,

    /// Additional headers to send to the collector.
    headers: Option<HashMap<String, String>>,
}

#[cfg(any(
    feature = "reqwest-blocking-client",
    feature = "reqwest-client",
    feature = "hyper-client"
))]
impl Default for HttpConfig {
    fn default() -> Self {
        #[cfg(feature = "reqwest-blocking-client")]
        let default_client = std::thread::spawn(|| {
            Some(Arc::new(reqwest::blocking::Client::new()) as Arc<dyn HttpClient>)
        })
        .join()
        .expect("creating reqwest::blocking::Client on a new thread not to fail");
        #[cfg(all(not(feature = "reqwest-blocking-client"), feature = "reqwest-client"))]
        let default_client = Some(Arc::new(reqwest::Client::new()) as Arc<dyn HttpClient>);
        #[cfg(all(
            not(feature = "reqwest-client"),
            not(feature = "reqwest-blocking-client"),
            feature = "hyper-client"
        ))]
        // TODO - support configuring custom connector and executor
        let default_client = Some(Arc::new(HyperClient::with_default_connector(
            Duration::from_secs(10),
            None,
        )) as Arc<dyn HttpClient>);
        #[cfg(all(
            not(feature = "reqwest-client"),
            not(feature = "reqwest-blocking-client"),
            not(feature = "hyper-client")
        ))]
        let default_client = None;
        HttpConfig {
            client: default_client,
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
/// use opentelemetry_sdk::metrics::Temporality;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a span exporter you can use to when configuring tracer providers
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
    fn build_client(
        &mut self,
        signal_endpoint_var: &str,
        signal_endpoint_path: &str,
        signal_timeout_var: &str,
        signal_http_headers_var: &str,
    ) -> Result<OtlpHttpClient, crate::Error> {
        let endpoint = resolve_http_endpoint(
            signal_endpoint_var,
            signal_endpoint_path,
            self.exporter_config.endpoint.clone(),
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
    pub fn build_log_exporter(mut self) -> opentelemetry_sdk::logs::LogResult<crate::LogExporter> {
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

        Ok(crate::LogExporter::from_http(client))
    }

    /// Create a metrics exporter with the current configuration
    #[cfg(feature = "metrics")]
    pub fn build_metrics_exporter(
        mut self,
        temporality: opentelemetry_sdk::metrics::Temporality,
    ) -> opentelemetry_sdk::metrics::MetricResult<crate::MetricExporter> {
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

        Ok(crate::MetricExporter::new(client, temporality))
    }
}

#[derive(Debug)]
pub(crate) struct OtlpHttpClient {
    client: Mutex<Option<Arc<dyn HttpClient>>>,
    collector_endpoint: Uri,
    headers: HashMap<HeaderName, HeaderValue>,
    protocol: Protocol,
    _timeout: Duration,
    #[allow(dead_code)]
    // <allow dead> would be removed once we support set_resource for metrics and traces.
    resource: opentelemetry_proto::transform::common::tonic::ResourceAttributesWithSchema,
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
            resource: ResourceAttributesWithSchema::default(),
        }
    }

    #[cfg(feature = "trace")]
    fn build_trace_export_body(
        &self,
        spans: Vec<SpanData>,
    ) -> opentelemetry::trace::TraceResult<(Vec<u8>, &'static str)> {
        use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
        let resource_spans = group_spans_by_resource_and_scope(spans, &self.resource);

        let req = ExportTraceServiceRequest { resource_spans };
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
        logs: LogBatch<'_>,
    ) -> opentelemetry_sdk::logs::LogResult<(Vec<u8>, &'static str)> {
        use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
        let resource_logs = group_logs_by_resource_and_scope(logs, &self.resource);
        let req = ExportLogsServiceRequest { resource_logs };

        match self.protocol {
            #[cfg(feature = "http-json")]
            Protocol::HttpJson => match serde_json::to_string_pretty(&req) {
                Ok(json) => Ok((json.into(), "application/json")),
                Err(e) => Err(opentelemetry_sdk::logs::LogError::from(e.to_string())),
            },
            _ => Ok((req.encode_to_vec(), "application/x-protobuf")),
        }
    }

    #[cfg(feature = "metrics")]
    fn build_metrics_export_body(
        &self,
        metrics: &mut ResourceMetrics,
    ) -> opentelemetry_sdk::metrics::MetricResult<(Vec<u8>, &'static str)> {
        use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;

        let req: ExportMetricsServiceRequest = (&*metrics).into();

        match self.protocol {
            #[cfg(feature = "http-json")]
            Protocol::HttpJson => match serde_json::to_string_pretty(&req) {
                Ok(json) => Ok((json.into(), "application/json")),
                Err(e) => Err(opentelemetry_sdk::metrics::MetricError::Other(
                    e.to_string(),
                )),
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
fn resolve_http_endpoint(
    signal_endpoint_var: &str,
    signal_endpoint_path: &str,
    provided_endpoint: Option<String>,
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

    provided_endpoint
        .map(|e| e.parse().map_err(From::from))
        .unwrap_or_else(|| {
            build_endpoint_uri(
                OTEL_EXPORTER_OTLP_HTTP_ENDPOINT_DEFAULT,
                signal_endpoint_path,
            )
        })
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
pub trait HasHttpConfig {
    /// Return a mutable reference to the config within the exporter builders.
    fn http_client_config(&mut self) -> &mut HttpConfig;
}

/// Expose interface for modifying builder config.
impl HasHttpConfig for HttpExporterBuilder {
    fn http_client_config(&mut self) -> &mut HttpConfig {
        &mut self.http_config
    }
}

/// This trait will be implemented for every struct that implemented [`HasHttpConfig`] trait.
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
}

impl<B: HasHttpConfig> WithHttpConfig for B {
    fn with_http_client<T: HttpClient + 'static>(mut self, client: T) -> Self {
        self.http_client_config().client = Some(Arc::new(client));
        self
    }

    fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        // headers will be wrapped, so we must do some logic to unwrap first.
        self.http_client_config()
            .headers
            .iter_mut()
            .zip(headers)
            .for_each(|(http_client_headers, (key, value))| {
                http_client_headers.insert(key, super::url_decode(&value).unwrap_or(value));
            });
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
                let endpoint = resolve_http_endpoint(
                    OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
                    "/v1/traces",
                    Some("http://localhost:4317".to_string()),
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
                let endpoint = super::resolve_http_endpoint(
                    OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
                    "/v1/traces",
                    Some("http://localhost:4317".to_string()),
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
                let endpoint = super::resolve_http_endpoint(
                    OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
                    "/v1/traces",
                    Some("http://localhost:4317".to_string()),
                )
                .unwrap();
                assert_eq!(endpoint, "http://example.com");
            },
        );
    }

    #[test]
    fn test_use_provided_or_default_when_others_missing() {
        run_env_test(vec![], || {
            let endpoint = super::resolve_http_endpoint(
                "NON_EXISTENT_VAR",
                "/v1/traces",
                Some("http://localhost:4317".to_string()),
            )
            .unwrap();
            assert_eq!(endpoint, "http://localhost:4317/");
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
                let endpoint = super::resolve_http_endpoint(
                    OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
                    "/v1/traces",
                    Some("http://localhost:4317".to_string()),
                )
                .unwrap();
                assert_eq!(endpoint, "http://example.com/v1/traces");
            },
        );
    }

    #[test]
    fn test_all_invalid_urls_falls_back_to_error() {
        run_env_test(vec![], || {
            let result = super::resolve_http_endpoint(
                OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
                "/v1/traces",
                Some("-*/*-/*-//-/-/yet-another-invalid-uri".to_string()),
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

    #[test]
    fn test_http_exporter_builder_with_header() {
        use std::collections::HashMap;
        // Arrange
        let initial_headers = HashMap::from([("k1".to_string(), "v1".to_string())]);
        let extra_headers = HashMap::from([("k2".to_string(), "v2".to_string())]);
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
            },
            exporter_config: crate::ExportConfig::default(),
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
                exporter.exporter_config.endpoint,
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
                exporter.exporter_config.endpoint,
            )
            .unwrap();

            assert_eq!(url, "http://localhost:4318/v1/tracesbutnotreally");
        });
    }
}
