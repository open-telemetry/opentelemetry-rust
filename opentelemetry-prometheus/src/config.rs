use core::fmt;
use opentelemetry_sdk::metrics::{ManualReaderBuilder, MetricResult};
use std::sync::Arc;

use crate::{ExporterConfig, PrometheusExporter, ResourceSelector};

/// [PrometheusExporter] configuration options
#[derive(Default)]
pub struct ExporterBuilder {
    disable_target_info: bool,
    without_units: bool,
    without_counter_suffixes: bool,
    namespace: Option<String>,
    disable_scope_info: bool,
    reader: ManualReaderBuilder,
    resource_selector: ResourceSelector,
}

impl fmt::Debug for ExporterBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExporterBuilder")
            .field("disable_target_info", &self.disable_target_info)
            .field("without_units", &self.without_units)
            .field("without_counter_suffixes", &self.without_counter_suffixes)
            .field("namespace", &self.namespace)
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

    /// Disables exporter's addition `_total` suffixes on counters.
    ///
    /// By default, metric names include a `_total` suffix to follow Prometheus
    /// naming conventions. For example, the counter metric `happy.people` would
    /// become `happy_people_total`. With this option set, the name would instead be
    /// `happy_people`.
    pub fn without_counter_suffixes(mut self) -> Self {
        self.without_counter_suffixes = true;
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

    /// Configures the exporter to prefix metrics with the given namespace.
    ///
    /// Metrics such as `target_info` and `otel_scope_info` are not prefixed since
    /// these have special behavior based on their name.
    pub fn with_namespace(mut self, namespace: impl Into<String>) -> Self {
        let mut namespace = namespace.into();

        // namespace and metric names should be separated with an underscore,
        // adds a trailing underscore if there is not one already.
        if !namespace.ends_with('_') {
            namespace.push('_')
        }

        self.namespace = Some(namespace);
        self
    }

    /// Configures whether to export resource as attributes with every metric.
    ///
    /// Note that this is orthogonal to the `target_info` metric, which can be disabled using `without_target_info`.
    ///
    /// If you called `without_target_info` and `with_resource_selector` with `ResourceSelector::None`, resource will not be exported at all.
    pub fn with_resource_selector(
        mut self,
        resource_selector: impl Into<ResourceSelector>,
    ) -> Self {
        self.resource_selector = resource_selector.into();
        self
    }

    /// Creates a new [PrometheusExporter] from this configuration.
    pub fn build(self) -> MetricResult<PrometheusExporter> {
        let reader = Arc::new(self.reader.build());

        let config = Arc::new(ExporterConfig {
            disable_target_info: self.disable_target_info,
            without_units: self.without_units,
            without_counter_suffixes: self.without_counter_suffixes,
            disable_scope_info: self.disable_scope_info,
            namespace: self.namespace,
            resource_selector: self.resource_selector,
        });

        Ok(PrometheusExporter { reader, config })
    }
}
