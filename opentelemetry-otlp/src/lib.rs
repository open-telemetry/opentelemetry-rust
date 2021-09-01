//! The OTLP Exporter supports exporting trace and metric data in the OTLP
//! format to the OpenTelemetry collector or other compatible backend.
//!
//! The OpenTelemetry Collector offers a vendor-agnostic implementation on how
//! to receive, process, and export telemetry data. In addition, it removes
//! the need to run, operate, and maintain multiple agents/collectors in
//! order to support open-source telemetry data formats (e.g. Jaeger,
//! Prometheus, etc.) sending to multiple open-source or commercial back-ends.
//!
//! Currently, this crate only support sending tracing data or metrics in OTLP
//! via grpc and http(in binary format). Supports for other format and protocol
//! will be added in the future. The details of what's currently offering in this
//! crate can be found in this doc.
//!
//! # Quickstart
//!
//! First make sure you have a running version of the opentelemetry collector
//! you want to send data to:
//!
//! ```shell
//! $ docker run -p 4317:4317 otel/opentelemetry-collector-dev:latest
//! ```
//!
//! Then install a new pipeline with the recommended defaults to start exporting
//! telemetry. You will have to build a OTLP exporter first.
//!
//! To start a OTLP tracing pipeline. Use `new_pipeline().tracing()` function.
//!
//! To start a OTLP metrics pipeline. Use `new_pipeline().metrics()` function.
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
//! # Kitchen Sink Full Configuration
//!
//! Example showing how to override all configuration options.
//!
//! Generally there are two parts of configuration. One is metrics config
//! or tracing config. Users can config it via [`OtlpTracePipeline`]
//! or [`OtlpMetricPipeline`]. The other is exporting configuration.
//! Users can set those configurations using [`OtlpExporterPipeline`] based
//! on the choice of exporters.
//!
//! ```no_run
//! use opentelemetry::{KeyValue, trace::Tracer};
//! use opentelemetry::sdk::{trace::{self, IdGenerator, Sampler}, Resource};
//! use opentelemetry::sdk::metrics::{selectors, PushController};
//! use opentelemetry::util::tokio_interval_stream;
//! use opentelemetry_otlp::{Protocol, WithExportConfig, ExportConfig};
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
//!     let export_config = ExportConfig {
//!         endpoint: "http://localhost:4317".to_string(),
//!         timeout: Duration::from_secs(3),
//!         protocol: Protocol::Grpc
//!     };
//!
//!     let meter = opentelemetry_otlp::new_pipeline()
//!         .metrics(tokio::spawn, tokio_interval_stream)
//!         .with_exporter(
//!             opentelemetry_otlp::new_exporter()
//!                 .tonic()
//!                 .with_export_config(export_config),
//!                 // can also config it using with_* functions like the tracing part above.
//!         )
//!         .with_stateful(true)
//!         .with_period(Duration::from_secs(3))
//!         .with_timeout(Duration::from_secs(10))
//!         .with_aggregator_selector(selectors::simple::Selector::Exact)
//!         .build();
//!
//!     tracer.in_span("doing_work", |cx| {
//!         // Traced app logic here...
//!     });
//!
//!     Ok(())
//! }
//! ```
//!
//! # Grpc libraries comparison
//!
//! The table below provides a short comparison between `grpcio` and `tonic`, two
//! of the most popular grpc libraries in Rust. Users can choose between them when
//! working with otlp and grpc.
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
#![cfg_attr(docsrs, feature(doc_cfg), deny(rustdoc::broken_intra_doc_links))]
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
#[doc(hidden)]
pub mod proto;

mod exporter;
#[cfg(feature = "metrics")]
mod metric;
mod span;
mod transform;

pub use crate::exporter::{Credentials, ExportConfig};
pub use crate::span::{OtlpTracePipeline, SpanExporter};

#[cfg(feature = "metrics")]
pub use crate::metric::{MetricsExporter, OtlpMetricPipeline};

pub use crate::exporter::{
    HasExportConfig, WithExportConfig, OTEL_EXPORTER_OTLP_ENDPOINT,
    OTEL_EXPORTER_OTLP_ENDPOINT_DEFAULT, OTEL_EXPORTER_OTLP_TIMEOUT,
    OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT, OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
    OTEL_EXPORTER_OTLP_TRACES_TIMEOUT,
};

use opentelemetry::sdk::export::ExportError;

#[cfg(feature = "grpc-sys")]
pub use crate::exporter::grpcio::GrpcioExporterBuilder;
#[cfg(feature = "http-proto")]
pub use crate::exporter::http::HttpExporterBuilder;
#[cfg(feature = "tonic")]
pub use crate::exporter::tonic::TonicExporterBuilder;

#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

/// General builder for both tracing and metrics.
#[derive(Debug)]
pub struct OtlpPipeline;

/// Build a OTLP metrics or tracing exporter builder. See functions below to understand
/// what's currently supported.
#[derive(Debug)]
pub struct OtlpExporterPipeline;

impl OtlpExporterPipeline {
    /// Use tonic as grpc layer, return a `TonicExporterBuilder` to config tonic and build the exporter.
    ///
    /// This exporter can be used in both `tracing` and `metrics` pipeline.
    #[cfg(feature = "tonic")]
    pub fn tonic(self) -> TonicExporterBuilder {
        TonicExporterBuilder::default()
    }

    /// Use grpcio as grpc layer, return a `GrpcioExporterBuilder` to config the grpcio and build the exporter.
    ///
    /// This exporter can only be used in `tracing` pipeline. Support for `metrics` pipeline will be
    /// added in the future.
    #[cfg(feature = "grpc-sys")]
    pub fn grpcio(self) -> GrpcioExporterBuilder {
        GrpcioExporterBuilder::default()
    }

    /// Use HTTP as transport layer, return a `HttpExporterBuilder` to config the http transport
    /// and build the exporter.
    ///
    /// This exporter can only be used in `tracing` pipeline. Support for `metrics` pipeline will be
    /// added in the future.
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
///     let tracing_builder = opentelemetry_otlp::new_pipeline()
///                    .tracing();
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

/// Wrap type for errors from this crate.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Wrap error from [`tonic::transport::Error`]
    #[cfg(feature = "tonic")]
    #[error("transport error {0}")]
    Transport(#[from] tonic::transport::Error),

    /// Wrap the [`tonic::codegen::http::uri::InvalidUri`] error
    #[cfg(any(feature = "tonic", feature = "http-proto"))]
    #[error("invalid URI {0}")]
    InvalidUri(#[from] http::uri::InvalidUri),

    /// Wrap type for [`tonic::Status`]
    #[cfg(feature = "tonic")]
    #[error("the grpc server returns error {code}")]
    Status {
        /// grpc status code
        code: tonic::Code,
        /// error message
        message: String,
    },

    /// Wrap errors from grpcio.
    #[cfg(feature = "grpc-sys")]
    #[error("grpcio error {0}")]
    Grpcio(#[from] grpcio::Error),

    /// Http requests failed because no http client is provided.
    #[cfg(feature = "http-proto")]
    #[error("no http client, you must select one")]
    NoHttpClient,

    /// Http requests failed.
    #[cfg(feature = "http-proto")]
    #[error("http request failed with {0}")]
    RequestFailed(#[from] http::Error),

    /// The provided value is invalid in HTTP headers.
    #[cfg(feature = "http-proto")]
    #[error("http header value error {0}")]
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),

    /// The provided name is invalid in HTTP headers.
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

    /// The pipeline will need a exporter to complete setup. Throw this error if none is provided.
    #[error("no exporter builder is provided, please provide one using with_exporter() method")]
    NoExporterBuilder,
}

#[cfg(feature = "tonic")]
impl From<tonic::Status> for Error {
    fn from(status: tonic::Status) -> Error {
        Error::Status {
            code: status.code(),
            message: {
                if !status.message().is_empty() {
                    ", detailed error message: ".to_string() + status.message()
                } else {
                    "".to_string()
                }
            },
        }
    }
}

impl ExportError for Error {
    fn exporter_name(&self) -> &'static str {
        "otlp"
    }
}

/// The communication protocol to use when exporting data.
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug)]
pub enum Protocol {
    /// GRPC protocol
    Grpc,
    // TODO add support for other protocols
    // HttpJson,
    /// HTTP protocol with binary protobuf
    HttpBinary,
}
