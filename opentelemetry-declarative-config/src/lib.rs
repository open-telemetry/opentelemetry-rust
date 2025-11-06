//! # OpenTelemetry declarative configuration
//!
//! This crate provides a declarative way to configure OpenTelemetry SDKs using YAML files.

pub mod logs;
pub mod metrics;
pub mod telemetry_config;

use opentelemetry::global;
use opentelemetry_sdk::{
    error::OTelSdkResult, logs::SdkLoggerProvider, metrics::SdkMeterProvider,
    trace::SdkTracerProvider,
};

use crate::{
    logs::{BatchExporterFactory, LogsBatchExporterFactory, LogsConfig},
    metrics::{
        MetricsConfig, MetricsPeriodicExporterFactory, MetricsPullExporterFactory,
        PeriodicExporterFactory, PullExporterFactory,
    },
    telemetry_config::TelemetryConfig,
};

pub struct Configurator {}

impl Configurator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn configure_telemetry_from_yaml(
        &self,
        telemetry_config_str: String,
    ) -> Result<TelemetryProviders, Box<dyn std::error::Error>> {
        let config = TelemetryConfig::from_yaml(&telemetry_config_str)?;
        self.configure_telemetry(config)
    }

    pub fn configure_telemetry_from_yaml_file(
        &self,
        file_path: &str,
    ) -> Result<TelemetryProviders, Box<dyn std::error::Error>> {
        let config = TelemetryConfig::from_yaml_file(file_path)?;
        self.configure_telemetry(config)
    }

    pub fn configure_telemetry(
        &self,
        telemetry_config: TelemetryConfig,
    ) -> Result<TelemetryProviders, Box<dyn std::error::Error>> {
        let mut configured_telemetry_providers = TelemetryProviders::new();

        let resource = Self::as_resource(telemetry_config.resource);
        if let Some(metrics_config) = telemetry_config.metrics {
            let sdk_meter_provider_option =
                self.build_metrics_sdk_provider(metrics_config, resource.clone())?;
            configured_telemetry_providers =
                configured_telemetry_providers.with_meter_provider(sdk_meter_provider_option);
        }

        if let Some(logs_config) = telemetry_config.logs {
            let sdk_logger_provider_option =
                self.build_logs_sdk_provider(logs_config, resource.clone())?;
            configured_telemetry_providers =
                configured_telemetry_providers.with_logs_provider(sdk_logger_provider_option);
        }

        //TODO: Add similar configuration for traces when implemented
        Ok(configured_telemetry_providers)
    }

    fn build_metrics_sdk_provider(
        &self,
        metrics_config: MetricsConfig,
        resource: opentelemetry_sdk::Resource,
    ) -> Result<SdkMeterProvider, Box<dyn std::error::Error>> {
        let mut provider_builder = SdkMeterProvider::builder().with_resource(resource);

        for reader_config in metrics_config.readers {
            if let Some(periodic_reader) = reader_config.periodic {
                let periodic_exporter_config = periodic_reader.exporter;
                if let Some(periodic_exporter_mapping) = periodic_exporter_config.as_mapping() {
                    for (key, value) in periodic_exporter_mapping {
                        if let Some(periodic_exporter_factory_name) = key.as_str() {
                            let exporter_factory =
                                PeriodicExporterFactory::from_name(periodic_exporter_factory_name)?;
                            let config = value;
                            match exporter_factory {
                                PeriodicExporterFactory::Stdout(factory) => {
                                    let periodic_exporter =
                                        factory.create_metrics_periodic_exporter(config)?;
                                    provider_builder =
                                        provider_builder.with_periodic_exporter(periodic_exporter);
                                }
                                PeriodicExporterFactory::Otlp(factory) => {
                                    let periodic_exporter =
                                        factory.create_metrics_periodic_exporter(config)?;
                                    provider_builder =
                                        provider_builder.with_periodic_exporter(periodic_exporter);
                                }
                            }
                        } else {
                            return Err("Invalid periodic exporter factory configuration".into());
                        }
                    }
                } else {
                    return Err("Periodic exporter configuration must be defined".into());
                }
            }
            if let Some(pull_reader) = reader_config.pull {
                let pull_exporter_config = pull_reader.exporter;
                if let Some(pull_exporter_mapping) = pull_exporter_config.as_mapping() {
                    for (key, value) in pull_exporter_mapping {
                        if let Some(pull_exporter_factory_name) = key.as_str() {
                            let exporter_factory =
                                PullExporterFactory::from_name(pull_exporter_factory_name)?;
                            let config = value;
                            match exporter_factory {
                                PullExporterFactory::Prometheus(factory) => {
                                    let pull_exporter =
                                        factory.create_metrics_pull_exporter(config)?;
                                    provider_builder = provider_builder.with_reader(pull_exporter);
                                }
                            }
                        } else {
                            return Err("Invalid pull exporter factory configuration".into());
                        }
                    }
                } else {
                    return Err("Pull exporter configuration must be defined".into());
                }
            }
        }

        let provider = provider_builder.build();
        global::set_meter_provider(provider.clone());
        Ok(provider)
    }

    fn build_logs_sdk_provider(
        &self,
        logs_config: LogsConfig,
        resource: opentelemetry_sdk::Resource,
    ) -> Result<SdkLoggerProvider, Box<dyn std::error::Error>> {
        let mut provider_builder = SdkLoggerProvider::builder().with_resource(resource);

        for processor_config in logs_config.processors {
            if let Some(batch_processor) = processor_config.batch {
                let batch_exporter_config = batch_processor.exporter;
                if let Some(batch_exporter_mapping) = batch_exporter_config.as_mapping() {
                    for (key, value) in batch_exporter_mapping {
                        if let Some(batch_exporter_factory_name) = key.as_str() {
                            let exporter_factory =
                                BatchExporterFactory::from_name(batch_exporter_factory_name)?;
                            let config = value;
                            match exporter_factory {
                                BatchExporterFactory::Stdout(factory) => {
                                    let batch_exporter =
                                        factory.create_logs_batch_exporter(config)?;
                                    provider_builder =
                                        provider_builder.with_batch_exporter(batch_exporter);
                                }
                                BatchExporterFactory::Otlp(factory) => {
                                    let batch_exporter =
                                        factory.create_logs_batch_exporter(config)?;
                                    provider_builder =
                                        provider_builder.with_batch_exporter(batch_exporter);
                                }
                            }
                        } else {
                            return Err("Invalid batch exporter factory configuration".into());
                        }
                    }
                } else {
                    return Err("Batch exporter configuration must be defined".into());
                }
            }
        }

        let provider = provider_builder.build();
        Ok(provider)
    }

    fn as_resource(
        resource_attributes: std::collections::HashMap<String, String>,
    ) -> opentelemetry_sdk::Resource {
        let mut resource_builder = opentelemetry_sdk::Resource::builder();
        for (key, value) in resource_attributes {
            resource_builder =
                resource_builder.with_attribute(opentelemetry::KeyValue::new(key, value));
        }
        resource_builder.build()
    }
}

impl Default for Configurator {
    fn default() -> Self {
        Self::new()
    }
}

/// Holds the configured telemetry providers
pub struct TelemetryProviders {
    meter_provider: Option<SdkMeterProvider>,
    traces_provider: Option<SdkTracerProvider>,
    logs_provider: Option<SdkLoggerProvider>,
}

impl TelemetryProviders {
    pub fn new() -> Self {
        TelemetryProviders {
            meter_provider: None,
            traces_provider: None,
            logs_provider: None,
        }
    }

    pub fn with_meter_provider(mut self, meter_provider: SdkMeterProvider) -> Self {
        self.meter_provider = Some(meter_provider);
        self
    }

    pub fn with_traces_provider(mut self, traces_provider: SdkTracerProvider) -> Self {
        self.traces_provider = Some(traces_provider);
        self
    }
    pub fn with_logs_provider(mut self, logs_provider: SdkLoggerProvider) -> Self {
        self.logs_provider = Some(logs_provider);
        self
    }

    pub fn meter_provider(&self) -> Option<&SdkMeterProvider> {
        self.meter_provider.as_ref()
    }

    pub fn traces_provider(&self) -> Option<&SdkTracerProvider> {
        self.traces_provider.as_ref()
    }

    pub fn logs_provider(&self) -> Option<&SdkLoggerProvider> {
        self.logs_provider.as_ref()
    }

    pub fn shutdown(self) -> OTelSdkResult {
        if let Some(meter_provider) = self.meter_provider {
            meter_provider.shutdown()?;
        }
        if let Some(traces_provider) = self.traces_provider {
            traces_provider.shutdown()?;
        }
        if let Some(logs_provider) = self.logs_provider {
            logs_provider.shutdown()?;
        }
        Ok(())
    }
}

impl Default for TelemetryProviders {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_empty() -> Result<(), Box<dyn std::error::Error>> {
        let config = Configurator::new();
        let telemetry_config = TelemetryConfig::default();
        let telemetry_providers = config.configure_telemetry(telemetry_config)?;

        assert!(telemetry_providers.meter_provider().is_none());
        assert!(telemetry_providers.traces_provider().is_none());
        assert!(telemetry_providers.logs_provider().is_none());

        assert!(telemetry_providers.shutdown().is_ok());
        Ok(())
    }

    #[test]
    fn test_shutdown_empty_providers() {
        let providers = TelemetryProviders::new();
        assert!(providers.shutdown().is_ok());
    }

    #[test]
    fn test_configurator_default() -> Result<(), Box<dyn std::error::Error>> {
        let config = Configurator::default();
        let telemetry_config = TelemetryConfig::default();
        let telemetry_providers = config.configure_telemetry(telemetry_config)?;

        assert!(telemetry_providers.meter_provider().is_none());
        assert!(telemetry_providers.traces_provider().is_none());
        assert!(telemetry_providers.logs_provider().is_none());

        assert!(telemetry_providers.shutdown().is_ok());
        Ok(())
    }

    #[test]
    fn test_configurator_from_yaml() -> Result<(), Box<dyn std::error::Error>> {
        let config = Configurator::new();
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
        let telemetry_providers = config.configure_telemetry_from_yaml(yaml_str.into())?;

        assert!(telemetry_providers.meter_provider().is_some());
        assert!(telemetry_providers.logs_provider().is_some());
        assert!(telemetry_providers.traces_provider().is_none());

        assert!(telemetry_providers.shutdown().is_ok());
        Ok(())
    }

    #[test]
    fn test_telemetry_providers_default() {
        let providers = TelemetryProviders::default();
        assert!(providers.meter_provider().is_none());
        assert!(providers.traces_provider().is_none());
        assert!(providers.logs_provider().is_none());
    }

    #[test]
    fn test_telemetry_providers_getters() {
        let meter_provider = SdkMeterProvider::builder().build();
        let logs_provider = SdkLoggerProvider::builder().build();
        let traces_provider = SdkTracerProvider::builder().build();

        let providers = TelemetryProviders::new()
            .with_logs_provider(logs_provider)
            .with_meter_provider(meter_provider)
            .with_traces_provider(traces_provider);

        assert!(providers.meter_provider().is_some());
        assert!(providers.traces_provider().is_some());
        assert!(providers.logs_provider().is_some());
        providers.shutdown().unwrap();
    }
}
