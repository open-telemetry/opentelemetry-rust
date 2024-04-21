//! # OTLP - Span Exporter
//!
//! Defines a [SpanExporter] to send trace data via the OpenTelemetry Protocol (OTLP)

use std::fmt::Debug;

use futures_core::future::BoxFuture;
use opentelemetry::{
    global,
    trace::{TraceError, TracerProvider},
};
use opentelemetry_sdk::{
    self as sdk,
    export::trace::{ExportResult, SpanData},
};
use opentelemetry_semantic_conventions::SCHEMA_URL;
use sdk::runtime::RuntimeChannel;

#[cfg(feature = "grpc-tonic")]
use crate::exporter::tonic::TonicExporterBuilder;

#[cfg(any(feature = "http-proto", feature = "http-json"))]
use crate::exporter::http::HttpExporterBuilder;

use crate::{NoExporterConfig, OtlpPipeline};

/// Target to which the exporter is going to send spans, defaults to https://localhost:4317/v1/traces.
/// Learn about the relationship between this constant and default/metrics/logs at
/// <https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/protocol/exporter.md#endpoint-urls-for-otlphttp>
pub const OTEL_EXPORTER_OTLP_TRACES_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT";
/// Max waiting time for the backend to process each spans batch, defaults to 10s.
pub const OTEL_EXPORTER_OTLP_TRACES_TIMEOUT: &str = "OTEL_EXPORTER_OTLP_TRACES_TIMEOUT";
/// Compression algorithm to use, defaults to none.
pub const OTEL_EXPORTER_OTLP_TRACES_COMPRESSION: &str = "OTEL_EXPORTER_OTLP_TRACES_COMPRESSION";
/// Key-value pairs to be used as headers associated with gRPC or HTTP requests
/// for sending spans.
/// Example: `k1=v1,k2=v2`
/// Note: this is only supported for HTTP.
pub const OTEL_EXPORTER_OTLP_TRACES_HEADERS: &str = "OTEL_EXPORTER_OTLP_TRACES_HEADERS";

impl OtlpPipeline {
    /// Create a OTLP tracing pipeline.
    pub fn tracing(self) -> OtlpTracePipeline<NoExporterConfig> {
        OtlpTracePipeline {
            exporter_builder: NoExporterConfig(()),
            trace_config: None,
            batch_config: None,
        }
    }
}

/// Recommended configuration for an OTLP exporter pipeline.
///
/// ## Examples
///
/// ```no_run
/// let tracing_pipeline = opentelemetry_otlp::new_pipeline().tracing();
/// ```
#[derive(Debug)]
pub struct OtlpTracePipeline<EB> {
    exporter_builder: EB,
    trace_config: Option<sdk::trace::Config>,
    batch_config: Option<sdk::trace::BatchConfig>,
}

impl<EB> OtlpTracePipeline<EB> {
    /// Set the trace provider configuration.
    pub fn with_trace_config(mut self, trace_config: sdk::trace::Config) -> Self {
        self.trace_config = Some(trace_config);
        self
    }

    /// Set the batch span processor configuration, and it will override the env vars.
    pub fn with_batch_config(mut self, batch_config: sdk::trace::BatchConfig) -> Self {
        self.batch_config = Some(batch_config);
        self
    }
}

impl OtlpTracePipeline<NoExporterConfig> {
    /// Set the OTLP span exporter builder.
    ///
    /// Note that the pipeline will not build the exporter until [`install_batch`] or [`install_simple`]
    /// is called.
    ///
    /// [`install_batch`]: OtlpTracePipeline::install_batch
    /// [`install_simple`]: OtlpTracePipeline::install_simple
    pub fn with_exporter<B: Into<SpanExporterBuilder>>(
        self,
        pipeline: B,
    ) -> OtlpTracePipeline<SpanExporterBuilder> {
        OtlpTracePipeline {
            exporter_builder: pipeline.into(),
            trace_config: self.trace_config,
            batch_config: self.batch_config,
        }
    }
}

impl OtlpTracePipeline<SpanExporterBuilder> {
    /// Install the configured span exporter.
    ///
    /// Returns a [`Tracer`] with the name `opentelemetry-otlp` and current crate version.
    ///
    /// [`Tracer`]: opentelemetry::trace::Tracer
    pub fn install_simple(self) -> Result<sdk::trace::Tracer, TraceError> {
        Ok(build_simple_with_exporter(
            self.exporter_builder.build_span_exporter()?,
            self.trace_config,
        ))
    }

    /// Install the configured span exporter and a batch span processor using the
    /// specified runtime.
    ///
    /// Returns a [`Tracer`] with the name `opentelemetry-otlp` and current crate version.
    ///
    /// `install_batch` will panic if not called within a tokio runtime
    ///
    /// [`Tracer`]: opentelemetry::trace::Tracer
    pub fn install_batch<R: RuntimeChannel>(
        self,
        runtime: R,
    ) -> Result<sdk::trace::Tracer, TraceError> {
        Ok(build_batch_with_exporter(
            self.exporter_builder.build_span_exporter()?,
            self.trace_config,
            runtime,
            self.batch_config,
        ))
    }
}

fn build_simple_with_exporter(
    exporter: SpanExporter,
    trace_config: Option<sdk::trace::Config>,
) -> sdk::trace::Tracer {
    let mut provider_builder = sdk::trace::TracerProvider::builder().with_simple_exporter(exporter);
    if let Some(config) = trace_config {
        provider_builder = provider_builder.with_config(config);
    }
    let provider = provider_builder.build();
    let tracer = provider
        .tracer_builder("opentelemetry-otlp")
        .with_version(env!("CARGO_PKG_VERSION"))
        .with_schema_url(SCHEMA_URL)
        .build();
    let _ = global::set_tracer_provider(provider);
    tracer
}

fn build_batch_with_exporter<R: RuntimeChannel>(
    exporter: SpanExporter,
    trace_config: Option<sdk::trace::Config>,
    runtime: R,
    batch_config: Option<sdk::trace::BatchConfig>,
) -> sdk::trace::Tracer {
    let mut provider_builder = sdk::trace::TracerProvider::builder();
    let batch_processor = sdk::trace::BatchSpanProcessor::builder(exporter, runtime)
        .with_batch_config(batch_config.unwrap_or_default())
        .build();
    provider_builder = provider_builder.with_span_processor(batch_processor);

    if let Some(config) = trace_config {
        provider_builder = provider_builder.with_config(config);
    }
    let provider = provider_builder.build();
    let tracer = provider
        .tracer_builder("opentelemetry-otlp")
        .with_version(env!("CARGO_PKG_VERSION"))
        .with_schema_url(SCHEMA_URL)
        .build();
    let _ = global::set_tracer_provider(provider);
    tracer
}

/// OTLP span exporter builder.
#[derive(Debug)]
// This enum only used during initialization stage of application. The overhead should be OK.
// Users can also disable the unused features to make the overhead on object size smaller.
#[allow(clippy::large_enum_variant)]
#[non_exhaustive]
pub enum SpanExporterBuilder {
    /// Tonic span exporter builder
    #[cfg(feature = "grpc-tonic")]
    Tonic(TonicExporterBuilder),
    /// Http span exporter builder
    #[cfg(any(feature = "http-proto", feature = "http-json"))]
    Http(HttpExporterBuilder),
}

impl SpanExporterBuilder {
    /// Build a OTLP span exporter using the given tonic configuration and exporter configuration.
    pub fn build_span_exporter(self) -> Result<SpanExporter, TraceError> {
        match self {
            #[cfg(feature = "grpc-tonic")]
            SpanExporterBuilder::Tonic(builder) => builder.build_span_exporter(),
            #[cfg(any(feature = "http-proto", feature = "http-json"))]
            SpanExporterBuilder::Http(builder) => builder.build_span_exporter(),
        }
    }
}

#[cfg(feature = "grpc-tonic")]
impl From<TonicExporterBuilder> for SpanExporterBuilder {
    fn from(exporter: TonicExporterBuilder) -> Self {
        SpanExporterBuilder::Tonic(exporter)
    }
}

#[cfg(any(feature = "http-proto", feature = "http-json"))]
impl From<HttpExporterBuilder> for SpanExporterBuilder {
    fn from(exporter: HttpExporterBuilder) -> Self {
        SpanExporterBuilder::Http(exporter)
    }
}

/// OTLP exporter that sends tracing information
#[derive(Debug)]
pub struct SpanExporter(Box<dyn opentelemetry_sdk::export::trace::SpanExporter>);

impl SpanExporter {
    /// Build a new span exporter from a client
    pub fn new(client: impl opentelemetry_sdk::export::trace::SpanExporter + 'static) -> Self {
        SpanExporter(Box::new(client))
    }
}

impl opentelemetry_sdk::export::trace::SpanExporter for SpanExporter {
    fn export(&mut self, batch: Vec<SpanData>) -> BoxFuture<'static, ExportResult> {
        self.0.export(batch)
    }
}
