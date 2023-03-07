use core::fmt;
use std::sync::{Arc, Mutex};

use once_cell::sync::OnceCell;
use opentelemetry_api::metrics::{MetricsError, Result};
use opentelemetry_sdk::metrics::{reader::AggregationSelector, ManualReaderBuilder};

use crate::{Collector, PrometheusExporter};

/// [PrometheusExporter] configuration options
#[derive(Default)]
pub struct ExporterBuilder {
    registry: Option<prometheus::Registry>,
    disable_target_info: bool,
    without_units: bool,
    aggregation: Option<Box<dyn AggregationSelector>>,
    disable_scope_info: bool,
}

impl fmt::Debug for ExporterBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExporterBuilder")
            .field("registry", &self.registry)
            .field("disable_target_info", &self.disable_target_info)
            .field("without_units", &self.without_units)
            .field("aggregation", &self.aggregation.is_some())
            .field("disable_scope_info", &self.disable_scope_info)
            .finish()
    }
}

impl ExporterBuilder {
    /// Disables exporter's addition of unit suffixes to metric names.
    ///
    /// By default, metric names include a unit suffix to follow Prometheus naming
    /// conventions. For example, the counter metric `request.duration`, with unit
    /// `ms` would become `request_duration_milliseconds_total`.
    ///
    /// With this option set, the name would instead be `request_duration_total`.
    pub fn without_units(mut self) -> Self {
        self.without_units = true;
        self
    }

    /// Configures the exporter to not export the resource `target_info` metric.
    ///
    /// If not specified, the exporter will create a `target_info` metric containing
    /// the metrics' [Resource] attributes.
    ///
    /// [Resource]: opentelemetry_sdk::Resource
    pub fn without_target_info(mut self) -> Self {
        self.disable_target_info = true;
        self
    }

    /// Configures the exporter to not export the `otel_scope_info` metric.
    ///
    /// If not specified, the exporter will create a `otel_scope_info` metric
    /// containing the metrics' Instrumentation Scope, and also add labels about
    /// Instrumentation Scope to all metric points.
    pub fn without_scope_info(mut self) -> Self {
        self.disable_scope_info = true;
        self
    }

    /// Configures which [prometheus::Registry] the exporter will use.
    ///
    /// If no registry is specified, the prometheus default is used.
    pub fn with_registry(mut self, registry: prometheus::Registry) -> Self {
        self.registry = Some(registry);
        self
    }

    /// Configure the [AggregationSelector] the exporter will use.
    ///
    /// If no selector is provided, the [DefaultAggregationSelector] is used.
    ///
    /// [DefaultAggregationSelector]: opentelemetry_sdk::metrics::reader::DefaultAggregationSelector
    pub fn with_aggregation_selector(mut self, agg: impl AggregationSelector + 'static) -> Self {
        self.aggregation = Some(Box::new(agg));
        self
    }

    /// Creates a new [PrometheusExporter] from this configuration.
    pub fn build(self) -> Result<PrometheusExporter> {
        let mut reader = ManualReaderBuilder::new();
        if let Some(selector) = self.aggregation {
            reader = reader.with_aggregation_selector(selector)
        }
        let reader = Arc::new(reader.build());

        let collector = Collector {
            reader: Arc::clone(&reader),
            disable_target_info: self.disable_target_info,
            without_units: self.without_units,
            disable_scope_info: self.disable_scope_info,
            create_target_info_once: OnceCell::new(),
            inner: Mutex::new(Default::default()),
        };

        let registry = self.registry.unwrap_or_else(prometheus::Registry::new);
        registry
            .register(Box::new(collector))
            .map_err(|e| MetricsError::Other(e.to_string()))?;

        Ok(PrometheusExporter { reader })
    }
}
