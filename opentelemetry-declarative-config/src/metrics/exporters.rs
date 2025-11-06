//! # Metrics Exporters module.
//!
//! This module contains the definitions and common implementations for various metrics exporters
//! that can be used with OpenTelemetry SDKs. Exporters are responsible for sending
//! collected metrics data to different backends or systems.

use serde::{Deserialize, Deserializer};

pub mod otlp_periodic_exporter;
pub mod prometheus_pull_exporter;
pub mod stdout_periodic_exporter;

/// Deserializes an optional Temporality from a string
pub fn deserialize_temporality<'de, D>(
    deserializer: D,
) -> Result<Option<opentelemetry_sdk::metrics::Temporality>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt_string = Option::<String>::deserialize(deserializer)?;
    match opt_string {
        Some(s) => match s.trim().to_lowercase().as_str() {
            "cumulative" => Ok(Some(opentelemetry_sdk::metrics::Temporality::Cumulative)),
            "delta" => Ok(Some(opentelemetry_sdk::metrics::Temporality::Delta)),
            _ => Err(serde::de::Error::custom(format!(
                "Invalid temporality: {}",
                s
            ))),
        },
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Deserialize)]
    struct MockTemporalityHolder {
        #[serde(default, deserialize_with = "deserialize_temporality")]
        pub temporality: Option<opentelemetry_sdk::metrics::Temporality>,
    }

    #[test]
    fn test_deserialize_cumulative_temporality() {
        let yaml_cumulative = r#"
            temporality: cumulative
        "#;
        let temporality_holder: MockTemporalityHolder =
            serde_yaml::from_str(yaml_cumulative).unwrap();
        assert_eq!(
            temporality_holder.temporality,
            Some(opentelemetry_sdk::metrics::Temporality::Cumulative)
        );
    }

    #[test]
    fn test_deserialize_delta_temporality() {
        let yaml_cumulative = r#"
            temporality: delta
        "#;
        let temporality_holder: MockTemporalityHolder =
            serde_yaml::from_str(yaml_cumulative).unwrap();
        assert_eq!(
            temporality_holder.temporality,
            Some(opentelemetry_sdk::metrics::Temporality::Delta)
        );
    }

    #[test]
    fn test_deserialize_unknown_temporality() {
        let yaml_cumulative = r#"
            temporality: unknown
        "#;
        let temporality_holder_result =
            serde_yaml::from_str::<MockTemporalityHolder>(yaml_cumulative);
        if let Err(e) = temporality_holder_result {
            let err_msg = e.to_string();
            assert!(err_msg.contains("Invalid temporality"));
        } else {
            panic!("Expected error for unknown temporality");
        }
    }

    #[test]
    fn test_deserialize_no_temporality() {
        let yaml_cumulative = r#"
        "#;
        let temporality_holder: MockTemporalityHolder =
            serde_yaml::from_str(yaml_cumulative).unwrap();
        assert_eq!(temporality_holder.temporality, None);
    }
}
