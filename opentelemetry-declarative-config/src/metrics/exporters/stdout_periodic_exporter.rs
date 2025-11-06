//! # Stdout metrics periodic exporter module.
//!
//! This module contains the definitions and common implementations for the Stdout metrics periodic exporter
//! that can be used with OpenTelemetry SDKs. Exporters are responsible for sending
//! collected metrics data to different backends or systems.

use opentelemetry_sdk::metrics::exporter::PushMetricExporter;

use serde::Deserialize;
use serde_yaml::Value;

use crate::metrics::exporters::deserialize_temporality;
use crate::metrics::MetricsPeriodicExporterFactory;

/// Factory for creating Stdout Metrics Periodic Exporters
pub struct StdoutMetricsPeriodicExporterFactory {}

impl StdoutMetricsPeriodicExporterFactory {
    pub fn new() -> Self {
        StdoutMetricsPeriodicExporterFactory {}
    }
}

impl Default for StdoutMetricsPeriodicExporterFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsPeriodicExporterFactory for StdoutMetricsPeriodicExporterFactory {
    fn create_metrics_periodic_exporter(
        &self,
        config: &Value,
    ) -> Result<impl PushMetricExporter, Box<dyn std::error::Error>> {
        let mut exporter_builder = opentelemetry_stdout::MetricExporter::builder();

        let config_parsed = serde_yaml::from_value::<StdoutConfig>(config.clone())?;

        if let Some(temporality) = config_parsed.temporality {
            exporter_builder = exporter_builder.with_temporality(temporality);
        }

        let exporter = exporter_builder.build();
        Ok(exporter)
    }
}

/// Configuration for Stdout Metrics Exporter
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StdoutConfig {
    #[serde(default, deserialize_with = "deserialize_temporality")]
    pub temporality: Option<opentelemetry_sdk::metrics::Temporality>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_stdout_metrics_periodic_exporter_factory() {
        let factory = StdoutMetricsPeriodicExporterFactory::default();
        assert!(factory
            .create_metrics_periodic_exporter(&Value::Null)
            .is_ok());
    }

    #[test]
    fn test_deserialize_config_all_fields_in() {
        let yaml_grpc = r#"        
            temporality: cumulative
        "#;

        let config: StdoutConfig = serde_yaml::from_str(yaml_grpc).unwrap();
        assert_eq!(
            config.temporality,
            Some(opentelemetry_sdk::metrics::Temporality::Cumulative)
        );
    }

    #[test]
    fn test_deserialize_config_no_fields_in() {
        let yaml_grpc = r#"        
        "#;

        let config: StdoutConfig = serde_yaml::from_str(yaml_grpc).unwrap();
        assert_eq!(config.temporality, None);
    }

    #[test]
    fn test_deserialize_config_extra_fields_in() {
        let yaml_grpc = r#"        
            unknown: field
        "#;

        let config_result: Result<StdoutConfig, _> = serde_yaml::from_str(yaml_grpc);
        assert!(config_result.is_err());
        if let Err(e) = config_result {
            let err_msg = e.to_string();
            assert!(err_msg.contains("unknown field"));
        }
    }
}
