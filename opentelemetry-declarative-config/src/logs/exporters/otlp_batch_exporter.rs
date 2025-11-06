//! # OTLP log exporter module.
//!
//! This module contains the definitions and common implementations for the OTLP logs exporter
//! that can be used with OpenTelemetry SDKs. Exporters are responsible for sending
//! collected logs data to OTLP-compatible backends or systems.

use opentelemetry_sdk::logs::LogExporter;
use serde::{Deserialize, Deserializer};
use serde_yaml::Value;

use crate::logs::LogsBatchExporterFactory;

/// Factory for creating OTLP Logs Batch Exporters
pub struct OtlpLogsBatchExporterFactory {}

impl OtlpLogsBatchExporterFactory {
    /// Creates a new OtlpLogsBatchExporterFactory
    pub fn new() -> Self {
        OtlpLogsBatchExporterFactory {}
    }
}

impl Default for OtlpLogsBatchExporterFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl LogsBatchExporterFactory for OtlpLogsBatchExporterFactory {
    /// Creates a LogExporter based on the provided configuration
    fn create_logs_batch_exporter(
        &self,
        _config: &Value,
    ) -> Result<impl LogExporter + 'static, Box<dyn std::error::Error>> {
        let exporter_builder = opentelemetry_otlp::LogExporter::builder();

        #[cfg(feature = "tonic-client")]
        let exporter_builder = exporter_builder.with_tonic();
        #[cfg(not(feature = "tonic-client"))]
        #[cfg(any(
            feature = "hyper-client",
            feature = "reqwest-client",
            feature = "reqwest-blocking-client"
        ))]
        let exporter_builder = exporter_builder.with_http();

        /*
        let config_parsed = serde_yaml::from_value::<OtlpConfig>(config.clone())?;
        */

        // TODO: Configure the exporter based on config_parsed fields. There are no methods in the builder to pass its parameters yet..

        let exporter = exporter_builder.build()?;

        Ok(exporter)
    }
}

/// Configuration for OTLP Logs Exporter
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OtlpConfig {
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
    use serde_yaml::from_str;

    #[tokio::test]
    async fn test_default_otlp_logs_batch_exporter_factory() {
        let factory = OtlpLogsBatchExporterFactory::default();
        assert!(factory.create_logs_batch_exporter(&Value::Null).is_ok());
    }

    #[test]
    fn test_deserialize_otlp_config() {
        let yaml_data = r#"
            protocol: "grpc"
            endpoint: "http://localhost:4317"
        "#;
        let config: OtlpConfig = from_str(yaml_data).unwrap();
        assert_eq!(config.protocol, Some(opentelemetry_otlp::Protocol::Grpc));
        assert_eq!(config.endpoint, Some("http://localhost:4317".into()));
    }

    #[test]
    fn test_deserialize_protocol() {
        let yaml_grpc = r#"
            protocol: "grpc"
        "#;
        let config: OtlpConfig = from_str(yaml_grpc).unwrap();
        assert_eq!(config.protocol, Some(opentelemetry_otlp::Protocol::Grpc));

        let yaml_http_binary = r#"
            protocol: "http/protobuf"
        "#;
        let config: OtlpConfig = from_str(yaml_http_binary).unwrap();
        assert_eq!(
            config.protocol,
            Some(opentelemetry_otlp::Protocol::HttpBinary)
        );

        let yaml_http_json = r#"
            protocol: "http/json"
        "#;
        let config: OtlpConfig = from_str(yaml_http_json).unwrap();
        assert_eq!(
            config.protocol,
            Some(opentelemetry_otlp::Protocol::HttpJson)
        );

        let yaml_unknown = r#"
            protocol: "http/unknown"
        "#;
        let config: Result<OtlpConfig, _> = from_str(yaml_unknown);
        assert!(config.is_err());
    }
}
