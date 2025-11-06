//! # Stdout logs exporter module.
//!
//! This module contains the definitions and common implementations for the stdout logs exporter
//! that can be used with OpenTelemetry SDKs. Exporters are responsible for sending
//! collected logs data to different backends or systems.

use opentelemetry_sdk::logs::LogExporter;
use serde_yaml::Value;

use crate::logs::LogsBatchExporterFactory;

/// Factory for creating Stdout Logs Batch Exporters
pub struct StdoutLogsBatchExporterFactory {}

impl StdoutLogsBatchExporterFactory {
    /// Creates a new StdoutLogsBatchExporterFactory
    pub fn new() -> Self {
        StdoutLogsBatchExporterFactory {}
    }
}

impl Default for StdoutLogsBatchExporterFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl LogsBatchExporterFactory for StdoutLogsBatchExporterFactory {
    /// Creates a LogExporter based on the provided configuration
    fn create_logs_batch_exporter(
        &self,
        _config: &Value,
    ) -> Result<impl LogExporter + 'static, Box<dyn std::error::Error>> {
        let exporter = opentelemetry_stdout::LogExporter::default();
        Ok(exporter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_stdout_logs_batch_exporter_factory() {
        let factory = StdoutLogsBatchExporterFactory::default();
        assert!(factory.create_logs_batch_exporter(&Value::Null).is_ok());
    }
}
