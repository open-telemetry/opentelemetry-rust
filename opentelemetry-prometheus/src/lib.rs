//! # OpenTelemetry Prometheus Exporter
//!
//! ### Prometheus Exporter Example
//!
//! ```rust
//! use opentelemetry::{global, KeyValue, sdk::Resource};
//! use opentelemetry_prometheus::PrometheusExporter;
//! use prometheus::{TextEncoder, Encoder};
//!
//! fn init_meter() -> PrometheusExporter {
//!     opentelemetry_prometheus::exporter()
//!         .with_resource(Resource::new(vec![KeyValue::new("R", "V")]))
//!         .init()
//! }
//!
//! let exporter = init_meter();
//! let meter = global::meter("my-app");
//!
//! // Use two instruments
//! let counter = meter
//!     .u64_counter("a.counter")
//!     .with_description("Counts things")
//!     .init();
//! let recorder = meter
//!     .i64_value_recorder("a.value_recorder")
//!     .with_description("Records values")
//!     .init();
//!
//! counter.add(100, &[KeyValue::new("key", "value")]);
//! recorder.record(100, &[KeyValue::new("key", "value")]);
//!
//! // Encode data as text or protobuf
//! let encoder = TextEncoder::new();
//! let metric_families = exporter.registry().gather();
//! let mut result = Vec::new();
//! encoder.encode(&metric_families, &mut result);
//!
//! // result now contains encoded metrics:
//! //
//! // # HELP a_counter Counts things
//! // # TYPE a_counter counter
//! // a_counter{R="V",key="value"} 100
//! // # HELP a_value_recorder Records values
//! // # TYPE a_value_recorder histogram
//! // a_value_recorder_bucket{R="V",key="value",le="0.5"} 0
//! // a_value_recorder_bucket{R="V",key="value",le="0.9"} 0
//! // a_value_recorder_bucket{R="V",key="value",le="0.99"} 0
//! // a_value_recorder_bucket{R="V",key="value",le="+Inf"} 1
//! // a_value_recorder_sum{R="V",key="value"} 100
//! // a_value_recorder_count{R="V",key="value"} 1
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
#![cfg_attr(docsrs, feature(doc_cfg), deny(broken_intra_doc_links))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/master/assets/logo.svg"
)]
#![cfg_attr(test, deny(warnings))]

use opentelemetry::global;
use opentelemetry::sdk::{
    export::metrics::{CheckpointSet, ExportKind, Histogram, Record, Sum},
    metrics::{
        aggregators::{HistogramAggregator, SumAggregator},
        controllers,
        selectors::simple::Selector,
        PullController,
    },
    Resource,
};
use opentelemetry::{
    labels,
    metrics::{
        registry::RegistryMeterProvider, Descriptor, InstrumentKind, MetricsError, NumberKind,
    },
    KeyValue,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

mod sanitize;

use sanitize::sanitize;

/// Cache disabled by default.
const DEFAULT_CACHE_PERIOD: Duration = Duration::from_secs(0);
const EXPORT_KIND: ExportKind = ExportKind::Cumulative;

/// Create a new prometheus exporter builder.
pub fn exporter() -> ExporterBuilder {
    ExporterBuilder::default()
}

/// Configuration for the prometheus exporter.
#[derive(Debug, Default)]
pub struct ExporterBuilder {
    /// The OpenTelemetry `Resource` associated with all Meters
    /// created by the pull controller.
    resource: Option<Resource>,

    /// The period which a recently-computed result will be returned without
    /// gathering metric data again.
    ///
    /// If the period is zero, caching of the result is disabled, which is the
    /// prometheus default.
    cache_period: Option<Duration>,

    /// The default summary quantiles to use. Use nil to specify the system-default
    /// summary quantiles.
    default_summary_quantiles: Option<Vec<f64>>,

    /// Defines the default histogram bucket boundaries.
    default_histogram_boundaries: Option<Vec<f64>>,

    /// The prometheus registry that will be used to register instruments.
    ///
    /// If not set a new empty `Registry` is created.
    registry: Option<prometheus::Registry>,
}

impl ExporterBuilder {
    /// Set the resource to be associated with all `Meter`s for this exporter
    pub fn with_resource(self, resource: Resource) -> Self {
        ExporterBuilder {
            resource: Some(resource),
            ..self
        }
    }

    /// Set the period which a recently-computed result will be returned without
    /// gathering metric data again.
    ///
    /// If the period is zero, caching of the result is disabled (default).
    pub fn with_cache_period(self, period: Duration) -> Self {
        ExporterBuilder {
            cache_period: Some(period),
            ..self
        }
    }

    /// Set the default summary quantiles to be used by exported prometheus histograms
    pub fn with_default_summary_quantiles(self, quantiles: Vec<f64>) -> Self {
        ExporterBuilder {
            default_summary_quantiles: Some(quantiles),
            ..self
        }
    }

    /// Set the default boundaries to be used by exported prometheus histograms
    pub fn with_default_histogram_boundaries(self, boundaries: Vec<f64>) -> Self {
        ExporterBuilder {
            default_histogram_boundaries: Some(boundaries),
            ..self
        }
    }

    /// Set the prometheus registry to be used by this exporter
    pub fn with_registry(self, registry: prometheus::Registry) -> Self {
        ExporterBuilder {
            registry: Some(registry),
            ..self
        }
    }

    /// Sets up a complete export pipeline with the recommended setup, using the
    /// recommended selector and standard processor.
    pub fn try_init(self) -> Result<PrometheusExporter, MetricsError> {
        let registry = self.registry.unwrap_or_else(prometheus::Registry::new);
        let default_summary_quantiles = self
            .default_summary_quantiles
            .unwrap_or_else(|| vec![0.5, 0.9, 0.99]);
        let default_histogram_boundaries = self
            .default_histogram_boundaries
            .unwrap_or_else(|| vec![0.5, 0.9, 0.99]);
        let selector = Box::new(Selector::Histogram(default_histogram_boundaries.clone()));
        let mut controller_builder = controllers::pull(selector, Box::new(EXPORT_KIND))
            .with_cache_period(self.cache_period.unwrap_or(DEFAULT_CACHE_PERIOD))
            .with_memory(true);
        if let Some(resource) = self.resource {
            controller_builder = controller_builder.with_resource(resource);
        }
        let controller = controller_builder.build();

        global::set_meter_provider(controller.provider());

        PrometheusExporter::new(
            registry,
            controller,
            default_summary_quantiles,
            default_histogram_boundaries,
        )
    }

    /// Sets up a complete export pipeline with the recommended setup, using the
    /// recommended selector and standard processor.
    ///
    /// # Panics
    ///
    /// This panics if the exporter cannot be registered in the prometheus registry.
    pub fn init(self) -> PrometheusExporter {
        self.try_init().unwrap()
    }
}

/// An implementation of `metrics::Exporter` that sends metrics to Prometheus.
///
/// This exporter supports Prometheus pulls, as such it does not
/// implement the export.Exporter interface.
#[derive(Clone, Debug)]
pub struct PrometheusExporter {
    registry: prometheus::Registry,
    controller: Arc<Mutex<PullController>>,
    default_summary_quantiles: Vec<f64>,
    default_histogram_boundaries: Vec<f64>,
}

impl PrometheusExporter {
    /// Create a new prometheus exporter
    pub fn new(
        registry: prometheus::Registry,
        controller: PullController,
        default_summary_quantiles: Vec<f64>,
        default_histogram_boundaries: Vec<f64>,
    ) -> Result<Self, MetricsError> {
        let controller = Arc::new(Mutex::new(controller));
        let collector = Collector::with_controller(controller.clone());
        registry
            .register(Box::new(collector))
            .map_err(|e| MetricsError::Other(e.to_string()))?;

        Ok(PrometheusExporter {
            registry,
            controller,
            default_summary_quantiles,
            default_histogram_boundaries,
        })
    }

    /// Returns a reference to the current prometheus registry.
    pub fn registry(&self) -> &prometheus::Registry {
        &self.registry
    }

    /// Get this exporter's provider.
    pub fn provider(&self) -> Result<RegistryMeterProvider, MetricsError> {
        self.controller
            .lock()
            .map_err(Into::into)
            .map(|locked| locked.provider())
    }

    /// Determine the export kind this exporter should use for a given instrument
    /// and descriptor.
    pub fn export_kind_for(&self, _descriptor: &Descriptor, _kind: &InstrumentKind) -> ExportKind {
        // NOTE: Summary values should use Delta aggregation, then be
        // combined into a sliding window, see the TODO below.
        // NOTE: Prometheus also supports a "GaugeDelta" exposition format,
        // which is expressed as a delta histogram.  Need to understand if this
        // should be a default behavior for ValueRecorder/ValueObserver.
        EXPORT_KIND
    }
}

#[derive(Debug)]
struct Collector {
    controller: Arc<Mutex<PullController>>,
}

impl Collector {
    fn with_controller(controller: Arc<Mutex<PullController>>) -> Self {
        Collector { controller }
    }
}

impl prometheus::core::Collector for Collector {
    /// Unused as descriptors are dynamically registered.
    fn desc(&self) -> Vec<&prometheus::core::Desc> {
        Vec::new()
    }

    /// Collect all otel metrics and convert to prometheus metrics.
    fn collect(&self) -> Vec<prometheus::proto::MetricFamily> {
        if let Ok(mut controller) = self.controller.lock() {
            let mut metrics = Vec::new();

            if let Err(err) = controller.collect() {
                global::handle_error(err);
                return metrics;
            }

            if let Err(err) = controller.try_for_each(&EXPORT_KIND, &mut |record| {
                let agg = record.aggregator().ok_or(MetricsError::NoDataCollected)?;
                let number_kind = record.descriptor().number_kind();

                let mut label_keys = Vec::new();
                let mut label_values = Vec::new();
                merge_labels(record, &mut label_keys, Some(&mut label_values));

                let desc = to_desc(&record, label_keys);

                if let Some(hist) = agg.as_any().downcast_ref::<HistogramAggregator>() {
                    metrics.push(build_histogram(hist, number_kind, desc, label_values)?);
                } else if let Some(sum) = agg.as_any().downcast_ref::<SumAggregator>() {
                    metrics.push(build_counter(sum, number_kind, desc, label_values)?);
                }

                Ok(())
            }) {
                global::handle_error(err);
            }

            metrics
        } else {
            Vec::new()
        }
    }
}

fn build_counter(
    sum: &SumAggregator,
    kind: &NumberKind,
    desc: prometheus::core::Desc,
    labels: Vec<KeyValue>,
) -> Result<prometheus::proto::MetricFamily, MetricsError> {
    let sum = sum.sum()?;

    let mut c = prometheus::proto::Counter::default();
    c.set_value(sum.to_f64(kind));

    let mut m = prometheus::proto::Metric::default();
    m.set_label(protobuf::RepeatedField::from_vec(
        labels.into_iter().map(build_label_pair).collect(),
    ));
    m.set_counter(c);

    let mut mf = prometheus::proto::MetricFamily::default();
    mf.set_name(desc.fq_name);
    mf.set_help(desc.help);
    mf.set_field_type(prometheus::proto::MetricType::COUNTER);
    mf.set_metric(protobuf::RepeatedField::from_vec(vec![m]));

    Ok(mf)
}

fn build_histogram(
    hist: &HistogramAggregator,
    kind: &NumberKind,
    desc: prometheus::core::Desc,
    labels: Vec<KeyValue>,
) -> Result<prometheus::proto::MetricFamily, MetricsError> {
    let raw_buckets = hist.histogram()?;
    let sum = hist.sum()?;

    let mut h = prometheus::proto::Histogram::default();
    h.set_sample_sum(sum.to_f64(kind));

    let mut count = 0;
    let mut buckets = Vec::with_capacity(raw_buckets.boundaries().len());
    for (i, upper_bound) in raw_buckets.boundaries().iter().enumerate() {
        count += raw_buckets.counts()[i] as u64;
        let mut b = prometheus::proto::Bucket::default();
        b.set_cumulative_count(count);
        b.set_upper_bound(*upper_bound);
        buckets.push(b);
    }
    // Include the +inf bucket in the total count.
    count += raw_buckets.counts()[raw_buckets.counts().len() - 1] as u64;
    h.set_bucket(protobuf::RepeatedField::from_vec(buckets));
    h.set_sample_count(count);

    let mut m = prometheus::proto::Metric::default();
    m.set_label(protobuf::RepeatedField::from_vec(
        labels.into_iter().map(build_label_pair).collect(),
    ));
    m.set_histogram(h);

    let mut mf = prometheus::proto::MetricFamily::default();
    mf.set_name(desc.fq_name);
    mf.set_help(desc.help);
    mf.set_field_type(prometheus::proto::MetricType::HISTOGRAM);
    mf.set_metric(protobuf::RepeatedField::from_vec(vec![m]));

    Ok(mf)
}

fn build_label_pair(label: KeyValue) -> prometheus::proto::LabelPair {
    let mut lp = prometheus::proto::LabelPair::new();
    lp.set_name(label.key.into());
    lp.set_value(label.value.to_string());

    lp
}

fn merge_labels(
    record: &Record<'_>,
    keys: &mut Vec<String>,
    mut values: Option<&mut Vec<KeyValue>>,
) {
    // Duplicate keys are resolved by taking the record label value over
    // the resource value.

    let iter = labels::merge_iters(record.labels().iter(), record.resource().iter());
    for (key, value) in iter {
        keys.push(sanitize(key.as_str()));
        if let Some(ref mut values) = values {
            values.push(KeyValue::new(key.clone(), value.clone()));
        }
    }
}

fn to_desc(record: &Record<'_>, label_keys: Vec<String>) -> prometheus::core::Desc {
    let desc = record.descriptor();
    prometheus::core::Desc::new(
        sanitize(desc.name()),
        desc.description()
            .cloned()
            .unwrap_or_else(|| desc.name().to_string()),
        label_keys,
        HashMap::new(),
    )
    .unwrap()
}
