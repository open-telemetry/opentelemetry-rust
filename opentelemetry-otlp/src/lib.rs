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
//! $ docker run -p 55680:55680 otel/opentelemetry-collector-dev:latest
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
//! ## Performance
//!
//! For optimal performance, a batch exporter is recommended as the simple
//! exporter will export each span synchronously on drop. You can enable the
//! [`tokio`] or [`async-std`] features to have a batch exporter configured for
//! you automatically for either executor when you install the pipeline.
//!
//! ```toml
//! [dependencies]
//! opentelemetry = { version = "*", features = ["tokio"] }
//! opentelemetry-otlp = "*"
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
//! ```no_run
//! use opentelemetry::{KeyValue, trace::Tracer};
//! use opentelemetry::sdk::{trace::{self, IdGenerator, Sampler}, Resource};
//! use opentelemetry_otlp::{Compression, Credentials, Protocol};
//! use std::time::Duration;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
//!     let headers = vec![("X-Custom".to_string(), "Custom-Value".to_string())]
//!         .into_iter()
//!         .collect();
//!
//!     let (tracer, _uninstall) = opentelemetry_otlp::new_pipeline()
//!         .with_endpoint("localhost:55680")
//!         .with_protocol(Protocol::Grpc)
//!         .with_headers(headers)
//!         .with_compression(Compression::Gzip)
//!         .with_timeout(Duration::from_secs(3))
//!         .with_completion_queue_count(2)
//!         .with_credentials(Credentials {
//!             cert: "tls.cert".to_string(),
//!             key: "tls.key".to_string(),
//!         })
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
//! ## Feature flags
//!
//! By default `opentelemetry-otlp` uses `boringssl` for grpc crypto. You can switch
//! this to use `openssl` by enabling the `openssl` feature:
//!
//! ```toml
//! [dependencies]
//! opentelemetry-otlp = { version = "*", features = ["openssl"] }
//! ```
//!
//! If you would like to use a vendored `openssl` version, use the `openssl-vendored` feature.
//! For more info, see https://github.com/tikv/grpc-rs#feature-openssl-and-openssl-vendored.
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
use std::collections::HashMap;
use std::time::Duration;

#[allow(clippy::all, unreachable_pub, dead_code)]
#[rustfmt::skip]
mod proto;
mod span;
mod transform;

pub use crate::span::{Compression, Credentials, Exporter, ExporterConfig, Protocol};
use opentelemetry::exporter::ExportError;
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

    /// Set the credentials to use when communicating with the collector.
    pub fn with_credentials(mut self, credentials: Credentials) -> Self {
        self.exporter_config.credentials = Some(credentials);
        self
    }

    /// Set Additional headers to send to the collector.
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.exporter_config.headers = Some(headers);
        self
    }

    /// Set the compression algorithm to use when communicating with the collector.
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
    pub fn install(mut self) -> Result<(sdk::trace::Tracer, Uninstall), TraceError> {
        let exporter = Exporter::new(self.exporter_config);

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
#[derive(Debug)]
pub struct Uninstall(global::TracerProviderGuard);

/// Wrap type for errors from opentelemetry otel
#[derive(thiserror::Error, Debug)]
pub enum Error {
    // FIXME: wait until https://github.com/open-telemetry/opentelemetry-rust/pull/352 merged
    /// Error from grpcio module
    #[error("grpcio error {0}")]
    Grpcio(#[from] grpcio::Error),
}

impl ExportError for Error {
    fn exporter_name(&self) -> &'static str {
        "otlp"
    }
}
