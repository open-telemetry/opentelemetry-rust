//! OTLP - Log Exporter
//!
//! Defines a [LogExporter] to send logs via the OpenTelemetry Protocol (OTLP)

#[cfg(feature = "grpc-tonic")]
use crate::exporter::tonic::TonicExporterBuilder;

#[cfg(feature = "grpc-sys")]
use crate::exporter::grpcio::GrpcioExporterBuilder;

#[cfg(feature = "http-proto")]
use crate::exporter::http::HttpExporterBuilder;

use crate::{NoExporterConfig, OtlpPipeline};
use async_trait::async_trait;
use std::marker::PhantomData;
use std::{borrow::Cow, fmt::Debug};

use opentelemetry::{
    global,
    logs::{LogError, LoggerProvider},
};
use opentelemetry_sdk::{self, export::logs::LogData, logs::BatchMessage, runtime::RuntimeChannel};

/// Compression algorithm to use, defaults to none.
pub const OTEL_EXPORTER_OTLP_LOGS_COMPRESSION: &str = "OTEL_EXPORTER_OTLP_LOGS_COMPRESSION";

/// Target to which the exporter is going to send logs
pub const OTEL_EXPORTER_OTLP_LOGS_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_LOGS_ENDPOINT";

/// Maximum time the OTLP exporter will wait for each batch logs export.
pub const OTEL_EXPORTER_OTLP_LOGS_TIMEOUT: &str = "OTEL_EXPORTER_OTLP_LOGS_TIMEOUT";

impl OtlpPipeline {
    /// Create a OTLP logging pipeline.
    pub fn logging(self) -> OtlpLogPipeline<NoExporterConfig> {
        OtlpLogPipeline {
            log_config: None,
            exporter_builder: NoExporterConfig(()),
        }
    }
}

/// OTLP log exporter builder
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
#[non_exhaustive]
pub enum LogExporterBuilder {
    /// Tonic log exporter builder
    #[cfg(feature = "grpc-tonic")]
    Tonic(TonicExporterBuilder),
    /// Grpc log exporter builder
    #[cfg(feature = "grpc-sys")]
    Grpcio(GrpcioExporterBuilder),
    /// Http log exporter builder
    #[cfg(feature = "http-proto")]
    Http(HttpExporterBuilder),
}

impl LogExporterBuilder {
    /// Build a OTLP log exporter using the given configuration.
    pub fn build_log_exporter(self) -> Result<LogExporter, LogError> {
        match self {
            #[cfg(feature = "grpc-tonic")]
            LogExporterBuilder::Tonic(builder) => builder.build_log_exporter(),
            #[cfg(feature = "grpc-sys")]
            LogExporterBuilder::Grpcio(builder) => builder.build_log_exporter(),
            #[cfg(feature = "http-proto")]
            LogExporterBuilder::Http(builder) => builder.build_log_exporter(),
        }
    }
}

#[cfg(feature = "grpc-tonic")]
impl From<TonicExporterBuilder> for LogExporterBuilder {
    fn from(exporter: TonicExporterBuilder) -> Self {
        LogExporterBuilder::Tonic(exporter)
    }
}

#[cfg(feature = "grpc-sys")]
impl From<GrpcioExporterBuilder> for LogExporterBuilder {
    fn from(exporter: GrpcioExporterBuilder) -> Self {
        LogExporterBuilder::Grpcio(exporter)
    }
}

#[cfg(feature = "http-proto")]
impl From<HttpExporterBuilder> for LogExporterBuilder {
    fn from(exporter: HttpExporterBuilder) -> Self {
        LogExporterBuilder::Http(exporter)
    }
}

/// OTLP exporter that sends log data
#[derive(Debug)]
pub struct LogExporter {
    client: Box<dyn opentelemetry_sdk::export::logs::LogExporter>,
}

impl LogExporter {
    /// Create a new log exporter
    pub fn new(client: impl opentelemetry_sdk::export::logs::LogExporter + 'static) -> Self {
        LogExporter {
            client: Box::new(client),
        }
    }
}

#[async_trait]
impl opentelemetry_sdk::export::logs::LogExporter for LogExporter {
    async fn export(&mut self, batch: Vec<LogData>) -> opentelemetry::logs::LogResult<()> {
        self.client.export(batch).await
    }
}

/// Recommended configuration for an OTLP exporter pipeline.
#[derive(Debug)]
pub struct OtlpLogPipeline<EB> {
    exporter_builder: EB,
    log_config: Option<opentelemetry_sdk::logs::Config>,
}

impl<EB> OtlpLogPipeline<EB> {
    /// Set the log provider configuration.
    pub fn with_log_config(mut self, log_config: opentelemetry_sdk::logs::Config) -> Self {
        self.log_config = Some(log_config);
        self
    }
}

impl OtlpLogPipeline<NoExporterConfig> {
    /// Set the OTLP log exporter builder.
    pub fn with_exporter<B: Into<LogExporterBuilder>>(
        self,
        pipeline: B,
    ) -> OtlpLogPipeline<LogExporterBuilder> {
        OtlpLogPipeline {
            exporter_builder: pipeline.into(),
            log_config: self.log_config,
        }
    }
}

impl OtlpLogPipeline<LogExporterBuilder> {
    /// Install the configured log exporter.
    ///
    /// Returns a [`Logger`] with the name `opentelemetry-otlp` and the current crate version.
    ///
    /// [`Logger`]: opentelemetry_sdk::logs::Logger
    pub fn install_simple(self) -> Result<opentelemetry_sdk::logs::Logger, LogError> {
        Ok(build_simple_with_exporter(
            self.exporter_builder.build_log_exporter()?,
            self.log_config,
        ))
    }

    /// Install the configured log exporter and a batch span processor using the
    /// specified runtime.
    ///
    /// Returns a [`Logger`] with the name `opentelemetry-otlp` and the current crate version.
    ///
    /// [`Logger`]: opentelemetry_sdk::logs::Logger
    pub fn install_batch<R: RuntimeChannel<BatchMessage>>(
        self,
        runtime: R,
    ) -> Result<opentelemetry_sdk::logs::Logger, LogError> {
        Ok(build_batch_with_exporter(
            self.exporter_builder.build_log_exporter()?,
            self.log_config,
            runtime,
        ))
    }
}

fn build_simple_with_exporter(
    exporter: LogExporter,
    log_config: Option<opentelemetry_sdk::logs::Config>,
) -> opentelemetry_sdk::logs::Logger {
    let mut provider_builder =
        opentelemetry_sdk::logs::LoggerProvider::builder().with_simple_exporter(exporter);
    if let Some(config) = log_config {
        provider_builder = provider_builder.with_config(config);
    }
    let provider = provider_builder.build();
    let logger = provider.versioned_logger(
        Cow::Borrowed("opentelemetry-otlp"),
        Some(Cow::Borrowed(env!("CARGO_PKG_VERSION"))),
        None,
        None,
    );
    let _ = global::set_logger_provider(provider);
    logger
}

fn build_batch_with_exporter<R: RuntimeChannel<BatchMessage>>(
    exporter: LogExporter,
    log_config: Option<opentelemetry_sdk::logs::Config>,
    runtime: R,
) -> opentelemetry_sdk::logs::Logger {
    let mut provider_builder =
        opentelemetry_sdk::logs::LoggerProvider::builder().with_batch_exporter(exporter, runtime);
    if let Some(config) = log_config {
        provider_builder = provider_builder.with_config(config);
    }
    let provider = provider_builder.build();
    let logger = provider.versioned_logger(
        Cow::Borrowed("opentelemetry-otlp"),
        Some(Cow::Borrowed(env!("CARGO_PKG_VERSION"))),
        None,
        None,
    );
    let _ = global::set_logger_provider(provider);
    logger
}
