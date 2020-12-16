//! The OTLP Exporter supports exporting trace and metric data in the OTLP
//! format to the OpenTelemetry collector. The OpenTelemetry Collector offers a
//! vendor-agnostic implementation on how to receive, process, and export
//! telemetry data. In addition, it removes the need to run, operate, and
//! maintain multiple agents/collectors in order to support open-source
//! telemetry data formats (e.g. Jaeger, Prometheus, etc.) sending to multiple
//! open-source or commercial back-ends.
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
//! telemetry:
//!
//! ```no_run
//! use opentelemetry::trace::Tracer;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
//!     let (tracer, _uninstall) = opentelemetry_otlp::new_pipeline().install()?;
//!
//!     tracer.in_span("doing_work", |cx| {
//!         // Traced app logic here...
//!     });
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Options
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
//!
//! ## Performance
//!
//! For optimal performance, a batch exporter is recommended as the simple
//! exporter will export each span synchronously on drop. Enable a runtime
//! to have a batch exporter configured automatically for either executor
//! when using the pipeline.
//!
//! ```toml
//! [dependencies]
//! opentelemetry = { version = "*", features = ["async-std"] }
//! opentelemetry-otlp = { version = "*", features = ["grpc-sys"] }
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
//! [`OtlpPipelineBuilder`]: struct.OtlpPipelineBuilder.html
//!
//!
//! ```text, no_run
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
//!     let (tracer, _uninstall) = opentelemetry_otlp::new_pipeline()
//!         .with_endpoint("localhost:4317")
//!         .with_protocol(Protocol::Grpc)
//!         .with_metadata(map)
//!         .with_timeout(Duration::from_secs(3))
//!         .with_trace_config(
//!             trace::config()
//!                 .with_default_sampler(Sampler::AlwaysOn)
//!                 .with_id_generator(IdGenerator::default())
//!                 .with_max_events_per_span(64)
//!                 .with_max_attributes_per_span(16)
//!                 .with_max_events_per_span(16)
//!                 .with_resource(Resource::new(vec![KeyValue::new("key", "value")])),
//!         )
//!         .install()?;
//!
//!     tracer.in_span("doing_work", |cx| {
//!         // Traced app logic here...
//!     });
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
#![cfg_attr(docsrs, feature(doc_cfg), deny(broken_intra_doc_links))]
#![cfg_attr(test, deny(warnings))]

use opentelemetry::{global, sdk, trace::TracerProvider};

#[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
use std::collections::HashMap;

use std::time::Duration;

#[cfg(feature = "tonic")]
#[rustfmt::skip]
#[allow(clippy::all, unreachable_pub)]
mod proto;

#[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
#[allow(clippy::all, unreachable_pub, dead_code)]
mod proto;

#[cfg(feature = "metrics")]
mod metric;
mod span;
mod transform;

#[cfg(feature = "tonic")]
use tonic::metadata::MetadataMap;

#[cfg(all(feature = "tonic", feature = "tls"))]
use tonic::transport::ClientTlsConfig;

pub use crate::span::{TraceExporter, TraceExporterConfig};

#[cfg(feature = "metrics")]
pub use crate::metric::MetricsExporter;

#[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
pub use crate::span::{Compression, Credentials};

use opentelemetry::sdk::export::ExportError;
use opentelemetry::trace::TraceError;

/// Create a new pipeline builder with the recommended configuration.
///
/// ## Examples
///
/// ```no_run
/// fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
///     let (tracer, _uninstall) = opentelemetry_otlp::new_pipeline().install()?;
///
///     Ok(())
/// }
/// ```
pub fn new_pipeline() -> OtlpPipelineBuilder {
    OtlpPipelineBuilder::default()
}

/// Recommended configuration for an Otlp exporter pipeline.
///
/// ## Examples
///
/// ```no_run
/// fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
///     let (tracer, _uninstall) = opentelemetry_otlp::new_pipeline().install()?;
///
///     Ok(())
/// }
/// ```
#[derive(Default, Debug)]
pub struct OtlpPipelineBuilder {
    exporter_config: TraceExporterConfig,
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

    /// Set the TLS settings for the collector endpoint.
    #[cfg(all(feature = "tonic", feature = "tls"))]
    pub fn with_tls_config(mut self, tls_config: ClientTlsConfig) -> Self {
        self.exporter_config.tls_config = Some(tls_config);
        self
    }

    /// Set the credentials to use when communicating with the collector.
    #[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
    pub fn with_credentials(mut self, credentials: Credentials) -> Self {
        self.exporter_config.credentials = Some(credentials);
        self
    }

    /// Set custom metadata entries to send to the collector.
    #[cfg(feature = "tonic")]
    pub fn with_metadata(mut self, metadata: MetadataMap) -> Self {
        self.exporter_config.metadata = Some(metadata);
        self
    }

    /// Set Additional headers to send to the collector.
    #[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.exporter_config.headers = Some(headers);
        self
    }

    /// Set the compression algorithm to use when communicating with the collector.
    #[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
    pub fn with_compression(mut self, compression: Compression) -> Self {
        self.exporter_config.compression = Some(compression);
        self
    }

    /// Set the timeout to the collector.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.exporter_config.timeout = timeout;
        self
    }

    /// Set the number of GRPC worker threads to poll queues.
    #[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
    pub fn with_completion_queue_count(mut self, count: usize) -> Self {
        self.exporter_config.completion_queue_count = count;
        self
    }

    /// Set the trace provider configuration.
    pub fn with_trace_config(mut self, trace_config: sdk::trace::Config) -> Self {
        self.trace_config = Some(trace_config);
        self
    }

    /// Install the OTLP exporter pipeline with the recommended defaults.
    #[cfg(feature = "tonic")]
    pub fn install(mut self) -> Result<(sdk::trace::Tracer, Uninstall), TraceError> {
        let exporter = TraceExporter::new(self.exporter_config)?;

        let mut provider_builder = sdk::trace::TracerProvider::builder().with_exporter(exporter);
        if let Some(config) = self.trace_config.take() {
            provider_builder = provider_builder.with_config(config);
        }
        let provider = provider_builder.build();
        let tracer = provider.get_tracer("opentelemetry-otlp", Some(env!("CARGO_PKG_VERSION")));
        let provider_guard = global::set_tracer_provider(provider);

        Ok((tracer, Uninstall(provider_guard)))
    }

    /// Install the OTLP exporter pipeline with the recommended defaults.
    #[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
    pub fn install(mut self) -> Result<(sdk::trace::Tracer, Uninstall), TraceError> {
        let exporter = TraceExporter::new(self.exporter_config);

        let mut provider_builder = sdk::trace::TracerProvider::builder().with_exporter(exporter);
        if let Some(config) = self.trace_config.take() {
            provider_builder = provider_builder.with_config(config);
        }
        let provider = provider_builder.build();
        let tracer = provider.get_tracer("opentelemetry-otlp", Some(env!("CARGO_PKG_VERSION")));
        let provider_guard = global::set_tracer_provider(provider);

        Ok((tracer, Uninstall(provider_guard)))
    }
}

/// Uninstalls the OTLP pipeline on drop
#[must_use]
#[derive(Debug)]
pub struct Uninstall(global::TracerProviderGuard);

/// Wrap type for errors from opentelemetry otel
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Error from tonic::transport::Error
    #[cfg(feature = "tonic")]
    #[error("transport error {0}")]
    Transport(#[from] tonic::transport::Error),

    /// Error from tonic::codegen::http::uri::InvalidUri
    #[cfg(feature = "tonic")]
    #[error("invalid URI {0}")]
    InvalidUri(#[from] tonic::codegen::http::uri::InvalidUri),

    /// Error from tonic::Status
    #[cfg(feature = "tonic")]
    #[error("status error {0}")]
    Status(#[from] tonic::Status),

    /// Error from grpcio module
    #[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
    #[error("grpcio error {0}")]
    Grpcio(#[from] grpcio::Error),
}

impl ExportError for Error {
    fn exporter_name(&self) -> &'static str {
        "otlp"
    }
}

/// The communication protocol to use when sending data.
#[derive(Clone, Copy, Debug)]
pub enum Protocol {
    /// GRPC protocol
    Grpc,
    // TODO add support for other protocols
    // HttpJson,
    // HttpProto,
}
