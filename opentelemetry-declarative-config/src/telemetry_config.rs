//! # Telemetry Configuration module
//!
//! This module defines the configuration structures for telemetry
//! used in OpenTelemetry SDKs.

use std::{collections::HashMap, error::Error};

use serde::Deserialize;

use crate::{logs::LogsConfig, metrics::MetricsConfig};

/// Configuration for Telemetry
#[derive(Deserialize)]
pub struct TelemetryConfig {
    /// Metrics telemetry configuration
    pub metrics: Option<MetricsConfig>,

    /// Logs telemetry configuration
    pub logs: Option<LogsConfig>,

    /// Resource attributes to be associated with all telemetry data
    #[serde(default)]
    pub resource: HashMap<String, String>,
}

impl TelemetryConfig {
    pub fn new() -> Self {
        TelemetryConfig {
            metrics: None,
            logs: None,
            resource: HashMap::new(),
        }
    }

    /// Creates a TelemetryConfig from a YAML string
    pub fn from_yaml(yaml_str: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config: TelemetryConfig = serde_yaml::from_str(yaml_str)?;
        Ok(config)
    }

    /// Creates a TelemetryConfig from a YAML file
    pub fn from_yaml_file(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let yaml_str = std::fs::read_to_string(file_path)?;
        Self::from_yaml(&yaml_str)
    }
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_telemetry_config_from_yaml() {
        let yaml_str = r#"
        resource:
          service.name: test-service
        metrics:
          readers:
            - periodic:
              exporter:
                name: stdout
        logs:
          processors:
            - batch:
              exporter:
                name: otlp
        "#;
        let config = TelemetryConfig::from_yaml(yaml_str).unwrap();
        assert!(config.metrics.is_some());
        assert!(config.logs.is_some());
        assert_eq!(config.resource.get("service.name").unwrap(), "test-service");
    }

    #[test]
    fn test_telemetry_config_default() {
        let config = TelemetryConfig::default();
        assert!(config.metrics.is_none());
        assert!(config.logs.is_none());
        assert!(config.resource.is_empty());
    }
}
