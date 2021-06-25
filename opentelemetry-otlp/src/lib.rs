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
//!     // First, create a OTLP exporter builder. Configure it as you need.
//!     let otlp_exporter = opentelemetry_otlp::new_exporter().tonic();
//!     // Then pass it into pipeline builder
//!     let tracer = opentelemetry_otlp::new_pipeline()
//!             .tracing()
//!             .with_exporter(otlp_exporter)
//!             .install_simple()?;
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
//!     .tracing()
//!     .with_exporter(opentelemetry_otlp::new_exporter().tonic())
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
//!         .tracing()
//!         .with_exporter(
//!             opentelemetry_otlp::new_exporter()
//!             .tonic()
//!             .with_endpoint("http://localhost:4317")
//!             .with_timeout(Duration::from_secs(3))
//!             .with_metadata(map)
//!          )
//!         .with_trace_config(
//!             trace::config()
//!                 .with_sampler(Sampler::AlwaysOn)
//!                 .with_id_generator(IdGenerator::default())
//!                 .with_max_events_per_span(64)
//!                 .with_max_attributes_per_span(16)
//!                 .with_max_events_per_span(16)
//!                 .with_resource(Resource::new(vec![KeyValue::new("service.name", "example")])),
//!         )
//!         .install_batch(opentelemetry::runtime::Tokio)?;
//!
//!     tracer.in_span("doing_work", |cx| {
//!         // Traced app logic here...
//!     });
//!
//!     Ok(())
//! }
//! ```
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

mod exporter;
#[cfg(feature = "metrics")]
mod metric;
mod span;
mod transform;

pub use crate::exporter::ExportConfig;
pub use crate::span::SpanExporter;

#[cfg(feature = "metrics")]
pub use crate::metric::{MetricsExporter, OtlpMetricPipelineBuilder};

pub use crate::exporter::{HasExportConfig, WithExportConfig};

use opentelemetry::sdk::export::ExportError;

#[cfg(feature = "grpc-sys")]
use crate::exporter::grpcio::GrpcioExporterBuilder;
#[cfg(feature = "http-proto")]
use crate::exporter::http::HttpExporterBuilder;
#[cfg(feature = "tonic")]
use crate::exporter::tonic::TonicExporterBuilder;

/// General builder for both tracing and metrics.
#[derive(Debug)]
pub struct OtlpPipeline;

/// Build a OTLP metrics or tracing exporter builder.
#[derive(Debug)]
pub struct OtlpExporterPipeline;

impl OtlpExporterPipeline {
    /// Use tonic as grpc layer, return a `TonicExporterBuilder` to config tonic and build the exporter.
    #[cfg(feature = "tonic")]
    pub fn tonic(self) -> TonicExporterBuilder {
        TonicExporterBuilder::default()
    }

    /// Use grpcio as grpc layer, return a `GrpcioExporterBuilder` to config the grpcio and build the exporter.
    #[cfg(feature = "grpc-sys")]
    pub fn grpcio(self) -> GrpcioExporterBuilder {
        GrpcioExporterBuilder::default()
    }

    /// Use HTTP as transport layer, return a `HttpExporterBuilder` to config the http transport
    /// and build the exporter
    #[cfg(feature = "http-proto")]
    pub fn http(self) -> HttpExporterBuilder {
        HttpExporterBuilder::default()
    }
}

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
pub fn new_pipeline() -> OtlpPipeline {
    OtlpPipeline
}

/// Create a builder to build OTLP metrics exporter or tracing exporter.
pub fn new_exporter() -> OtlpExporterPipeline {
    OtlpExporterPipeline
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
