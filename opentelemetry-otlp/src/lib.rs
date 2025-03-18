//! # OpenTelemetry OTLP Exporter
//!
//! The OTLP Exporter enables exporting telemetry data (logs, metrics, and traces) in the
//! OpenTelemetry Protocol (OTLP) format to compatible backends. These backends include:
//!
//! - OpenTelemetry Collector
//! - Open-source observability tools (Prometheus, Jaeger, etc.)
//! - Vendor-specific monitoring platforms
//!
//! This crate supports sending OTLP data via:
//! - gRPC
//! - HTTP (binary protobuf or JSON)
//!
//! ## Quickstart with OpenTelemetry Collector
//!
//! ### HTTP Transport (Port 4318)
//!
//! Run the OpenTelemetry Collector:
//!
//! ```shell
//! $ docker run -p 4318:4318 otel/opentelemetry-collector:latest
//! ```
//!
//! Configure your application to export traces via HTTP:
//!
//! ```no_run
//! # #[cfg(all(feature = "trace", feature = "http-proto"))]
//! # {
//! use opentelemetry::global;
//! use opentelemetry::trace::Tracer;
//! use opentelemetry_otlp::Protocol;
//! use opentelemetry_otlp::WithExportConfig;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
//!     // Initialize OTLP exporter using HTTP binary protocol
//!     let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
//!         .with_http()
//!         .with_protocol(Protocol::HttpBinary)
//!         .build()?;
//!
//!     // Create a tracer provider with the exporter
//!     let _ = opentelemetry_sdk::trace::SdkTracerProvider::builder()
//!         .with_simple_exporter(otlp_exporter)
//!         .build();
//!
//!     // Get a tracer and create spans
//!     let tracer = global::tracer("my_tracer");
//!     tracer.in_span("doing_work", |_cx| {
//!         // Your application logic here...
//!     });
//!
//!     Ok(())
//! # }
//! }
//! ```
//!
//! ### gRPC Transport (Port 4317)
//!
//! Run the OpenTelemetry Collector:
//!
//! ```shell
//! $ docker run -p 4317:4317 otel/opentelemetry-collector:latest
//! ```
//!
//! Configure your application to export traces via gRPC:
//!
//! ```no_run
//! # #[cfg(all(feature = "trace", feature = "grpc-tonic"))]
//! # {
//! use opentelemetry::global;
//! use opentelemetry::trace::Tracer;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
//!     // Initialize OTLP exporter using gRPC (Tonic)
//!     let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
//!         .with_tonic()
//!         .build()?;
//!
//!     // Create a tracer provider with the exporter
//!     let _ = opentelemetry_sdk::trace::SdkTracerProvider::builder()
//!         .with_simple_exporter(otlp_exporter)
//!         .build();
//!
//!     // Get a tracer and create spans
//!     let tracer = global::tracer("my_tracer");
//!     tracer.in_span("doing_work", |_cx| {
//!         // Your application logic here...
//!     });
//!
//!     Ok(())
//! # }
//! }
//! ```
//!
//! ## Using with Jaeger
//!
//! Jaeger natively supports the OTLP protocol, making it easy to send traces directly:
//!
//! ```shell
//! $ docker run -p 16686:16686 -p 4317:4317 -e COLLECTOR_OTLP_ENABLED=true jaegertracing/all-in-one:latest
//! ```
//!
//! After running your application configured with the OTLP exporter, view traces at:
//! `http://localhost:16686`
//!
//! ## Using with Prometheus (TODO)
//! ## Show Logs, Metrics too (TODO)
//!
//! ## Performance
//!
//! For optimal performance, a batch exporting processor is recommended as the simple
//! processor will export each span synchronously on dropping, and is only good
//! for test/debug purposes.
//!
//! ```toml
//! [dependencies]
//! opentelemetry-otlp = { version = "*", features = ["grpc-tonic"] }
//! ```
//!
//! ```no_run
//! # #[cfg(all(feature = "trace", feature = "grpc-tonic"))]
//! # {
//! use opentelemetry::global;
//! use opentelemetry::trace::Tracer;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
//!     // First, create a OTLP exporter builder. Configure it as you need.
//!     let otlp_exporter = opentelemetry_otlp::SpanExporter::builder().with_tonic().build()?;
//!     // Then pass it into provider builder
//!     let _ = opentelemetry_sdk::trace::SdkTracerProvider::builder()
//!         .with_batch_exporter(otlp_exporter)
//!         .build();
//!     let tracer = global::tracer("my_tracer");
//!     tracer.in_span("doing_work", |cx| {
//!         // Traced app logic here...
//!     });
//!
//!     Ok(())
//!   # }
//! }
//! ```
//!
//! [`tokio`]: https://tokio.rs
//!
//! # Feature Flags
//! The following feature flags can enable exporters for different telemetry signals:
//!
//! * `trace`: Includes the trace exporters.
//! * `metrics`: Includes the metrics exporters.
//! * `logs`: Includes the logs exporters.
//!
//! The following feature flags generate additional code and types:
//! * `serialize`: Enables serialization support for type defined in this crate via `serde`.
//!
//! The following feature flags offer additional configurations on gRPC:
//!
//! For users using `tonic` as grpc layer:
//! * `grpc-tonic`: Use `tonic` as grpc layer.
//! * `gzip-tonic`: Use gzip compression for `tonic` grpc layer.
//! * `zstd-tonic`: Use zstd compression for `tonic` grpc layer.
//! * `tls-roots`: Adds system trust roots to rustls-based gRPC clients using the rustls-native-certs crate
//! * `tls-webpki-roots`: Embeds Mozilla's trust roots to rustls-based gRPC clients using the webpki-roots crate
//!
//! The following feature flags offer additional configurations on http:
//!
//! * `http-proto`: Use http as transport layer, protobuf as body format. This feature is enabled by default.
//! * `reqwest-blocking-client`: Use reqwest blocking http client. This feature is enabled by default.
//! * `reqwest-client`: Use reqwest http client.
//! * `reqwest-rustls`: Use reqwest with TLS with system trust roots via `rustls-native-certs` crate.
//! * `reqwest-rustls-webpki-roots`: Use reqwest with TLS with Mozilla's trust roots via `webpki-roots` crate.
//!
//! # Kitchen Sink Full Configuration
//!
//! Example showing how to override all configuration options.
//!
//! Generally there are two parts of configuration. One is the exporter, the other is the provider.
//! Users can configure the exporter using [SpanExporter::builder()] for traces,
//! and [MetricExporter::builder()] + [opentelemetry_sdk::metrics::PeriodicReader::builder()] for metrics.
//! Once you have an exporter, you can add it to either a [opentelemetry_sdk::trace::SdkTracerProvider::builder()] for traces,
//! or [opentelemetry_sdk::metrics::SdkMeterProvider::builder()] for metrics.
//!
//! ```no_run
//! use opentelemetry::{global, KeyValue, trace::Tracer};
//! use opentelemetry_sdk::{trace::{self, RandomIdGenerator, Sampler}, Resource};
//! # #[cfg(feature = "metrics")]
//! use opentelemetry_sdk::metrics::Temporality;
//! use opentelemetry_otlp::{Protocol, WithExportConfig, WithTonicConfig};
//! use std::time::Duration;
//! # #[cfg(feature = "grpc-tonic")]
//! use tonic::metadata::*;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
//!     # #[cfg(all(feature = "trace", feature = "grpc-tonic"))]
//!     # let tracer = {
//!     let mut map = MetadataMap::with_capacity(3);
//!
//!     map.insert("x-host", "example.com".parse().unwrap());
//!     map.insert("x-number", "123".parse().unwrap());
//!     map.insert_bin("trace-proto-bin", MetadataValue::from_bytes(b"[binary data]"));
//!     let exporter = opentelemetry_otlp::SpanExporter::builder()
//!         .with_tonic()
//!         .with_endpoint("http://localhost:4317")
//!         .with_timeout(Duration::from_secs(3))
//!         .with_metadata(map)
//!         .build()?;
//!
//!     let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
//!         .with_batch_exporter(exporter)
//!         .with_sampler(Sampler::AlwaysOn)
//!         .with_id_generator(RandomIdGenerator::default())
//!         .with_max_events_per_span(64)
//!         .with_max_attributes_per_span(16)
//!         .with_resource(Resource::builder_empty().with_attributes([KeyValue::new("service.name", "example")]).build())
//!         .build();
//!     global::set_tracer_provider(tracer_provider.clone());
//!     let tracer = global::tracer("tracer-name");
//!         # tracer
//!     # };
//!
//!     # #[cfg(all(feature = "metrics", feature = "grpc-tonic"))]
//!     # {
//!     let exporter = opentelemetry_otlp::MetricExporter::builder()
//!        .with_tonic()
//!        .with_endpoint("http://localhost:4318/v1/metrics")
//!        .with_protocol(Protocol::Grpc)
//!        .with_timeout(Duration::from_secs(3))
//!        .build()
//!        .unwrap();
//!
//!    let provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
//!         .with_periodic_exporter(exporter)
//!         .with_resource(Resource::builder_empty().with_attributes([KeyValue::new("service.name", "example")]).build())
//!         .build();
//!     # }
//!
//! # #[cfg(all(feature = "trace", feature = "grpc-tonic"))]
//! # {
//!     tracer.in_span("doing_work", |cx| {
//!         // Traced app logic here...
//!     });
//! # }
//!
//!     Ok(())
//! }
//! ```
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
#![cfg_attr(
    docsrs,
    feature(doc_cfg, doc_auto_cfg),
    deny(rustdoc::broken_intra_doc_links)
)]
#![cfg_attr(test, deny(warnings))]

mod exporter;
#[cfg(feature = "logs")]
#[cfg(any(feature = "http-proto", feature = "http-json", feature = "grpc-tonic"))]
mod logs;
#[cfg(feature = "metrics")]
#[cfg(any(feature = "http-proto", feature = "http-json", feature = "grpc-tonic"))]
mod metric;
#[cfg(feature = "trace")]
#[cfg(any(feature = "http-proto", feature = "http-json", feature = "grpc-tonic"))]
mod span;

pub use crate::exporter::Compression;
pub use crate::exporter::ExportConfig;
pub use crate::exporter::ExporterBuildError;
#[cfg(feature = "trace")]
#[cfg(any(feature = "http-proto", feature = "http-json", feature = "grpc-tonic"))]
pub use crate::span::{
    SpanExporter, OTEL_EXPORTER_OTLP_TRACES_COMPRESSION, OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
    OTEL_EXPORTER_OTLP_TRACES_HEADERS, OTEL_EXPORTER_OTLP_TRACES_TIMEOUT,
};

#[cfg(feature = "metrics")]
#[cfg(any(feature = "http-proto", feature = "http-json", feature = "grpc-tonic"))]
pub use crate::metric::{
    MetricExporter, OTEL_EXPORTER_OTLP_METRICS_COMPRESSION, OTEL_EXPORTER_OTLP_METRICS_ENDPOINT,
    OTEL_EXPORTER_OTLP_METRICS_HEADERS, OTEL_EXPORTER_OTLP_METRICS_TIMEOUT,
};

#[cfg(feature = "logs")]
#[cfg(any(feature = "http-proto", feature = "http-json", feature = "grpc-tonic"))]
pub use crate::logs::{
    LogExporter, OTEL_EXPORTER_OTLP_LOGS_COMPRESSION, OTEL_EXPORTER_OTLP_LOGS_ENDPOINT,
    OTEL_EXPORTER_OTLP_LOGS_HEADERS, OTEL_EXPORTER_OTLP_LOGS_TIMEOUT,
};

#[cfg(any(feature = "http-proto", feature = "http-json"))]
pub use crate::exporter::http::{HasHttpConfig, WithHttpConfig};

#[cfg(feature = "grpc-tonic")]
pub use crate::exporter::tonic::{HasTonicConfig, WithTonicConfig};

pub use crate::exporter::{
    HasExportConfig, WithExportConfig, OTEL_EXPORTER_OTLP_COMPRESSION, OTEL_EXPORTER_OTLP_ENDPOINT,
    OTEL_EXPORTER_OTLP_ENDPOINT_DEFAULT, OTEL_EXPORTER_OTLP_HEADERS, OTEL_EXPORTER_OTLP_PROTOCOL,
    OTEL_EXPORTER_OTLP_PROTOCOL_DEFAULT, OTEL_EXPORTER_OTLP_TIMEOUT,
    OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT,
};

use opentelemetry_sdk::ExportError;

/// Type to indicate the builder does not have a client set.
#[derive(Debug, Default, Clone)]
pub struct NoExporterBuilderSet;

/// Type to hold the [TonicExporterBuilder] and indicate it has been set.
///
/// Allowing access to [TonicExporterBuilder] specific configuration methods.
#[cfg(feature = "grpc-tonic")]
#[derive(Debug, Default)]
pub struct TonicExporterBuilderSet(TonicExporterBuilder);

/// Type to hold the [HttpExporterBuilder] and indicate it has been set.
///
/// Allowing access to [HttpExporterBuilder] specific configuration methods.
#[cfg(any(feature = "http-proto", feature = "http-json"))]
#[derive(Debug, Default)]
pub struct HttpExporterBuilderSet(HttpExporterBuilder);

#[cfg(any(feature = "http-proto", feature = "http-json"))]
pub use crate::exporter::http::HttpExporterBuilder;

#[cfg(feature = "grpc-tonic")]
pub use crate::exporter::tonic::{TonicConfig, TonicExporterBuilder};

#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

/// Wrap type for errors from this crate.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Wrap error from [`tonic::transport::Error`]
    #[cfg(feature = "grpc-tonic")]
    #[error("transport error {0}")]
    Transport(#[from] tonic::transport::Error),

    /// Wrap the [`tonic::codegen::http::uri::InvalidUri`] error
    #[cfg(any(feature = "grpc-tonic", feature = "http-proto", feature = "http-json"))]
    #[error("invalid URI {0}")]
    InvalidUri(#[from] http::uri::InvalidUri),

    /// Wrap type for [`tonic::Status`]
    #[cfg(feature = "grpc-tonic")]
    #[error("the grpc server returns error ({code}): {message}")]
    Status {
        /// grpc status code
        code: tonic::Code,
        /// error message
        message: String,
    },

    /// Http requests failed because no http client is provided.
    #[cfg(any(feature = "http-proto", feature = "http-json"))]
    #[error(
        "no http client, you must select one from features or provide your own implementation"
    )]
    NoHttpClient,

    /// Http requests failed.
    #[cfg(any(feature = "http-proto", feature = "http-json"))]
    #[error("http request failed with {0}")]
    RequestFailed(#[from] opentelemetry_http::HttpError),

    /// The provided value is invalid in HTTP headers.
    #[cfg(any(feature = "grpc-tonic", feature = "http-proto", feature = "http-json"))]
    #[error("http header value error {0}")]
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),

    /// The provided name is invalid in HTTP headers.
    #[cfg(any(feature = "grpc-tonic", feature = "http-proto", feature = "http-json"))]
    #[error("http header name error {0}")]
    InvalidHeaderName(#[from] http::header::InvalidHeaderName),

    /// Prost encode failed
    #[cfg(any(
        feature = "http-proto",
        all(feature = "http-json", not(feature = "trace"))
    ))]
    #[error("prost encoding error {0}")]
    EncodeError(#[from] prost::EncodeError),

    /// The lock in exporters has been poisoned.
    #[cfg(feature = "metrics")]
    #[error("the lock of the {0} has been poisoned")]
    PoisonedLock(&'static str),

    /// Unsupported compression algorithm.
    #[error("unsupported compression algorithm '{0}'")]
    UnsupportedCompressionAlgorithm(String),

    /// Feature required to use the specified compression algorithm.
    #[cfg(any(not(feature = "gzip-tonic"), not(feature = "zstd-tonic")))]
    #[error("feature '{0}' is required to use the compression algorithm '{1}'")]
    FeatureRequiredForCompressionAlgorithm(&'static str, Compression),
}

#[cfg(feature = "grpc-tonic")]
impl From<tonic::Status> for Error {
    fn from(status: tonic::Status) -> Error {
        Error::Status {
            code: status.code(),
            message: {
                if !status.message().is_empty() {
                    let mut result = ", detailed error message: ".to_string() + status.message();
                    if status.code() == tonic::Code::Unknown {
                        let source = (&status as &dyn std::error::Error)
                            .source()
                            .map(|e| format!("{:?}", e));
                        result.push(' ');
                        result.push_str(source.unwrap_or_default().as_ref());
                    }
                    result
                } else {
                    String::new()
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Protocol {
    /// GRPC protocol
    Grpc,
    /// HTTP protocol with binary protobuf
    HttpBinary,
    /// HTTP protocol with JSON payload
    HttpJson,
}

#[derive(Debug, Default)]
#[doc(hidden)]
/// Placeholder type when no exporter pipeline has been configured in telemetry pipeline.
pub struct NoExporterConfig(());
