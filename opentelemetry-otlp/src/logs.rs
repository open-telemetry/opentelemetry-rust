//! OTLP - Log Exporter
//!
//! Defines a [LogExporter] to send logs via the OpenTelemetry Protocol (OTLP)

use async_trait::async_trait;
use std::fmt::Debug;

use opentelemetry_sdk::logs::LogResult;

use opentelemetry_sdk::export::logs::LogBatch;

use crate::{HasExportConfig, NoExporterBuilderSet};

#[cfg(feature = "grpc-tonic")]
use crate::{HasTonicConfig, TonicExporterBuilder, TonicExporterBuilderSet};

#[cfg(any(feature = "http-proto", feature = "http-json"))]
use crate::{HasHttpConfig, HttpExporterBuilder, HttpExporterBuilderSet};

/// Compression algorithm to use, defaults to none.
pub const OTEL_EXPORTER_OTLP_LOGS_COMPRESSION: &str = "OTEL_EXPORTER_OTLP_LOGS_COMPRESSION";

/// Target to which the exporter is going to send logs
pub const OTEL_EXPORTER_OTLP_LOGS_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_LOGS_ENDPOINT";

/// Maximum time the OTLP exporter will wait for each batch logs export.
pub const OTEL_EXPORTER_OTLP_LOGS_TIMEOUT: &str = "OTEL_EXPORTER_OTLP_LOGS_TIMEOUT";

/// Key-value pairs to be used as headers associated with gRPC or HTTP requests
/// for sending logs.
/// Example: `k1=v1,k2=v2`
/// Note: this is only supported for HTTP.
pub const OTEL_EXPORTER_OTLP_LOGS_HEADERS: &str = "OTEL_EXPORTER_OTLP_LOGS_HEADERS";

#[derive(Debug, Default, Clone)]
pub struct LogExporterBuilder<C> {
    client: C,
    endpoint: Option<String>,
}

impl LogExporterBuilder<NoExporterBuilderSet> {
    pub fn new() -> Self {
        LogExporterBuilder::default()
    }

    #[cfg(feature = "grpc-tonic")]
    pub fn with_tonic(self) -> LogExporterBuilder<TonicExporterBuilderSet> {
        LogExporterBuilder {
            client: TonicExporterBuilderSet(TonicExporterBuilder::default()),
            endpoint: self.endpoint,
        }
    }

    #[cfg(any(feature = "http-proto", feature = "http-json"))]
    pub fn with_http(self) -> LogExporterBuilder<HttpExporterBuilderSet> {
        LogExporterBuilder {
            client: HttpExporterBuilderSet(HttpExporterBuilder::default()),
            endpoint: self.endpoint,
        }
    }
}

#[cfg(feature = "grpc-tonic")]
impl LogExporterBuilder<TonicExporterBuilderSet> {
    pub fn build(self) -> Result<LogExporter, opentelemetry_sdk::logs::LogError> {
        self.client.0.build_log_exporter()
    }
}

#[cfg(any(feature = "http-proto", feature = "http-json"))]
impl LogExporterBuilder<HttpExporterBuilderSet> {
    pub fn build(self) -> Result<LogExporter, opentelemetry_sdk::logs::LogError> {
        self.client.0.build_log_exporter()
    }
}

#[cfg(feature = "grpc-tonic")]
impl HasExportConfig for LogExporterBuilder<TonicExporterBuilderSet> {
    fn export_config(&mut self) -> &mut crate::ExportConfig {
        &mut self.client.0.exporter_config
    }
}

#[cfg(any(feature = "http-proto", feature = "http-json"))]
impl HasExportConfig for LogExporterBuilder<HttpExporterBuilderSet> {
    fn export_config(&mut self) -> &mut crate::ExportConfig {
        &mut self.client.0.exporter_config
    }
}

#[cfg(feature = "grpc-tonic")]
impl HasTonicConfig for LogExporterBuilder<TonicExporterBuilderSet> {
    fn tonic_config(&mut self) -> &mut crate::TonicConfig {
        &mut self.client.0.tonic_config
    }
}

#[cfg(any(feature = "http-proto", feature = "http-json"))]
impl HasHttpConfig for LogExporterBuilder<HttpExporterBuilderSet> {
    fn http_client_config(&mut self) -> &mut crate::exporter::http::HttpConfig {
        &mut self.client.0.http_config
    }
}

/// OTLP exporter that sends log data
#[derive(Debug)]
pub struct LogExporter {
    client: Box<dyn opentelemetry_sdk::export::logs::LogExporter>,
}

impl LogExporter {
    /// Obtain a builder to configure a [LogExporter].
    pub fn builder() -> LogExporterBuilder<NoExporterBuilderSet> {
        LogExporterBuilder::default()
    }

    /// Create a new log exporter
    pub fn new(client: impl opentelemetry_sdk::export::logs::LogExporter + 'static) -> Self {
        LogExporter {
            client: Box::new(client),
        }
    }
}

#[async_trait]
impl opentelemetry_sdk::export::logs::LogExporter for LogExporter {
    async fn export(&mut self, batch: LogBatch<'_>) -> LogResult<()> {
        self.client.export(batch).await
    }

    fn set_resource(&mut self, resource: &opentelemetry_sdk::Resource) {
        self.client.set_resource(resource);
    }
}
