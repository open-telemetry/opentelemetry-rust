//! The OTLP Exporter supports exporting trace and metric data in the OTLP
//! format to the OpenTelemetry collector or other compatible backend. The
//! OpenTelemetry Collector offers a vendor-agnostic implementation on how
//! to receive, process, and export telemetry data. In addition, it removes
//! the need to run, operate, and maintain multiple agents/collectors in
//! order to support open-source telemetry data formats (e.g. Jaeger,
//! Prometheus, etc.) sending to multiple open-source or commercial back-ends.
//!
//! Currently, this crate only support sending tracing data or metrics in OTLP
//! via grpc. Support for sending data via HTTP will be added in the future.
//!
//! ## Quickstart
//!
//! First make sure you have a running version of the opentelemetry collector
//! you want to send data to:
//!
//! ```shell
//! $ docker run -p 4317:4317 otel/opentelemetry-collector-dev:latest
//! ```
//!
//! Then install a new pipeline with the recommended defaults to start exporting
//! telemetry.
//!
//! ```no_run
//! use opentelemetry::trace::Tracer;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
//!     // use tonic as grpc layer here.
//!     // If you want to use grpcio. enable `grpc-sys` feature and use with_grpcio function here.
//!     let tracer = opentelemetry_otlp::new_pipeline().with_tonic().install_simple()?;
//!
//!     tracer.in_span("doing_work", |cx| {
//!         // Traced app logic here...
//!     });
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Performance
//!
//! For optimal performance, a batch exporter is recommended as the simple
//! exporter will export each span synchronously on dropping. You can enable the
//! [`rt-tokio`], [`rt-tokio-current-thread`] or [`rt-async-std`] features and
//! specify a runtime on the pipeline builder to have a batch exporter
//! configured for you automatically.
//!
//! ```toml
//! [dependencies]
//! opentelemetry = { version = "*", features = ["async-std"] }
//! opentelemetry-otlp = { version = "*", features = ["grpc-sys"] }
//! ```
//!
//! ```no_run
//! # fn main() -> Result<(), opentelemetry::trace::TraceError> {
//! let tracer = opentelemetry_otlp::new_pipeline()
//!     .install_batch(opentelemetry::runtime::AsyncStd)?;
//! # Ok(())
//! # }
//! ```
//!
//! [`tokio`]: https://tokio.rs
//! [`async-std`]: https://async.rs
//!
//! ## Kitchen Sink Full Configuration
//!
//! Example showing how to override all configuration options. See the
//! [`OtlpPipelineBuilder`] docs for details of each option.
//!
//! There are two types of configurations. The first is common configurations
//! that is used by both tonic and grpcio. The other is configuration that only
//! works with tonic or grpcio.
//!
//! The [`OtlpPipelineBuilder`] will first config the configurations that shared
//! by both grpc layers. Then users can choose their grpc layer by [`with_tonic`]
//! or [`with_grpcio`] functions. User can then config anything that only works
//! with specific grpc layers.
//!
//! ```no_run
//! use opentelemetry::{KeyValue, trace::Tracer};
//! use opentelemetry::sdk::{trace::{self, IdGenerator, Sampler}, Resource};
//! use opentelemetry_otlp::{Protocol};
//! use std::time::Duration;
//! use tonic::metadata::*;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
//!     let mut map = MetadataMap::with_capacity(3);
//!
//!     map.insert("x-host", "example.com".parse().unwrap());
//!     map.insert("x-number", "123".parse().unwrap());
//!     map.insert_bin("trace-proto-bin", MetadataValue::from_bytes(b"[binary data]"));
//!
//!     let tracer = opentelemetry_otlp::new_pipeline()
//!         .with_endpoint("http://localhost:4317")
//!         .with_protocol(Protocol::Grpc)
//!         .with_timeout(Duration::from_secs(3))
//!         .with_trace_config(
//!             trace::config()
//!                 .with_sampler(Sampler::AlwaysOn)
//!                 .with_id_generator(IdGenerator::default())
//!                 .with_max_events_per_span(64)
//!                 .with_max_attributes_per_span(16)
//!                 .with_max_events_per_span(16)
//!                 .with_resource(Resource::new(vec![KeyValue::new("service.name", "example")])),
//!         )
//!         .with_tonic()
//!         .with_metadata(map)
//!         .install_batch(opentelemetry::runtime::Tokio)?;
//!
//!     tracer.in_span("doing_work", |cx| {
//!         // Traced app logic here...
//!     });
//!
//!     Ok(())
//! }
//! ```
//! [`with_tonic`]: crate::OtlpPipelineBuilder::with_tonic
//! [`with_grpcio`]: crate::OtlpPipelineBuilder::with_grpcio
//!
//!
//! # Grpc libraries comparison
//!
//! The table below provides a short comparison between `grpcio` and `tonic`, two
//! of the most popular grpc libraries in Rust. Users can choose between them when
//! working with otlp with grpc as transport layer.
//!
//! | Project | [hyperium/tonic](https://github.com/hyperium/tonic) | [tikv/grpc-rs](https://github.com/tikv/grpc-rs) |
//! |---|---|---|
//! | Feature name | --features=default | --features=grpc-sys |
//! | gRPC library | [`tonic`](https://crates.io/crates/tonic) | [`grpcio`](https://crates.io/crates/grpcio) |
//! | Transport | [hyperium/hyper](https://github.com/hyperium/hyper) (Rust) | [grpc/grpc](https://github.com/grpc/grpc) (C++ binding) |
//! | TLS support | yes | yes |
//! | TLS optional | yes | yes |
//! | TLS library | rustls | OpenSSL |
//! | Supported .proto generator | [`prost`](https://crates.io/crates/prost) | [`prost`](https://crates.io/crates/prost), [`protobuf`](https://crates.io/crates/protobuf) |
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
#![cfg_attr(docsrs, feature(doc_cfg), deny(broken_intra_doc_links))]
#![cfg_attr(test, deny(warnings))]

use opentelemetry::{global, sdk, sdk::trace::TraceRuntime, trace::TracerProvider};

#[cfg(any(feature = "grpc-sys", feature = "http-proto"))]
use std::collections::HashMap;

use std::str::FromStr;
use std::time::Duration;

// proto mod contains file generated by protobuf or other build tools.
// we should manually change it. Thus skip format and lint check.
#[cfg(
not(feature = "integration-testing")
)]
#[rustfmt::skip]
#[allow(warnings)]
mod proto;

#[cfg(feature = "integration-testing")]
#[rustfmt::skip]
#[allow(warnings)]
pub mod proto;

#[cfg(feature = "metrics")]
mod metric;
mod span;
mod transform;

#[cfg(feature = "tonic")]
use tonic::metadata::MetadataMap;

#[cfg(all(feature = "tonic", feature = "tls"))]
use tonic::transport::ClientTlsConfig;

pub use crate::span::{ExporterConfig, TraceExporter};

#[cfg(feature = "tonic")]
pub use crate::span::TonicConfig;

#[cfg(feature = "grpc-sys")]
pub use crate::span::GrpcioConfig;

#[cfg(feature = "http-proto")]
pub use crate::span::HttpConfig;

#[cfg(feature = "metrics")]
pub use crate::metric::{new_metrics_pipeline, MetricsExporter, OtlpMetricPipelineBuilder};

#[cfg(feature = "grpc-sys")]
pub use crate::span::{Compression, Credentials};

#[cfg(feature = "http-proto")]
use opentelemetry_http::HttpClient;

use opentelemetry::sdk::export::ExportError;
use opentelemetry::trace::TraceError;

/// Target to which the exporter is going to send spans or metrics, defaults to https://localhost:4317.
const OTEL_EXPORTER_OTLP_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_ENDPOINT";
/// Default target to which the exporter is going to send spans or metrics.
const OTEL_EXPORTER_OTLP_ENDPOINT_DEFAULT: &str = "https://localhost:4317";
/// Max waiting time for the backend to process each spans or metrics batch, defaults to 10 seconds.
const OTEL_EXPORTER_OTLP_TIMEOUT: &str = "OTEL_EXPORTER_OTLP_TIMEOUT";
/// Default max waiting time for the backend to process each spans or metrics batch.
const OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT: u64 = 10;

/// Target to which the exporter is going to send spans, defaults to https://localhost:4317.
const OTEL_EXPORTER_OTLP_TRACES_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT";
/// Max waiting time for the backend to process each spans batch, defaults to 10s.
const OTEL_EXPORTER_OTLP_TRACES_TIMEOUT: &str = "OTEL_EXPORTER_OTLP_TRACES_TIMEOUT";

/// Create a new pipeline builder with the recommended configuration.
///
/// ## Examples
///
/// ```no_run
/// fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
///     let tracer = opentelemetry_otlp::new_pipeline().with_tonic().install_simple()?;
///
///     Ok(())
/// }
/// ```
pub fn new_pipeline() -> OtlpPipelineBuilder {
    OtlpPipelineBuilder::default()
}

/// Recommended configuration for an OTLP exporter pipeline.
///
/// ## Examples
///
/// ```no_run
/// fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
///     let tracer = opentelemetry_otlp::new_pipeline().with_tonic().install_simple()?;
///
///     Ok(())
/// }
/// ```
#[derive(Default, Debug)]
pub struct OtlpPipelineBuilder {
    exporter_config: ExporterConfig,
    trace_config: Option<sdk::trace::Config>,
}

impl OtlpPipelineBuilder {
    /// Set the address of the OTLP collector. If not set, the default address is used.
    pub fn with_endpoint<T: Into<String>>(mut self, endpoint: T) -> Self {
        self.exporter_config.endpoint = endpoint.into();
        self
    }

    /// Set the protocol to use when communicating with the collector.
    pub fn with_protocol(mut self, protocol: Protocol) -> Self {
        self.exporter_config.protocol = protocol;
        self
    }

    /// Set the timeout to the collector.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.exporter_config.timeout = timeout;
        self
    }

    /// Set the trace provider configuration.
    pub fn with_trace_config(mut self, trace_config: sdk::trace::Config) -> Self {
        self.trace_config = Some(trace_config);
        self
    }

    /// Set the trace provider configuration from the given environment variables.
    ///
    /// If the value in environment variables is illegal, will fall back to use default value.
    pub fn with_env(mut self) -> Self {
        let endpoint = match std::env::var(OTEL_EXPORTER_OTLP_TRACES_ENDPOINT) {
            Ok(val) => val,
            Err(_) => std::env::var(OTEL_EXPORTER_OTLP_ENDPOINT)
                .unwrap_or_else(|_| OTEL_EXPORTER_OTLP_ENDPOINT_DEFAULT.to_string()),
        };
        self.exporter_config.endpoint = endpoint;

        let timeout = match std::env::var(OTEL_EXPORTER_OTLP_TRACES_TIMEOUT) {
            Ok(val) => u64::from_str(&val).unwrap_or(OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT),
            Err(_) => std::env::var(OTEL_EXPORTER_OTLP_TIMEOUT)
                .map(|val| u64::from_str(&val).unwrap_or(OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT))
                .unwrap_or(OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT),
        };
        self.exporter_config.timeout = Duration::from_secs(timeout);
        self
    }

    /// Use tonic as grpc layer, return a `TonicPipelineBuilder` to config tonic and build the exporter.
    #[cfg(feature = "tonic")]
    pub fn with_tonic(self) -> TonicPipelineBuilder {
        TonicPipelineBuilder {
            exporter_config: self.exporter_config,
            trace_config: self.trace_config,
            tonic_config: TonicConfig::default(),
            channel: None,
        }
    }

    /// Use grpcio as grpc layer, return a `GrpcioPipelineBuilder` to config the grpcio and build the exporter.
    #[cfg(feature = "grpc-sys")]
    pub fn with_grpcio(self) -> GrpcioPipelineBuilder {
        GrpcioPipelineBuilder {
            exporter_config: self.exporter_config,
            trace_config: self.trace_config,
            grpcio_config: GrpcioConfig::default(),
        }
    }

    /// Use HTTP as transport layer, return a `HttpPipelineBuilder` to config the http transport
    /// and build the exporter
    #[cfg(feature = "http-proto")]
    pub fn with_http(self) -> HttpPipelineBuilder {
        HttpPipelineBuilder {
            exporter_config: self.exporter_config,
            trace_config: self.trace_config,
            http_config: HttpConfig::default(),
        }
    }
}

/// Build a trace exporter that uses [tonic] as grpc layer and opentelemetry protocol.
///
/// It provides methods to config tonic. The methods can be chained. The exporter can be built by calling
/// [`install_simple`] or [`install_batch`].
///
/// `TonicPipelineBuilder` can be constructed by calling [`with_tonic`] function in [`OtlpPipelineBuilder`]
///
/// [`with_tonic`]: crate::OtlpPipelineBuilder::with_tonic
/// [`OtlpPipelineBuilder`]: crate::OtlpPipelineBuilder
/// [`install_simple`]: crate::TonicPipelineBuilder::install_simple
/// [`install_batch`]: crate::TonicPipelineBuilder::install_batch
/// [tonic]: https://github.com/hyperium/tonic
#[derive(Default, Debug)]
#[cfg(feature = "tonic")]
pub struct TonicPipelineBuilder {
    exporter_config: ExporterConfig,
    tonic_config: TonicConfig,
    trace_config: Option<sdk::trace::Config>,
    channel: Option<tonic::transport::Channel>,
}

#[cfg(feature = "tonic")]
impl TonicPipelineBuilder {
    /// Set the TLS settings for the collector endpoint.
    #[cfg(feature = "tls")]
    pub fn with_tls_config(mut self, tls_config: ClientTlsConfig) -> Self {
        self.tonic_config.tls_config = Some(tls_config);
        self
    }

    /// Set custom metadata entries to send to the collector.
    pub fn with_metadata(mut self, metadata: MetadataMap) -> Self {
        self.tonic_config.metadata = Some(metadata);
        self
    }

    /// Use `channel` as tonic's transport channel.
    /// this will override tls config and should only be used
    /// when working with non-HTTP transports.
    ///
    /// Users MUST make sure the [`ExporterConfig::timeout`] is
    /// the same as the channel's timeout.
    ///
    /// [`ExporterConfig::timeout`]: crate::span::ExporterConfig::timeout
    pub fn with_channel(mut self, channel: tonic::transport::Channel) -> Self {
        self.channel = Some(channel);
        self
    }

    /// Install a trace exporter using [tonic] as grpc layer and a simple span processor.
    ///
    /// Returns a [`Tracer`] with the name `opentelemetry-otlp` and current crate version.
    ///
    /// `install_simple` will panic if not called within a tokio runtime
    ///
    /// [`Tracer`]: opentelemetry::trace::Tracer
    /// [tonic]: https://github.com/hyperium/tonic
    pub fn install_simple(self) -> Result<sdk::trace::Tracer, TraceError> {
        let exporter = match self.channel {
            Some(channel) => {
                TraceExporter::from_tonic_channel(self.exporter_config, self.tonic_config, channel)
            }
            None => TraceExporter::new_tonic(self.exporter_config, self.tonic_config),
        }?;
        Ok(build_simple_with_exporter(exporter, self.trace_config))
    }

    /// Install a trace exporter using [tonic] as grpc lazer and a batch span processor using the
    /// specified runtime.
    ///
    /// Returns a [`Tracer`] with the name `opentelemetry-otlp` and current crate version.
    ///
    /// `install_batch` will panic if not called within a tokio runtime
    ///
    /// [`Tracer`]: opentelemetry::trace::Tracer
    /// [tonic]: https://github.com/hyperium/tonic
    pub fn install_batch<R: TraceRuntime>(
        self,
        runtime: R,
    ) -> Result<sdk::trace::Tracer, TraceError> {
        let exporter = match self.channel {
            Some(channel) => {
                TraceExporter::from_tonic_channel(self.exporter_config, self.tonic_config, channel)
            }
            None => TraceExporter::new_tonic(self.exporter_config, self.tonic_config),
        }?;
        Ok(build_batch_with_exporter(
            exporter,
            self.trace_config,
            runtime,
        ))
    }
}

/// Build a trace exporter that uses [grpcio] as grpc layer and opentelemetry protocol.
///
/// It provides methods to config grpcio. The methods can be chained. The exporter can be built by calling
/// [`install_simple`] or [`install_batch`].
///
/// `GrpcioPipelineBuilder` can be constructed by calling [`with_grpcio`] function in [`OtlpPipelineBuilder`].
///
/// [`with_grpcio`]: crate::OtlpPipelineBuilder::with_grpcio
/// [`OtlpPipelineBuilder`]: crate::OtlpPipelineBuilder
/// [`install_simple`]: crate::GrpcioPipelineBuilder::install_simple
/// [`install_batch`]: crate::GrpcioPipelineBuilder::install_batch
/// [grpcio]: https://github.com/tikv/grpc-rs
#[derive(Default, Debug)]
#[cfg(feature = "grpc-sys")]
pub struct GrpcioPipelineBuilder {
    exporter_config: ExporterConfig,
    grpcio_config: GrpcioConfig,
    trace_config: Option<sdk::trace::Config>,
}

#[cfg(feature = "grpc-sys")]
impl GrpcioPipelineBuilder {
    /// Set the credentials to use when communicating with the collector.
    pub fn with_credentials(mut self, credentials: Credentials) -> Self {
        self.grpcio_config.credentials = Some(credentials);
        self
    }

    /// Set additional headers to send to the collector.
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.grpcio_config.headers = Some(headers);
        self
    }

    /// Set the compression algorithm to use when communicating with the collector.
    pub fn with_compression(mut self, compression: Compression) -> Self {
        self.grpcio_config.compression = Some(compression);
        self
    }

    /// Enable TLS without any certificate pinning.
    pub fn with_tls(mut self, use_tls: bool) -> Self {
        self.grpcio_config.use_tls = Some(use_tls);
        self
    }

    /// Set the number of GRPC worker threads to poll queues.
    pub fn with_completion_queue_count(mut self, count: usize) -> Self {
        self.grpcio_config.completion_queue_count = count;
        self
    }

    /// Install a trace exporter using [grpcio] as grpc layer and a simple span processor.
    ///
    /// Returns a [`Tracer`] with the name `opentelemetry-otlp` and current crate version.
    ///
    /// [`Tracer`]: opentelemetry::trace::Tracer
    /// [grpcio]: https://github.com/tikv/grpc-rs
    pub fn install_simple(self) -> Result<sdk::trace::Tracer, TraceError> {
        let exporter = TraceExporter::new_grpcio(self.exporter_config, self.grpcio_config);
        Ok(build_simple_with_exporter(exporter, self.trace_config))
    }

    /// Install a trace exporter using [grpcio] as grpc layer and a batch span processor using the
    /// specified runtime.
    ///
    /// Returns a [`Tracer`] with the name `opentelemetry-otlp` and current crate version.
    ///
    /// [`Tracer`]: opentelemetry::trace::Tracer
    /// [grpcio]: https://github.com/tikv/grpc-rs
    pub fn install_batch<R: TraceRuntime>(
        self,
        runtime: R,
    ) -> Result<sdk::trace::Tracer, TraceError> {
        let exporter = TraceExporter::new_grpcio(self.exporter_config, self.grpcio_config);
        Ok(build_batch_with_exporter(
            exporter,
            self.trace_config,
            runtime,
        ))
    }
}

/// Build a trace exporter that uses HTTP transport and opentelemetry protocol.
///
/// It provides methods to config http client. The methods can be chained. The exporter can be built by calling
/// [`install_simple`] or [`install_batch`].
///
/// `HttpPipelineBuilder` can be constructed by calling [`with_http`] function in [`OtlpPipelineBuilder`]
///
/// [`with_http`]: crate::OtlpPipelineBuilder::with_http
/// [`OtlpPipelineBuilder`]: crate::OtlpPipelineBuilder
/// [`install_simple`]: crate::HttpPipelineBuilder::install_simple
/// [`install_batch`]: crate::HttpPipelineBuilder::install_batch
#[derive(Default, Debug)]
#[cfg(feature = "http-proto")]
pub struct HttpPipelineBuilder {
    exporter_config: ExporterConfig,
    http_config: HttpConfig,
    trace_config: Option<sdk::trace::Config>,
}

#[cfg(feature = "http-proto")]
impl HttpPipelineBuilder {
    /// Assign client implementation
    pub fn with_http_client<T: HttpClient + 'static>(mut self, client: T) -> Self {
        self.http_config.client = Some(Box::new(client));
        self
    }

    /// Set additional headers to send to the collector.
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.http_config.headers = Some(headers);
        self
    }

    /// Install a trace exporter using HTTP as transport layer and a simple span processor.
    ///
    /// Returns a [`Tracer`] with the name `opentelemetry-otlp` and current crate version.
    ///
    /// `install_simple` will panic if not called within a tokio runtime
    ///
    /// [`Tracer`]: opentelemetry::trace::Tracer
    pub fn install_simple(self) -> Result<sdk::trace::Tracer, TraceError> {
        let exporter = TraceExporter::new_http(self.exporter_config, self.http_config)?;
        Ok(build_simple_with_exporter(exporter, self.trace_config))
    }

    /// Install a trace exporter using HTTP as transport layer and a batch span processor using the
    /// specified runtime.
    ///
    /// Returns a [`Tracer`] with the name `opentelemetry-otlp` and current crate version.
    ///
    /// `install_batch` will panic if not called within a tokio runtime
    ///
    /// [`Tracer`]: opentelemetry::trace::Tracer
    pub fn install_batch<R: TraceRuntime>(
        self,
        runtime: R,
    ) -> Result<sdk::trace::Tracer, TraceError> {
        let exporter = TraceExporter::new_http(self.exporter_config, self.http_config)?;
        Ok(build_batch_with_exporter(
            exporter,
            self.trace_config,
            runtime,
        ))
    }
}

fn build_simple_with_exporter(
    exporter: TraceExporter,
    trace_config: Option<sdk::trace::Config>,
) -> sdk::trace::Tracer {
    let mut provider_builder = sdk::trace::TracerProvider::builder().with_simple_exporter(exporter);
    if let Some(config) = trace_config {
        provider_builder = provider_builder.with_config(config);
    }
    let provider = provider_builder.build();
    let tracer = provider.get_tracer("opentelemetry-otlp", Some(env!("CARGO_PKG_VERSION")));
    let _ = global::set_tracer_provider(provider);
    tracer
}

fn build_batch_with_exporter<R: TraceRuntime>(
    exporter: TraceExporter,
    trace_config: Option<sdk::trace::Config>,
    runtime: R,
) -> sdk::trace::Tracer {
    let mut provider_builder =
        sdk::trace::TracerProvider::builder().with_batch_exporter(exporter, runtime);
    if let Some(config) = trace_config {
        provider_builder = provider_builder.with_config(config);
    }
    let provider = provider_builder.build();
    let tracer = provider.get_tracer("opentelemetry-otlp", Some(env!("CARGO_PKG_VERSION")));
    let _ = global::set_tracer_provider(provider);
    tracer
}

/// Wrap type for errors from opentelemetry otel
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Error from tonic::transport::Error
    #[cfg(feature = "tonic")]
    #[error("transport error {0}")]
    Transport(#[from] tonic::transport::Error),

    /// Error from tonic::codegen::http::uri::InvalidUri
    #[cfg(any(feature = "tonic", feature = "http-proto"))]
    #[error("invalid URI {0}")]
    InvalidUri(#[from] http::uri::InvalidUri),

    /// Error from tonic::Status
    #[cfg(feature = "tonic")]
    #[error("status error {0}")]
    Status(#[from] tonic::Status),

    /// Error from grpcio module
    #[cfg(feature = "grpc-sys")]
    #[error("grpcio error {0}")]
    Grpcio(#[from] grpcio::Error),

    /// Http requests failed
    #[cfg(feature = "http-proto")]
    #[error("No Http Client, you must select one")]
    NoHttpClient,

    /// Http requests failed
    #[cfg(feature = "http-proto")]
    #[error("http request failed with {0}")]
    RequestFailed(#[from] http::Error),

    /// Invalid Header Value
    #[cfg(feature = "http-proto")]
    #[error("http header value error {0}")]
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),

    /// Invalid Header Name
    #[cfg(feature = "http-proto")]
    #[error("http header name error {0}")]
    InvalidHeaderName(#[from] http::header::InvalidHeaderName),

    /// Prost encode failed
    #[cfg(feature = "http-proto")]
    #[error("prost encoding error {0}")]
    EncodeError(#[from] prost::EncodeError),

    /// The lock in exporters has been poisoned.
    #[cfg(feature = "metrics")]
    #[error("the lock of the {0} has been poisoned")]
    PoisonedLock(&'static str),
}

impl ExportError for Error {
    fn exporter_name(&self) -> &'static str {
        "otlp"
    }
}

/// The communication protocol to use when exporting data.
#[derive(Clone, Copy, Debug)]
pub enum Protocol {
    /// GRPC protocol
    Grpc,
    // TODO add support for other protocols
    // HttpJson,
    /// HTTP protocol with binary protobuf
    HttpBinary,
}

#[cfg(test)]
mod tests {
    use crate::{
        new_pipeline, OTEL_EXPORTER_OTLP_ENDPOINT, OTEL_EXPORTER_OTLP_TIMEOUT,
        OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT, OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
        OTEL_EXPORTER_OTLP_TRACES_TIMEOUT,
    };

    #[test]
    fn test_pipeline_builder_from_env() {
        std::env::set_var(OTEL_EXPORTER_OTLP_ENDPOINT, "https://otlp_endpoint:4317");
        std::env::set_var(OTEL_EXPORTER_OTLP_TIMEOUT, "bad_timeout");

        let mut pipeline_builder = new_pipeline().with_env();
        assert_eq!(
            pipeline_builder.exporter_config.timeout,
            std::time::Duration::from_secs(OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT)
        );

        std::env::set_var(OTEL_EXPORTER_OTLP_TIMEOUT, "60");

        pipeline_builder = new_pipeline().with_env();
        assert_eq!(
            pipeline_builder.exporter_config.timeout,
            std::time::Duration::from_secs(60)
        );

        std::env::remove_var(OTEL_EXPORTER_OTLP_ENDPOINT);
        std::env::remove_var(OTEL_EXPORTER_OTLP_TIMEOUT);
        assert!(std::env::var(OTEL_EXPORTER_OTLP_ENDPOINT).is_err());
        assert!(std::env::var(OTEL_EXPORTER_OTLP_TIMEOUT).is_err());

        // test from traces env var
        std::env::set_var(
            OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
            "https://otlp_traces_endpoint:4317",
        );
        std::env::set_var(OTEL_EXPORTER_OTLP_TRACES_TIMEOUT, "bad_timeout");

        let mut pipeline_builder = new_pipeline().with_env();
        assert_eq!(
            pipeline_builder.exporter_config.timeout,
            std::time::Duration::from_secs(OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT)
        );

        std::env::set_var(OTEL_EXPORTER_OTLP_TRACES_TIMEOUT, "60");

        pipeline_builder = new_pipeline().with_env();
        assert_eq!(
            pipeline_builder.exporter_config.timeout,
            std::time::Duration::from_secs(60)
        );

        std::env::remove_var(OTEL_EXPORTER_OTLP_TRACES_ENDPOINT);
        std::env::remove_var(OTEL_EXPORTER_OTLP_TRACES_TIMEOUT);
        assert!(std::env::var(OTEL_EXPORTER_OTLP_TRACES_ENDPOINT).is_err());
        assert!(std::env::var(OTEL_EXPORTER_OTLP_TRACES_TIMEOUT).is_err());
    }
}
