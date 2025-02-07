//! # OTLP - Span Exporter
//!
//! Defines a [SpanExporter] to send trace data via the OpenTelemetry Protocol (OTLP)

use std::fmt::Debug;

use futures_core::future::BoxFuture;
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::trace::SpanData;

#[cfg(feature = "grpc-tonic")]
use crate::{
    exporter::tonic::{HasTonicConfig, TonicExporterBuilder},
    TonicExporterBuilderSet,
};

#[cfg(any(feature = "http-proto", feature = "http-json"))]
use crate::{
    exporter::http::{HasHttpConfig, HttpExporterBuilder},
    HttpExporterBuilderSet,
};

use crate::{exporter::HasExportConfig, NoExporterBuilderSet};

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

#[derive(Debug, Default, Clone)]
pub struct SpanExporterBuilder<C> {
    client: C,
}

impl SpanExporterBuilder<NoExporterBuilderSet> {
    pub fn new() -> Self {
        SpanExporterBuilder::default()
    }

    #[cfg(feature = "grpc-tonic")]
    pub fn with_tonic(self) -> SpanExporterBuilder<TonicExporterBuilderSet> {
        SpanExporterBuilder {
            client: TonicExporterBuilderSet(TonicExporterBuilder::default()),
        }
    }

    #[cfg(any(feature = "http-proto", feature = "http-json"))]
    pub fn with_http(self) -> SpanExporterBuilder<HttpExporterBuilderSet> {
        SpanExporterBuilder {
            client: HttpExporterBuilderSet(HttpExporterBuilder::default()),
        }
    }
}

#[cfg(feature = "grpc-tonic")]
impl SpanExporterBuilder<TonicExporterBuilderSet> {
    pub fn build(self) -> Result<SpanExporter, opentelemetry::trace::TraceError> {
        let span_exporter = self.client.0.build_span_exporter()?;
        opentelemetry::otel_debug!(name: "SpanExporterBuilt");
        Ok(SpanExporter::new(span_exporter))
    }
}

#[cfg(any(feature = "http-proto", feature = "http-json"))]
impl SpanExporterBuilder<HttpExporterBuilderSet> {
    pub fn build(self) -> Result<SpanExporter, opentelemetry::trace::TraceError> {
        let span_exporter = self.client.0.build_span_exporter()?;
        Ok(SpanExporter::new(span_exporter))
    }
}

#[cfg(feature = "grpc-tonic")]
impl HasExportConfig for SpanExporterBuilder<TonicExporterBuilderSet> {
    fn export_config(&mut self) -> &mut crate::ExportConfig {
        &mut self.client.0.exporter_config
    }
}

#[cfg(any(feature = "http-proto", feature = "http-json"))]
impl HasExportConfig for SpanExporterBuilder<HttpExporterBuilderSet> {
    fn export_config(&mut self) -> &mut crate::ExportConfig {
        &mut self.client.0.exporter_config
    }
}

#[cfg(feature = "grpc-tonic")]
impl HasTonicConfig for SpanExporterBuilder<TonicExporterBuilderSet> {
    fn tonic_config(&mut self) -> &mut crate::TonicConfig {
        &mut self.client.0.tonic_config
    }
}

#[cfg(any(feature = "http-proto", feature = "http-json"))]
impl HasHttpConfig for SpanExporterBuilder<HttpExporterBuilderSet> {
    fn http_client_config(&mut self) -> &mut crate::exporter::http::HttpConfig {
        &mut self.client.0.http_config
    }
}

/// OTLP exporter that sends tracing information
#[derive(Debug)]
pub struct SpanExporter(Box<dyn opentelemetry_sdk::trace::SpanExporter>);

impl SpanExporter {
    /// Obtain a builder to configure a [SpanExporter].
    pub fn builder() -> SpanExporterBuilder<NoExporterBuilderSet> {
        SpanExporterBuilder::default()
    }

    /// Build a new span exporter from a client
    pub fn new(client: impl opentelemetry_sdk::trace::SpanExporter + 'static) -> Self {
        SpanExporter(Box::new(client))
    }
}

impl opentelemetry_sdk::trace::SpanExporter for SpanExporter {
    fn export(&mut self, batch: Vec<SpanData>) -> BoxFuture<'static, OTelSdkResult> {
        self.0.export(batch)
    }

    fn set_resource(&mut self, resource: &opentelemetry_sdk::Resource) {
        self.0.set_resource(resource);
    }
}
