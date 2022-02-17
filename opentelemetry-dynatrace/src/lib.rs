//! The OpenTelemetry Dynatrace Exporter supports exporting metric data to [Dynatrace].
//!
//! This exporter only supports the ingestion of metric data using the [Dynatrace Metrics ingestion protocol].
//! For trace data, use [`opentelemetry-otlp`] as described in the [Dynatrace documentation for Rust].
//!
//! # Quickstart
//!
//! You can start a new Dynatrace metrics pipeline by using [`DynatracePipelineBuilder::metrics()`].
//!
//! ```no_run
//! use opentelemetry::sdk::metrics::{selectors, PushController};
//! use opentelemetry::sdk::util::tokio_interval_stream;
//! use opentelemetry_dynatrace::ExportConfig;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
//!     // First, create a Dynatrace exporter builder. This is a minimal example. The exporter
//!     // will try to connect to the local OneAgent by default, if no endpoint is set.
//!     let dynatrace_exporter = opentelemetry_dynatrace::new_exporter();
//!
//!     // Then pass the exporter into pipeline builder
//!     let meter = opentelemetry_dynatrace::new_pipeline()
//!         .metrics(tokio::spawn, tokio_interval_stream)
//!         .with_exporter(dynatrace_exporter)
//!         .build();
//!
//!     Ok(())
//! }
//! ```
//!
//! # Kitchen Sink Full Configuration
//!
//! Example showing how to override all configuration options.
//!
//! Generally there are two parts of configuration. One part is metrics configuration.
//! Users can set metrics configuration using [`DynatraceMetricsPipeline`]. The other part is the
//! exporter configuration. Users can set the exporter configuration using [`ExportConfig`].
//!
//! ```
//! # #[cfg(feature = "reqwest-client")] {
//! use opentelemetry::sdk::metrics::{selectors, PushController};
//! use opentelemetry::sdk::util::tokio_interval_stream;
//! use opentelemetry::KeyValue;
//! use opentelemetry_dynatrace::transform::DimensionSet;
//! use opentelemetry_dynatrace::ExportConfig;
//! use std::collections::HashMap;
//! use std::time::Duration;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
//!     let export_config = ExportConfig {
//!         endpoint: Some("https://example.live.dynatrace.com/api/v2/metrics/ingest".to_string()),
//!         token: Some("*****".to_string())
//!     };
//!
//!     let meter = opentelemetry_dynatrace::new_pipeline()
//!         .metrics(tokio::spawn, tokio_interval_stream)
//!         .with_exporter(
//!             opentelemetry_dynatrace::new_exporter()
//!                 .with_export_config(
//!                     export_config
//!                         // The export config can also be set by using the with_* functions
//!                         .with_endpoint("https://example.live.dynatrace.com/api/v2/metrics/ingest")
//!                         .with_token("*****".to_string())
//!                 )
//!                 .with_headers(HashMap::from([
//!                     (http::header::USER_AGENT.to_string(), "custom-ua-string".to_string()),
//!                 ]))
//!         )
//!         // Send metric data in batches every 3 seconds
//!         .with_period(Duration::from_secs(3))
//!         .with_timeout(Duration::from_secs(10))
//!         //Prefix all metric data keys with a custom prefix
//!         .with_prefix("quickstart".to_string())
//!         // Key value pairs that will be added to all metric data
//!         .with_default_dimensions(DimensionSet::from(vec![
//!             KeyValue::new("version", env!("CARGO_PKG_VERSION")),
//!         ]))
//!         .with_aggregator_selector(selectors::simple::Selector::Exact)
//!         .build();
//!
//!     Ok(())
//! }
//! # }
//! ```
//! [Dynatrace]: https://www.dynatrace.com/
//! [Dynatrace Metrics ingestion protocol]: https://www.dynatrace.com/support/help/how-to-use-dynatrace/metrics/metric-ingestion/metric-ingestion-protocol/
//! [Dynatrace documentation for Rust]: https://www.dynatrace.com/support/help/extend-dynatrace/opentelemetry/opentelemetry-ingest/opent-rust/
//! [`opentelemetry-otlp`]: https://crates.io/crates/opentelemetry-otlp
#![warn(
    future_incompatible,
    missing_debug_implementations,
    missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    unreachable_pub,
    unused
)]
#![allow(elided_lifetimes_in_paths)]
#![cfg_attr(docsrs, feature(doc_cfg), deny(rustdoc::broken_intra_doc_links))]
#![cfg_attr(test, deny(warnings))]
mod exporter;

#[cfg(feature = "metrics")]
mod metric;

pub mod transform;
pub use crate::exporter::ExportConfig;

#[cfg(feature = "metrics")]
pub use crate::metric::{DynatraceMetricsPipeline, MetricsExporter};

use opentelemetry::sdk::export::ExportError;
use opentelemetry_http::HttpClient;
use std::collections::HashMap;

/// Dynatrace pipeline builder.
#[derive(Debug)]
pub struct DynatracePipelineBuilder;

/// Configuration of the http transport.
#[derive(Debug)]
#[cfg_attr(
    all(
        not(feature = "reqwest-blocking-client"),
        not(feature = "reqwest-client"),
        not(feature = "surf-client"),
        not(feature = "isahc-client"),
    ),
    derive(Default)
)]
pub struct HttpConfig {
    /// Default http client to be used for outbound requests.
    pub client: Option<Box<dyn HttpClient>>,

    /// Additional http headers to be set when communicating with the outbound endpoint.
    pub headers: Option<HashMap<String, String>>,
}

#[cfg(any(
    feature = "reqwest-blocking-client",
    feature = "reqwest-client",
    feature = "surf-client",
    feature = "isahc-client",
))]
impl Default for HttpConfig {
    fn default() -> Self {
        HttpConfig {
            #[cfg(feature = "reqwest-client")]
            client: Some(Box::new(reqwest::Client::new())),
            #[cfg(all(
                not(feature = "reqwest-client"),
                not(feature = "surf-client"),
                not(feature = "isahc-client"),
                feature = "reqwest-blocking-client"
            ))]
            client: Some(Box::new(reqwest::blocking::Client::new())),
            #[cfg(all(
                not(feature = "reqwest-client"),
                not(feature = "reqwest-blocking-client"),
                not(feature = "isahc-client"),
                feature = "surf-client"
            ))]
            client: Some(Box::new(surf::Client::new())),
            #[cfg(all(
                not(feature = "reqwest-client"),
                not(feature = "reqwest-blocking-client"),
                not(feature = "surf-client"),
                feature = "isahc-client"
            ))]
            client: Some(Box::new(isahc::HttpClient::new().unwrap())),
            #[cfg(all(
                not(feature = "reqwest-client"),
                not(feature = "reqwest-blocking-client"),
                not(feature = "surf-client"),
                not(feature = "isahc-client")
            ))]
            client: None,
            headers: None,
        }
    }
}

/// Dynatrace exporter builder.
#[derive(Debug)]
pub struct DynatraceExporterBuilder {
    pub(crate) export_config: ExportConfig,
    pub(crate) http_config: HttpConfig,
}

impl Default for DynatraceExporterBuilder {
    fn default() -> Self {
        DynatraceExporterBuilder {
            http_config: HttpConfig::default(),
            export_config: ExportConfig {
                ..ExportConfig::default()
            },
        }
    }
}

impl DynatraceExporterBuilder {
    /// Set the http client to be used for outbound requests.
    pub fn with_http_client<T: HttpClient + 'static>(mut self, client: T) -> Self {
        self.http_config.client = Some(Box::new(client));
        self
    }

    /// Set additional http headers to to be sent when communicating with the outbound endpoint.
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.http_config.headers = Some(headers);
        self
    }

    /// Set the export config. This will override all previous configuration.
    pub fn with_export_config(mut self, export_config: ExportConfig) -> Self {
        self.export_config = export_config;
        self
    }
}

/// Create a new pipeline builder with the default configuration.
///
/// ## Examples
///
/// ```no_run
/// use opentelemetry::sdk::util::tokio_interval_stream;
/// # fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
/// let meter = opentelemetry_dynatrace::new_pipeline()
///     .metrics(tokio::spawn, tokio_interval_stream);
/// # Ok(())
/// # }
/// ```
pub fn new_pipeline() -> DynatracePipelineBuilder {
    DynatracePipelineBuilder
}

/// Create a new `DynatraceExporterBuilder` with the default configuration.
///
/// ## Examples
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
/// let dynatrace_exporter = opentelemetry_dynatrace::new_exporter();
/// # Ok(())
/// }
/// ```
pub fn new_exporter() -> DynatraceExporterBuilder {
    DynatraceExporterBuilder::default()
}

/// Wrap type for errors from this crate.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// The pipeline will need a exporter to complete setup. Throw this error if none is provided.
    #[error("no exporter builder is provided, please provide one using with_exporter() method")]
    NoExporterBuilder,

    /// Invalid URI.
    #[error("invalid URI {0}")]
    InvalidUri(#[from] http::uri::InvalidUri),

    /// Http requests failed because no http client is provided.
    #[error(
        "no http client, you must select one from features or provide your own implementation"
    )]
    NoHttpClient,

    /// Http requests failed.
    #[error("http request failed with {0}")]
    RequestFailed(#[from] http::Error),

    /// The provided value is invalid in http headers.
    #[error("http header value error {0}")]
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),

    /// The provided name is invalid in http headers.
    #[error("http header name error {0}")]
    InvalidHeaderName(#[from] http::header::InvalidHeaderName),

    /// The lock in exporters has been poisoned.
    #[cfg(feature = "metrics")]
    #[error("the lock of the {0} has been poisoned")]
    PoisonedLock(&'static str),
}

impl ExportError for Error {
    fn exporter_name(&self) -> &'static str {
        "dynatrace"
    }
}
