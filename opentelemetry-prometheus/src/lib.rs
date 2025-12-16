//! An OpenTelemetry exporter for [Prometheus] metrics.
//!
//! This crate provides a simple exporter that converts OpenTelemetry metrics
//! to the Prometheus exposition format (text-based). It does not require any
//! external Prometheus dependencies and generates the text format directly.
//!
//! [Prometheus]: https://prometheus.io
//!
//! # Example
//!
//! ```
//! use opentelemetry::{metrics::MeterProvider, KeyValue};
//! use opentelemetry_sdk::metrics::SdkMeterProvider;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//!
//! // configure OpenTelemetry with the Prometheus exporter
//! let exporter = opentelemetry_prometheus::exporter().build()?;
//!
//! // set up a meter to create instruments
//! let provider = SdkMeterProvider::builder().with_reader(exporter.clone()).build();
//! let meter = provider.meter("my-app");
//!
//! // Use two instruments
//! let counter = meter
//!     .u64_counter("a.counter")
//!     .with_description("Counts things")
//!     .build();
//! let histogram = meter
//!     .u64_histogram("a.histogram")
//!     .with_description("Records values")
//!     .build();
//!
//! counter.add(100, &[KeyValue::new("key", "value")]);
//! histogram.record(100, &[KeyValue::new("key", "value")]);
//!
//! // Export metrics in Prometheus exposition format
//! let exported = exporter.export()?;
//! println!("{}", exported);
//!
//! // Output contains metrics in Prometheus format:
//! //
//! // # HELP a_counter_total Counts things
//! // # TYPE a_counter_total counter
//! // a_counter_total{key="value",otel_scope_name="my-app"} 100
//! // # HELP a_histogram Records values
//! // # TYPE a_histogram histogram
//! // a_histogram_bucket{key="value",otel_scope_name="my-app",le="0"} 0
//! // a_histogram_bucket{key="value",otel_scope_name="my-app",le="5"} 0
//! // ...
//! // a_histogram_bucket{key="value",otel_scope_name="my-app",le="+Inf"} 1
//! // a_histogram_sum{key="value",otel_scope_name="my-app"} 100
//! // a_histogram_count{key="value",otel_scope_name="my-app"} 1
//! // # HELP otel_scope_info Instrumentation Scope metadata
//! // # TYPE otel_scope_info gauge
//! // otel_scope_info{otel_scope_name="my-app"} 1
//! // # HELP target_info Target metadata
//! // # TYPE target_info gauge
//! // target_info{service_name="unknown_service"} 1
//! # Ok(())
//! # }
//! ```
#![warn(
    future_incompatible,
    missing_debug_implementations,
    missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    unreachable_pub,
    unused
)]
#![cfg_attr(
    docsrs,
    feature(doc_cfg, doc_auto_cfg),
    deny(rustdoc::broken_intra_doc_links)
)]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo.svg"
)]
#![cfg_attr(test, deny(warnings))]

use once_cell::sync::Lazy;
use opentelemetry_sdk::{
    error::OTelSdkResult,
    metrics::{
        data::{self, ResourceMetrics},
        reader::MetricReader,
        InstrumentKind, ManualReader, MetricResult, Pipeline, Temporality,
    },
    Resource,
};
use std::{
    any::TypeId,
    collections::BTreeMap,
    fmt,
    sync::{Arc, Weak},
};

const TARGET_INFO_NAME: &str = "target_info";
const TARGET_INFO_DESCRIPTION: &str = "Target metadata";

const SCOPE_INFO_METRIC_NAME: &str = "otel_scope_info";
const SCOPE_INFO_DESCRIPTION: &str = "Instrumentation Scope metadata";

// Type alias for metrics grouped by name with their type, labels, and metric data
type MetricsByName<'a> =
    std::collections::BTreeMap<String, Vec<(String, Vec<(String, String)>, &'a data::Metric)>>;

// prometheus counters MUST have a _total suffix by default:
// https://github.com/open-telemetry/opentelemetry-specification/blob/v1.20.0/specification/compatibility/prometheus_and_openmetrics.md
const COUNTER_SUFFIX: &str = "_total";

mod config;
mod exposition;
mod resource_selector;
mod utils;

pub use config::ExporterBuilder;
pub use exposition::ExpositionEncoder;
pub use resource_selector::ResourceSelector;

/// Creates a builder to configure a [PrometheusExporter]
pub fn exporter() -> ExporterBuilder {
    ExporterBuilder::default()
}

/// Prometheus metrics exporter
///
/// This exporter converts OpenTelemetry metrics to the Prometheus exposition format.
#[derive(Clone, Debug)]
pub struct PrometheusExporter {
    reader: Arc<ManualReader>,
    config: Arc<ExporterConfig>,
}

#[derive(Debug)]
struct ExporterConfig {
    disable_target_info: bool,
    without_units: bool,
    without_counter_suffixes: bool,
    disable_scope_info: bool,
    namespace: Option<String>,
    resource_selector: ResourceSelector,
}

impl PrometheusExporter {
    /// Exports metrics in Prometheus exposition format.
    ///
    /// This method collects the current metrics and encodes them as a String
    /// in the Prometheus exposition format.
    pub fn export(&self) -> MetricResult<String> {
        let mut metrics = ResourceMetrics {
            resource: Resource::builder_empty().build(),
            scope_metrics: vec![],
        };

        self.reader.collect(&mut metrics)?;

        Ok(self.encode_metrics(&metrics))
    }

    /// Builds scope labels from an InstrumentationScope.
    fn build_scope_labels(scope: &opentelemetry::InstrumentationScope) -> Vec<(String, String)> {
        let mut labels = Vec::new();
        labels.push(("otel_scope_name".to_string(), scope.name().to_string()));
        if let Some(version) = scope.version() {
            labels.push(("otel_scope_version".to_string(), version.to_string()));
        }
        if let Some(schema_url) = scope.schema_url() {
            labels.push(("otel_scope_schema_url".to_string(), schema_url.to_string()));
        }
        // Add custom scope attributes with otel_scope_ prefix
        for kv in scope.attributes() {
            let attr_name = format!("otel_scope_{}", kv.key.as_str());
            labels.push((attr_name, kv.value.to_string()));
        }
        labels
    }

    /// Builds labels from data point attributes combined with extra labels.
    fn build_data_point_labels<'a, I>(
        attributes: I,
        extra_labels: &[(String, String)],
    ) -> Vec<(String, String)>
    where
        I: Iterator<Item = (&'a opentelemetry::Key, &'a opentelemetry::Value)>,
    {
        let mut labels = exposition::otel_kv_to_labels(attributes);
        labels.extend_from_slice(extra_labels);
        labels
    }

    /// Checks if a Sum metric is monotonic
    fn is_sum_monotonic(data: &dyn std::any::Any) -> bool {
        if let Some(v) = data.downcast_ref::<data::Sum<i64>>() {
            v.is_monotonic
        } else if let Some(v) = data.downcast_ref::<data::Sum<u64>>() {
            v.is_monotonic
        } else if let Some(v) = data.downcast_ref::<data::Sum<f64>>() {
            v.is_monotonic
        } else {
            false
        }
    }

    /// Extracts the otel_scope_name value from a label set for sorting
    fn extract_scope_name(labels: &[(String, String)]) -> Option<&String> {
        labels
            .iter()
            .find(|(k, _)| k == "otel_scope_name")
            .map(|(_, v)| v)
    }

    /// Compares two label sets by their otel_scope_name for sorting
    fn compare_by_scope_name(a: &[(String, String)], b: &[(String, String)]) -> std::cmp::Ordering {
        Self::extract_scope_name(a).cmp(&Self::extract_scope_name(b))
    }

    /// Encodes metrics to Prometheus exposition format.
    fn encode_metrics(&self, metrics: &ResourceMetrics) -> String {
        let resource_labels = self.config.resource_selector.select(&metrics.resource);

        // Collect all metrics grouped by name for sorting
        let mut metrics_by_name: MetricsByName<'_> = BTreeMap::new();
        let mut scope_info_labels = Vec::new();

        for scope_metrics in &metrics.scope_metrics {
            let mut scope_labels = Vec::new();

            if !self.config.disable_scope_info {
                // Collect scope_info labels if there are attributes
                if scope_metrics.scope.attributes().count() > 0 {
                    scope_info_labels.push(Self::build_scope_labels(&scope_metrics.scope));
                }

                // Add scope labels to all metrics
                scope_labels = Self::build_scope_labels(&scope_metrics.scope);
                scope_labels.extend(resource_labels.iter().cloned());
            }

            // Collect metrics grouped by name
            for metric in &scope_metrics.metrics {
                if let Some((metric_type, name)) = self.metric_type_and_name(metric) {
                    metrics_by_name.entry(name.clone()).or_default().push((
                        metric_type.to_string(),
                        scope_labels.clone(),
                        metric,
                    ));
                }
            }
        }

        // Now encode in sorted order
        let mut encoder = ExpositionEncoder::with_capacity(8192);

        // Encode regular metrics in alphabetical order
        for (name, mut metric_entries) in metrics_by_name {
            if name == SCOPE_INFO_METRIC_NAME || name == TARGET_INFO_NAME {
                continue; // Skip these, we'll handle them separately
            }

            // Sort entries by scope labels for stable output order
            metric_entries.sort_by(|a, b| Self::compare_by_scope_name(&a.1, &b.1));

            // Use the first entry for HELP and TYPE
            if let Some((metric_type, _, first_metric)) = metric_entries.first() {
                let first_description = &first_metric.description;

                // Check for conflicts and filter entries
                let mut valid_entries = Vec::new();
                for (entry_type, extra_labels, metric) in &metric_entries {
                    // Check for type conflict
                    if entry_type != metric_type {
                        #[cfg(feature = "internal-logs")]
                        opentelemetry::otel_warn!(
                            name: "MetricValidationFailed",
                            message = "Instrument type conflict, using existing type definition",
                            metric_name = name.as_str(),
                            existing_type = metric_type,
                            dropped_type = entry_type.as_str(),
                        );
                        continue;
                    }
                    // Check for description conflict
                    if &metric.description != first_description {
                        #[cfg(feature = "internal-logs")]
                        opentelemetry::otel_warn!(
                            name: "MetricValidationFailed",
                            message = "Instrument description conflict, using existing",
                            metric_name = name.as_str(),
                            existing_description = first_description.to_string(),
                            dropped_description = metric.description.to_string(),
                        );
                    }
                    valid_entries.push((extra_labels, metric));
                }

                if !valid_entries.is_empty() {
                    encoder.encode_metric_header(&name, first_description, metric_type);

                    // Encode all valid data points for this metric
                    for (extra_labels, metric) in valid_entries {
                        self.encode_metric_data(&mut encoder, metric, &name, extra_labels);
                    }
                }
            }
        }

        // Encode scope_info if we have any
        if !self.config.disable_scope_info && !scope_info_labels.is_empty() {
            // Sort scope_info labels by scope name for stable output
            scope_info_labels.sort_by(|a, b| Self::compare_by_scope_name(a, b));

            encoder.encode_metric_header(SCOPE_INFO_METRIC_NAME, SCOPE_INFO_DESCRIPTION, "gauge");
            for labels in scope_info_labels {
                encoder.encode_sample(SCOPE_INFO_METRIC_NAME, &labels, 1.0);
            }
        }

        // Encode target_info last
        if !self.config.disable_target_info && !metrics.resource.is_empty() {
            encode_target_info(&mut encoder, &metrics.resource);
        }

        encoder.finish()
    }

    fn encode_metric_data(
        &self,
        encoder: &mut ExpositionEncoder,
        metric: &data::Metric,
        name: &str,
        extra_labels: &[(String, String)],
    ) {
        let data = metric.data.as_any();

        // Try to encode as histogram (i64, u64, f64)
        if try_encode_as::<data::Histogram<i64>>(data, |h| encode_histogram(encoder, h, name, extra_labels))
            || try_encode_as::<data::Histogram<u64>>(data, |h| encode_histogram(encoder, h, name, extra_labels))
            || try_encode_as::<data::Histogram<f64>>(data, |h| encode_histogram(encoder, h, name, extra_labels))
            // Try to encode as sum (u64, i64, f64)
            || try_encode_as::<data::Sum<u64>>(data, |s| encode_sum(encoder, s, name, extra_labels))
            || try_encode_as::<data::Sum<i64>>(data, |s| encode_sum(encoder, s, name, extra_labels))
            || try_encode_as::<data::Sum<f64>>(data, |s| encode_sum(encoder, s, name, extra_labels))
            // Try to encode as gauge (u64, i64, f64)
            || try_encode_as::<data::Gauge<u64>>(data, |g| encode_gauge(encoder, g, name, extra_labels))
            || try_encode_as::<data::Gauge<i64>>(data, |g| encode_gauge(encoder, g, name, extra_labels))
            || try_encode_as::<data::Gauge<f64>>(data, |g| encode_gauge(encoder, g, name, extra_labels))
        {
            // Successfully encoded
        }
    }

    fn metric_type_and_name(&self, m: &data::Metric) -> Option<(&'static str, String)> {
        let mut name = self.get_name(m);

        let data = m.data.as_any();
        let type_id = data.type_id();

        if HISTOGRAM_TYPES.contains(&type_id) {
            Some(("histogram", name))
        } else if GAUGE_TYPES.contains(&type_id) {
            Some(("gauge", name))
        } else if SUM_TYPES.contains(&type_id) {
            let is_monotonic = Self::is_sum_monotonic(data);

            if is_monotonic {
                if !self.config.without_counter_suffixes {
                    name = format!("{name}{COUNTER_SUFFIX}");
                }
                Some(("counter", name))
            } else {
                Some(("gauge", name))
            }
        } else {
            None
        }
    }

    fn get_name(&self, m: &data::Metric) -> String {
        let name = utils::sanitize_name(&m.name);
        let unit_suffixes = if self.config.without_units {
            None
        } else {
            utils::get_unit_suffixes(&m.unit)
        };
        match (&self.config.namespace, unit_suffixes) {
            (Some(namespace), Some(suffix)) => format!("{namespace}{name}_{suffix}"),
            (Some(namespace), None) => format!("{namespace}{name}"),
            (None, Some(suffix)) => format!("{name}_{suffix}"),
            (None, None) => name.into_owned(),
        }
    }
}

impl MetricReader for PrometheusExporter {
    fn register_pipeline(&self, pipeline: Weak<Pipeline>) {
        self.reader.register_pipeline(pipeline)
    }

    fn collect(&self, rm: &mut ResourceMetrics) -> MetricResult<()> {
        self.reader.collect(rm)
    }

    fn force_flush(&self) -> OTelSdkResult {
        self.reader.force_flush()
    }

    fn shutdown(&self) -> OTelSdkResult {
        self.reader.shutdown()
    }

    /// Note: Prometheus only supports cumulative temporality, so this will always be
    /// [Temporality::Cumulative].
    fn temporality(&self, _kind: InstrumentKind) -> Temporality {
        Temporality::Cumulative
    }
}

// TODO: Remove lazy and switch to pattern matching once `TypeId` is stable in
// const context: https://github.com/rust-lang/rust/issues/77125
static HISTOGRAM_TYPES: Lazy<[TypeId; 3]> = Lazy::new(|| {
    [
        TypeId::of::<data::Histogram<i64>>(),
        TypeId::of::<data::Histogram<u64>>(),
        TypeId::of::<data::Histogram<f64>>(),
    ]
});
static SUM_TYPES: Lazy<[TypeId; 3]> = Lazy::new(|| {
    [
        TypeId::of::<data::Sum<i64>>(),
        TypeId::of::<data::Sum<u64>>(),
        TypeId::of::<data::Sum<f64>>(),
    ]
});
static GAUGE_TYPES: Lazy<[TypeId; 3]> = Lazy::new(|| {
    [
        TypeId::of::<data::Gauge<i64>>(),
        TypeId::of::<data::Gauge<u64>>(),
        TypeId::of::<data::Gauge<f64>>(),
    ]
});

/// Helper function to try downcasting and encoding a metric type.
/// Returns true if the downcast succeeded and the function was called.
fn try_encode_as<T: 'static>(data: &dyn std::any::Any, f: impl FnOnce(&T)) -> bool {
    if let Some(value) = data.downcast_ref::<T>() {
        f(value);
        true
    } else {
        false
    }
}

/// Encodes target_info metric
fn encode_target_info(encoder: &mut ExpositionEncoder, resource: &Resource) {
    encoder.encode_metric_header(TARGET_INFO_NAME, TARGET_INFO_DESCRIPTION, "gauge");

    let labels = exposition::otel_kv_to_labels(resource.iter());
    encoder.encode_sample(TARGET_INFO_NAME, &labels, 1.0);
}

/// Encodes a histogram metric
fn encode_histogram<T: Numeric>(
    encoder: &mut ExpositionEncoder,
    histogram: &data::Histogram<T>,
    name: &str,
    extra_labels: &[(String, String)],
) {
    for dp in &histogram.data_points {
        let labels = PrometheusExporter::build_data_point_labels(
            dp.attributes.iter().map(|kv| (&kv.key, &kv.value)),
            extra_labels,
        );

        // Encode buckets
        let mut cumulative_count = 0u64;
        for (i, bound) in dp.bounds.iter().enumerate() {
            cumulative_count += dp.bucket_counts[i];
            encoder.encode_histogram_bucket(name, &labels, *bound, cumulative_count);
        }

        // Add +Inf bucket
        cumulative_count += dp.bucket_counts.get(dp.bounds.len()).copied().unwrap_or(0);
        encoder.encode_histogram_bucket(name, &labels, f64::INFINITY, cumulative_count);

        // Encode sum and count
        encoder.encode_histogram_sum(name, &labels, dp.sum.as_f64());
        encoder.encode_histogram_count(name, &labels, dp.count);
    }
}

/// Encodes a sum metric
fn encode_sum<T: Numeric>(
    encoder: &mut ExpositionEncoder,
    sum: &data::Sum<T>,
    name: &str,
    extra_labels: &[(String, String)],
) {
    for dp in &sum.data_points {
        let labels = PrometheusExporter::build_data_point_labels(
            dp.attributes.iter().map(|kv| (&kv.key, &kv.value)),
            extra_labels,
        );

        encoder.encode_sample(name, &labels, dp.value.as_f64());
    }
}

/// Encodes a gauge metric
fn encode_gauge<T: Numeric>(
    encoder: &mut ExpositionEncoder,
    gauge: &data::Gauge<T>,
    name: &str,
    extra_labels: &[(String, String)],
) {
    for dp in &gauge.data_points {
        let labels = PrometheusExporter::build_data_point_labels(
            dp.attributes.iter().map(|kv| (&kv.key, &kv.value)),
            extra_labels,
        );

        encoder.encode_sample(name, &labels, dp.value.as_f64());
    }
}

trait Numeric: fmt::Debug {
    // lossy at large values for u64 and i64 but prometheus only handles floats
    fn as_f64(&self) -> f64;
}

impl Numeric for u64 {
    fn as_f64(&self) -> f64 {
        *self as f64
    }
}

impl Numeric for i64 {
    fn as_f64(&self) -> f64 {
        *self as f64
    }
}

impl Numeric for f64 {
    fn as_f64(&self) -> f64 {
        *self
    }
}
