//! # Otlp metrics periodic exporter module.
//!
//! This module contains the definitions and common implementations for the OTLP metrics periodic exporter
//! that can be used with OpenTelemetry SDKs. Exporters are responsible for sending
//! collected metrics data to different backends or systems.

use opentelemetry_otlp::{MetricExporter, WithExportConfig};
use opentelemetry_sdk::metrics::exporter::PushMetricExporter;

use serde::{Deserialize, Deserializer};
use serde_yaml::Value;

use crate::metrics::exporters::deserialize_temporality;
use crate::metrics::MetricsPeriodicExporterFactory;

/// Factory for creating OTLP Metrics Periodic Exporters
pub struct OtlpMetricsPeriodicExporterFactory {}

impl OtlpMetricsPeriodicExporterFactory {
    pub fn new() -> Self {
        OtlpMetricsPeriodicExporterFactory {}
    }
}

impl Default for OtlpMetricsPeriodicExporterFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsPeriodicExporterFactory for OtlpMetricsPeriodicExporterFactory {
    fn create_metrics_periodic_exporter(
        &self,
        config_raw: &Value,
    ) -> Result<impl PushMetricExporter, Box<dyn std::error::Error>> {
        let exporter_builder = MetricExporter::builder();

        #[cfg(feature = "tonic-client")]
        let mut exporter_builder = exporter_builder.with_tonic();
        #[cfg(not(feature = "tonic-client"))]
        #[cfg(any(
            feature = "hyper-client",
            feature = "reqwest-client",
            feature = "reqwest-blocking-client"
        ))]
        let mut exporter_builder = exporter_builder.with_http();

        let config_parsed = serde_yaml::from_value::<OtlpConfig>(config_raw.clone())?;

        if let Some(temporality) = config_parsed.temporality {
            exporter_builder = exporter_builder.with_temporality(temporality);
        }

        if let Some(protocol) = config_parsed.protocol {
            exporter_builder = exporter_builder.with_protocol(protocol);
        }

        if let Some(endpoint) = config_parsed.endpoint {
            exporter_builder = exporter_builder.with_endpoint(&endpoint);
        }

        let exporter = exporter_builder.build()?;

        Ok(exporter)
    }
}

/// Configuration for OTLP Metrics Exporter
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OtlpConfig {
    #[serde(default, deserialize_with = "deserialize_temporality")]
    pub temporality: Option<opentelemetry_sdk::metrics::Temporality>,

    #[serde(default, deserialize_with = "deserialize_protocol")]
    pub protocol: Option<opentelemetry_otlp::Protocol>,

    #[serde(default)]
    pub endpoint: Option<String>,
}

fn deserialize_protocol<'de, D>(
    deserializer: D,
) -> Result<Option<opentelemetry_otlp::Protocol>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.trim().to_lowercase().as_str() {
        "grpc" => Ok(Some(opentelemetry_otlp::Protocol::Grpc)),
        "http/protobuf" => Ok(Some(opentelemetry_otlp::Protocol::HttpBinary)),
        "http/json" => Ok(Some(opentelemetry_otlp::Protocol::HttpJson)),
        _ => Err(serde::de::Error::custom(format!("Invalid protocol: {}", s))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_otlp_metrics_periodic_exporter_factory() {
        let factory = OtlpMetricsPeriodicExporterFactory::default();
        assert!(factory
            .create_metrics_periodic_exporter(&Value::Null)
            .is_ok());
    }

    #[test]
    fn test_deserialize_config_all_fields_in() {
        let yaml_grpc = r#"        
            protocol: grpc
            endpoint: https://backend:4318
            temporality: cumulative
        "#;

        let config: OtlpConfig = serde_yaml::from_str(yaml_grpc).unwrap();
        assert_eq!(config.protocol, Some(opentelemetry_otlp::Protocol::Grpc));
        assert_eq!(config.endpoint, Some("https://backend:4318".into()));
        assert_eq!(
            config.temporality,
            Some(opentelemetry_sdk::metrics::Temporality::Cumulative)
        );
    }

    #[test]
    fn test_deserialize_config_no_fields_in() {
        let yaml_grpc = r#"        
        "#;

        let config: OtlpConfig = serde_yaml::from_str(yaml_grpc).unwrap();
        assert_eq!(config.protocol, None);
        assert_eq!(config.endpoint, None);
        assert_eq!(config.temporality, None);
    }

    #[test]
    fn test_deserialize_config_extra_fields_in() {
        let yaml_grpc = r#"        
            protocol: grpc
            endpoint: https://backend:4318
            unknown: field
        "#;

        let config_result = serde_yaml::from_str::<OtlpConfig>(yaml_grpc);
        assert!(config_result.is_err());
        if let Err(e) = config_result {
            let err_msg = e.to_string();
            assert!(err_msg.contains("unknown field"));
        }
    }
}
