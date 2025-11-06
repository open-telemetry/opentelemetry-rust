//! # Metrics Configuration module
//!
//! This module defines the configuration structures for Metrics telemetry
//! used in OpenTelemetry SDKs.

pub mod exporters;
pub mod reader_config;

use opentelemetry_sdk::metrics::{exporter::PushMetricExporter, reader::MetricReader};
use serde::Deserialize;
use serde_yaml::Value;

use crate::metrics::{
    exporters::{
        otlp_periodic_exporter::OtlpMetricsPeriodicExporterFactory,
        prometheus_pull_exporter::PrometheusMetricsPullExporterFactory,
        stdout_periodic_exporter::StdoutMetricsPeriodicExporterFactory,
    },
    reader_config::ReaderConfig,
};

/// Configuration for Metrics telemetry
#[derive(Deserialize)]
pub struct MetricsConfig {
    pub readers: Vec<ReaderConfig>,
}

/// Factory trait implemented by the different periodic metric exporters
pub trait MetricsPeriodicExporterFactory {
    /// Creates a PushMetricExporter based on the provided configuration
    fn create_metrics_periodic_exporter(
        &self,
        config: &Value,
    ) -> Result<impl PushMetricExporter, Box<dyn std::error::Error>>;
}

/// Factory trait implemented by the different pull metric exporters
pub trait MetricsPullExporterFactory {
    type Exporter: MetricReader;
    /// Creates a MetricReader based on the provided configuration
    fn create_metrics_pull_exporter(
        &self,
        config: &Value,
    ) -> Result<Self::Exporter, Box<dyn std::error::Error>>;
}

/// Factory enum to create different types of exporters
pub enum PeriodicExporterFactory {
    Stdout(StdoutMetricsPeriodicExporterFactory),
    Otlp(OtlpMetricsPeriodicExporterFactory),
    //TODO: Add other variants as needed
}

impl PeriodicExporterFactory {
    /// Creates a Stdout Metrics Periodic Exporter Factory
    pub fn stdout() -> Self {
        PeriodicExporterFactory::Stdout(StdoutMetricsPeriodicExporterFactory::new())
    }

    /// Creates an OTLP Metrics Periodic Exporter Factory
    pub fn otlp() -> Self {
        PeriodicExporterFactory::Otlp(OtlpMetricsPeriodicExporterFactory::new())
    }

    // Get factory by name (useful for configuration-driven creation)
    pub fn from_name(name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        match name {
            "stdout" => Ok(Self::stdout()),
            "otlp" => Ok(Self::otlp()),
            _ => Err(format!("Unknown exporter factory: {}", name).into()),
        }
    }
}

/// Factory enum to create different types of pull exporters
pub enum PullExporterFactory {
    Prometheus(PrometheusMetricsPullExporterFactory),
}

impl PullExporterFactory {
    /// Creates a Prometheus Metrics Pull Exporter Factory
    pub fn prometheus() -> Self {
        PullExporterFactory::Prometheus(PrometheusMetricsPullExporterFactory::new())
    }

    // Get factory by name (useful for configuration-driven creation)
    pub fn from_name(name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        match name {
            "prometheus" => Ok(Self::prometheus()),
            _ => Err(format!("Unknown pull exporter factory: {}", name).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_periodic_exporter_factory_from_name() {
        let stdout_factory = PeriodicExporterFactory::from_name("stdout").unwrap();
        let otlp_factory = PeriodicExporterFactory::from_name("otlp").unwrap();
        let unknown_factory = PeriodicExporterFactory::from_name("unknown");
        assert!(matches!(stdout_factory, PeriodicExporterFactory::Stdout(_)));
        assert!(matches!(otlp_factory, PeriodicExporterFactory::Otlp(_)));
        assert!(unknown_factory.is_err());
    }

    #[test]
    fn test_pull_exporter_factory_from_name() {
        let prometheus_factory = PullExporterFactory::from_name("prometheus").unwrap();
        let unknown_factory = PullExporterFactory::from_name("unknown");
        assert!(matches!(
            prometheus_factory,
            PullExporterFactory::Prometheus(_)
        ));
        assert!(unknown_factory.is_err());
    }
}
