//! # Logs Configuration module
//!
//! This module defines the configuration structures for Logs telemetry
//! used in OpenTelemetry SDKs.

pub mod exporters;
pub mod processor_config;

use opentelemetry_sdk::logs::LogExporter;
use serde::Deserialize;
use serde_yaml::Value;

use crate::logs::{
    exporters::{
        otlp_batch_exporter::OtlpLogsBatchExporterFactory,
        stdout_batch_exporter::StdoutLogsBatchExporterFactory,
    },
    processor_config::ProcessorConfig,
};

/// Configuration for Logs telemetry
#[derive(Deserialize)]
pub struct LogsConfig {
    pub processors: Vec<ProcessorConfig>,
}

/// Factory trait implemented by the different logs exporters
pub trait LogsBatchExporterFactory {
    /// Creates a LogsExporter based on the provided configuration
    fn create_logs_batch_exporter(
        &self,
        config: &Value,
    ) -> Result<impl LogExporter + 'static, Box<dyn std::error::Error>>;
}

/// Enum representing different Batch Exporter Factories
pub enum BatchExporterFactory {
    Stdout(StdoutLogsBatchExporterFactory),
    Otlp(OtlpLogsBatchExporterFactory),
}

impl BatchExporterFactory {
    /// Creates a Stdout Logs Batch Exporter Factory
    pub fn stdout() -> Self {
        BatchExporterFactory::Stdout(StdoutLogsBatchExporterFactory::new())
    }
    /// Creates an OTLP Logs Batch Exporter Factory
    pub fn otlp() -> Self {
        BatchExporterFactory::Otlp(OtlpLogsBatchExporterFactory::new())
    }

    // Get factory by name (useful for configuration-driven creation)
    pub fn from_name(name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        match name {
            "stdout" => Ok(Self::stdout()),
            "otlp" => Ok(Self::otlp()),
            _ => Err(format!("Unknown logs exporter factory: {}", name).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logs_batch_exporter_factory_from_name() {
        let stdout_factory = BatchExporterFactory::from_name("stdout").unwrap();
        let otlp_factory = BatchExporterFactory::from_name("otlp").unwrap();
        let unknown_factory = BatchExporterFactory::from_name("unknown");

        assert!(matches!(stdout_factory, BatchExporterFactory::Stdout(_)));
        assert!(matches!(otlp_factory, BatchExporterFactory::Otlp(_)));
        assert!(unknown_factory.is_err());
    }
}
