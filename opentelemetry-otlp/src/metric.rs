//! OTEL metric exporter
//!
//! Defines a [MetricExporter] to send metric data to backend via OTLP protocol.
//!

#[cfg(any(feature = "http-proto", feature = "http-json", feature = "grpc-tonic"))]
use crate::HasExportConfig;

#[cfg(any(feature = "http-proto", feature = "http-json"))]
use crate::{exporter::http::HttpExporterBuilder, HasHttpConfig, HttpExporterBuilderSet};

#[cfg(feature = "grpc-tonic")]
use crate::{exporter::tonic::TonicExporterBuilder, HasTonicConfig, TonicExporterBuilderSet};

use crate::NoExporterBuilderSet;

use async_trait::async_trait;
use core::fmt;
use opentelemetry_sdk::error::ShutdownResult;
use opentelemetry_sdk::metrics::MetricResult;

use opentelemetry_sdk::metrics::{
    data::ResourceMetrics, exporter::PushMetricExporter, Temporality,
};
use std::fmt::{Debug, Formatter};

/// Target to which the exporter is going to send metrics, defaults to https://localhost:4317/v1/metrics.
/// Learn about the relationship between this constant and default/spans/logs at
/// <https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/protocol/exporter.md#endpoint-urls-for-otlphttp>
pub const OTEL_EXPORTER_OTLP_METRICS_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_METRICS_ENDPOINT";
/// Max waiting time for the backend to process each metrics batch, defaults to 10s.
pub const OTEL_EXPORTER_OTLP_METRICS_TIMEOUT: &str = "OTEL_EXPORTER_OTLP_METRICS_TIMEOUT";
/// Compression algorithm to use, defaults to none.
pub const OTEL_EXPORTER_OTLP_METRICS_COMPRESSION: &str = "OTEL_EXPORTER_OTLP_METRICS_COMPRESSION";
/// Key-value pairs to be used as headers associated with gRPC or HTTP requests
/// for sending metrics.
/// Example: `k1=v1,k2=v2`
/// Note: this is only supported for HTTP.
pub const OTEL_EXPORTER_OTLP_METRICS_HEADERS: &str = "OTEL_EXPORTER_OTLP_METRICS_HEADERS";

#[derive(Debug, Default, Clone)]
pub struct MetricExporterBuilder<C> {
    client: C,
    temporality: Temporality,
}

impl MetricExporterBuilder<NoExporterBuilderSet> {
    pub fn new() -> Self {
        MetricExporterBuilder::default()
    }
}

impl<C> MetricExporterBuilder<C> {
    #[cfg(feature = "grpc-tonic")]
    pub fn with_tonic(self) -> MetricExporterBuilder<TonicExporterBuilderSet> {
        MetricExporterBuilder {
            client: TonicExporterBuilderSet(TonicExporterBuilder::default()),
            temporality: self.temporality,
        }
    }

    #[cfg(any(feature = "http-proto", feature = "http-json"))]
    pub fn with_http(self) -> MetricExporterBuilder<HttpExporterBuilderSet> {
        MetricExporterBuilder {
            client: HttpExporterBuilderSet(HttpExporterBuilder::default()),
            temporality: self.temporality,
        }
    }

    pub fn with_temporality(self, temporality: Temporality) -> MetricExporterBuilder<C> {
        MetricExporterBuilder {
            client: self.client,
            temporality,
        }
    }
}

#[cfg(feature = "grpc-tonic")]
impl MetricExporterBuilder<TonicExporterBuilderSet> {
    pub fn build(self) -> MetricResult<MetricExporter> {
        let exporter = self.client.0.build_metrics_exporter(self.temporality)?;
        opentelemetry::otel_debug!(name: "MetricExporterBuilt");
        Ok(exporter)
    }
}

#[cfg(any(feature = "http-proto", feature = "http-json"))]
impl MetricExporterBuilder<HttpExporterBuilderSet> {
    pub fn build(self) -> MetricResult<MetricExporter> {
        let exporter = self.client.0.build_metrics_exporter(self.temporality)?;
        Ok(exporter)
    }
}

#[cfg(feature = "grpc-tonic")]
impl HasExportConfig for MetricExporterBuilder<TonicExporterBuilderSet> {
    fn export_config(&mut self) -> &mut crate::ExportConfig {
        &mut self.client.0.exporter_config
    }
}

#[cfg(any(feature = "http-proto", feature = "http-json"))]
impl HasExportConfig for MetricExporterBuilder<HttpExporterBuilderSet> {
    fn export_config(&mut self) -> &mut crate::ExportConfig {
        &mut self.client.0.exporter_config
    }
}

#[cfg(feature = "grpc-tonic")]
impl HasTonicConfig for MetricExporterBuilder<TonicExporterBuilderSet> {
    fn tonic_config(&mut self) -> &mut crate::TonicConfig {
        &mut self.client.0.tonic_config
    }
}

#[cfg(any(feature = "http-proto", feature = "http-json"))]
impl HasHttpConfig for MetricExporterBuilder<HttpExporterBuilderSet> {
    fn http_client_config(&mut self) -> &mut crate::exporter::http::HttpConfig {
        &mut self.client.0.http_config
    }
}

/// An interface for OTLP metrics clients
#[async_trait]
pub(crate) trait MetricsClient: fmt::Debug + Send + Sync + 'static {
    async fn export(&self, metrics: &mut ResourceMetrics) -> MetricResult<()>;
    fn shutdown(&self) -> ShutdownResult;
}

/// Export metrics in OTEL format.
pub struct MetricExporter {
    client: Box<dyn MetricsClient>,
    temporality: Temporality,
}

impl Debug for MetricExporter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetricExporter").finish()
    }
}

#[async_trait]
impl PushMetricExporter for MetricExporter {
    async fn export(&self, metrics: &mut ResourceMetrics) -> MetricResult<()> {
        self.client.export(metrics).await
    }

    async fn force_flush(&self) -> MetricResult<()> {
        // this component is stateless
        Ok(())
    }

    fn shutdown(&self) -> ShutdownResult {
        self.client.shutdown()
    }

    fn temporality(&self) -> Temporality {
        self.temporality
    }
}

impl MetricExporter {
    /// Obtain a builder to configure a [MetricExporter].
    pub fn builder() -> MetricExporterBuilder<NoExporterBuilderSet> {
        MetricExporterBuilder::default()
    }

    /// Create a new metrics exporter
    pub(crate) fn new(client: impl MetricsClient, temporality: Temporality) -> MetricExporter {
        MetricExporter {
            client: Box::new(client),
            temporality,
        }
    }
}
