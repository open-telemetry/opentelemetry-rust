use crate::{ExportConfig, Protocol, OTEL_EXPORTER_OTLP_ENDPOINT, OTEL_EXPORTER_OTLP_TIMEOUT};
use http::{HeaderName, HeaderValue, Uri};
use opentelemetry_http::HttpClient;
use std::collections::HashMap;
use std::env;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::default_headers;

#[cfg(feature = "metrics")]
mod metrics;

#[cfg(feature = "logs")]
mod logs;

#[cfg(feature = "trace")]
mod trace;

/// Configuration of the http transport
#[cfg(feature = "http-proto")]
#[derive(Debug)]
#[cfg_attr(
    all(
        not(feature = "reqwest-client"),
        not(feature = "surf-client"),
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

#[cfg(any(
    feature = "reqwest-blocking-client",
    feature = "reqwest-client",
    feature = "surf-client"
))]
impl Default for HttpConfig {
    fn default() -> Self {
        HttpConfig {
            #[cfg(feature = "reqwest-blocking-client")]
            client: Some(Arc::new(reqwest::blocking::Client::new())),
            #[cfg(all(
                not(feature = "reqwest-blocking-client"),
                not(feature = "surf-client"),
                feature = "reqwest-client"
            ))]
            client: Some(Arc::new(reqwest::Client::new())),
            #[cfg(all(
                not(feature = "reqwest-client"),
                not(feature = "reqwest-blocking-client"),
                feature = "surf-client"
            ))]
            client: Some(Arc::new(surf::Client::new())),
            #[cfg(all(
                not(feature = "reqwest-client"),
                not(feature = "surf-client"),
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
                protocol: Protocol::HttpBinary,
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
    /// Assign client implementation
    pub fn with_http_client<T: HttpClient + 'static>(mut self, client: T) -> Self {
        self.http_config.client = Some(Arc::new(client));
        self
    }

    /// Set additional headers to send to the collector.
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        // headers will be wrapped, so we must do some logic to unwrap first.
        let mut inst_headers = self.http_config.headers.unwrap_or_default();
        inst_headers.extend(headers);
        self.http_config.headers = Some(inst_headers);
        self
    }

    fn build_client(
        &mut self,
        signal_endpoint_var: &str,
        signal_endpoint_path: &str,
        signal_timeout_var: &str,
    ) -> Result<OtlpHttpClient, crate::Error> {
        let endpoint = match env::var(signal_endpoint_var)
            .ok()
            .or(env::var(OTEL_EXPORTER_OTLP_ENDPOINT).ok())
            .and_then(|s| s.parse().ok())
        {
            Some(val) => val,
            None => format!("{}{signal_endpoint_path}", self.exporter_config.endpoint).parse()?,
        };

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
        let headers = self
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

        Ok(OtlpHttpClient::new(http_client, endpoint, headers, timeout))
    }

    /// Create a log exporter with the current configuration
    #[cfg(feature = "trace")]
    pub fn build_span_exporter(
        mut self,
    ) -> Result<crate::SpanExporter, opentelemetry_api::trace::TraceError> {
        use crate::{OTEL_EXPORTER_OTLP_TRACES_ENDPOINT, OTEL_EXPORTER_OTLP_TRACES_TIMEOUT};

        let client = self.build_client(
            OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
            "/v1/traces",
            OTEL_EXPORTER_OTLP_TRACES_TIMEOUT,
        )?;

        Ok(crate::SpanExporter::new(client))
    }

    /// Create a log exporter with the current configuration
    #[cfg(feature = "logs")]
    pub fn build_log_exporter(mut self) -> opentelemetry_api::logs::LogResult<crate::LogExporter> {
        use crate::{OTEL_EXPORTER_OTLP_LOGS_ENDPOINT, OTEL_EXPORTER_OTLP_LOGS_TIMEOUT};

        let client = self.build_client(
            OTEL_EXPORTER_OTLP_LOGS_ENDPOINT,
            "/v1/logs",
            OTEL_EXPORTER_OTLP_LOGS_TIMEOUT,
        )?;

        Ok(crate::LogExporter::new(client))
    }

    /// Create a metrics exporter with the current configuration
    #[cfg(feature = "metrics")]
    pub fn build_metrics_exporter(
        mut self,
        aggregation_selector: Box<dyn opentelemetry_sdk::metrics::reader::AggregationSelector>,
        temporality_selector: Box<dyn opentelemetry_sdk::metrics::reader::TemporalitySelector>,
    ) -> opentelemetry_api::metrics::Result<crate::MetricsExporter> {
        use crate::{OTEL_EXPORTER_OTLP_METRICS_ENDPOINT, OTEL_EXPORTER_OTLP_METRICS_TIMEOUT};

        let client = self.build_client(
            OTEL_EXPORTER_OTLP_METRICS_ENDPOINT,
            "/v1/metrics",
            OTEL_EXPORTER_OTLP_METRICS_TIMEOUT,
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
    _timeout: Duration,
}

impl OtlpHttpClient {
    #[allow(clippy::mutable_key_type)] // http headers are not mutated
    fn new(
        client: Arc<dyn HttpClient>,
        collector_endpoint: Uri,
        headers: HashMap<HeaderName, HeaderValue>,
        timeout: Duration,
    ) -> Self {
        OtlpHttpClient {
            client: Mutex::new(Some(client)),
            collector_endpoint,
            headers,
            _timeout: timeout,
        }
    }
}
