//! # Prometheus metrics pull exporter module.
//!
//! Prometheus is not currently maintained in the OpenTelemetry Rust SDK.
//! This module provides a factory for creating a mock Prometheus metrics pull exporter
//! that can be used for configuration parsing purposes.

use std::{sync::Weak, time::Duration};

use opentelemetry_sdk::{
    error::OTelSdkResult,
    metrics::{data::ResourceMetrics, reader::MetricReader, InstrumentKind, Pipeline, Temporality},
};
use serde::Deserialize;
use serde_yaml::Value;

use crate::metrics::MetricsPullExporterFactory;

pub struct PrometheusMetricsPullExporterFactory {}

impl PrometheusMetricsPullExporterFactory {
    pub fn new() -> Self {
        PrometheusMetricsPullExporterFactory {}
    }
}

impl Default for PrometheusMetricsPullExporterFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsPullExporterFactory for PrometheusMetricsPullExporterFactory {
    type Exporter = MockPrometheusExporter;
    fn create_metrics_pull_exporter(
        &self,
        _config: &Value,
    ) -> Result<Self::Exporter, Box<dyn std::error::Error>> {
        Err(Box::new(std::io::Error::other(
            "Prometheus exporter is not maintained currently",
        )))
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct PrometheusConfig {
    pub host: Option<String>,
    pub port: Option<u16>,
}

/// As prometheus exporter is not maintained, implementing a mock version for parsing purposes
#[derive(Debug)]
pub struct MockPrometheusExporter {
    _config: PrometheusConfig,
}

impl MetricReader for MockPrometheusExporter {
    fn register_pipeline(&self, _pipeline: Weak<Pipeline>) {}

    fn collect(&self, _rm: &mut ResourceMetrics) -> OTelSdkResult {
        Ok(())
    }

    fn force_flush(&self) -> OTelSdkResult {
        Ok(())
    }

    fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
        Ok(())
    }

    /// shutdown with default timeout
    fn shutdown(&self) -> OTelSdkResult {
        self.shutdown_with_timeout(Duration::from_secs(5))
    }

    fn temporality(&self, _kind: InstrumentKind) -> Temporality {
        Temporality::Cumulative
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_prometheus_metrics_pull_exporter_factory() {
        let factory = PrometheusMetricsPullExporterFactory::default();
        if let Err(e) = factory.create_metrics_pull_exporter(&Value::Null) {
            let err_msg = e.to_string();
            assert!(err_msg.contains("not maintained"));
        } else {
            panic!("Expected error for unmaintained Prometheus exporter");
        }
    }

    #[test]
    fn test_deserialize_config_all_fields_in() {
        let yaml_grpc = r#"        
            host: localhost
            port: 9090
        "#;

        let config: PrometheusConfig = serde_yaml::from_str(yaml_grpc).unwrap();
        assert_eq!(config.host, Some("localhost".into()));
        assert_eq!(config.port, Some(9090));
    }

    #[test]
    fn test_deserialize_config_no_fields_in() {
        let yaml_grpc = r#"        
        "#;

        let config: PrometheusConfig = serde_yaml::from_str(yaml_grpc).unwrap();
        assert_eq!(config.host, None);
    }

    #[test]
    fn test_deserialize_config_extra_fields_in() {
        let yaml_grpc = r#"        
            unknown: field
        "#;

        let config_result: Result<PrometheusConfig, _> = serde_yaml::from_str(yaml_grpc);
        assert!(config_result.is_err());
        if let Err(e) = config_result {
            let err_msg = e.to_string();
            assert!(err_msg.contains("unknown field"));
        }
    }
}
