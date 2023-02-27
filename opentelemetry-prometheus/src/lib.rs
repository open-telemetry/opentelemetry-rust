//! # OpenTelemetry Prometheus Exporter
//!
//! ### Prometheus Exporter Example
//!
//! ```rust
//! use opentelemetry::{global, Context, KeyValue, sdk::Resource};
//! use opentelemetry::sdk::export::metrics::aggregation;
//! use opentelemetry::sdk::metrics::{controllers, processors, selectors};
//! use opentelemetry_prometheus::PrometheusExporter;
//! use prometheus::{TextEncoder, Encoder};
//!
//! fn init_meter() -> PrometheusExporter {
//!     let controller = controllers::basic(
//!         processors::factory(
//!             selectors::simple::histogram([1.0, 2.0, 5.0, 10.0, 20.0, 50.0]),
//!             aggregation::cumulative_temporality_selector(),
//!         )
//!     )
//!     .build();
//!
//!     opentelemetry_prometheus::exporter(controller).init()
//! }
//!
//! let cx = Context::current();
//! let exporter = init_meter();
//! let meter = global::meter("my-app");
//!
//! // Use two instruments
//! let counter = meter
//!     .u64_counter("a.counter")
//!     .with_description("Counts things")
//!     .init();
//! let recorder = meter
//!     .i64_histogram("a.histogram")
//!     .with_description("Records values")
//!     .init();
//!
//! counter.add(&cx, 100, &[KeyValue::new("key", "value")]);
//! recorder.record(&cx, 100, &[KeyValue::new("key", "value")]);
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
//! // a_counter{R="V",key="value",otel_scope_name="my-app",otel_scope_version=""} 100
//! // # HELP a_histogram Records values
//! // # TYPE a_histogram histogram
//! // a_histogram_bucket{R="V",key="value",le="0.5",otel_scope_name="my-app",otel_scope_version=""} 0
//! // a_histogram_bucket{R="V",key="value",le="0.9",otel_scope_name="my-app",otel_scope_version=""} 0
//! // a_histogram_bucket{R="V",key="value",le="0.99",otel_scope_name="my-app",otel_scope_version=""} 0
//! // a_histogram_bucket{R="V",key="value",le="+Inf",otel_scope_name="my-app",otel_scope_version=""} 1
//! // a_histogram_sum{R="V",key="value",otel_scope_name="my-app",otel_scope_version=""} 100
//! // a_histogram_count{R="V",key="value",otel_scope_name="my-app",otel_scope_version=""} 1
//! // HELP otel_scope_info Instrumentation Scope metadata
//! // TYPE otel_scope_info gauge
//! // otel_scope_info{otel_scope_name="ex.com/B",otel_scope_version=""} 1
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

use opentelemetry::metrics::MeterProvider;
use opentelemetry::sdk::export::metrics::aggregation::{
    self, AggregationKind, Temporality, TemporalitySelector,
};
use opentelemetry::sdk::export::metrics::InstrumentationLibraryReader;
use opentelemetry::sdk::metrics::sdk_api::Descriptor;
#[cfg(feature = "prometheus-encoding")]
pub use prometheus::{Encoder, TextEncoder};

use opentelemetry::sdk::{
    export::metrics::{
        aggregation::{Histogram, LastValue, Sum},
        Record,
    },
    metrics::{
        aggregators::{HistogramAggregator, LastValueAggregator, SumAggregator},
        controllers::BasicController,
        sdk_api::NumberKind,
    },
    Resource,
};
use opentelemetry::{attributes, metrics::MetricsError, Context, Key, Value};
use opentelemetry::{global, InstrumentationLibrary, StringValue};
use std::sync::{Arc, Mutex};

mod sanitize;

use sanitize::sanitize;

/// Monotonic Sum metric points MUST have _total added as a suffix to the metric name
/// https://github.com/open-telemetry/opentelemetry-specification/blob/v1.14.0/specification/metrics/data-model.md#sums-1
const MONOTONIC_COUNTER_SUFFIX: &str = "_total";

/// Instrumentation Scope name MUST added as otel_scope_name label.
const OTEL_SCOPE_NAME: &str = "otel_scope_name";

/// Instrumentation Scope version MUST added as otel_scope_name label.
const OTEL_SCOPE_VERSION: &str = "otel_scope_version";

/// otel_scope_name metric name.
const SCOPE_INFO_METRIC_NAME: &str = "otel_scope_info";

/// otel_scope_name metric help.
const SCOPE_INFO_DESCRIPTION: &str = "Instrumentation Scope metadata";
/// Create a new prometheus exporter builder.
pub fn exporter(controller: BasicController) -> ExporterBuilder {
    ExporterBuilder::new(controller)
}

/// Configuration for the prometheus exporter.
#[derive(Debug)]
pub struct ExporterBuilder {
    /// The prometheus registry that will be used to register instruments.
    ///
    /// If not set a new empty `Registry` is created.
    registry: Option<prometheus::Registry>,

    /// The metrics controller
    controller: BasicController,

    // config for exporter
    config: Option<ExporterConfig>,
}

impl ExporterBuilder {
    /// Create a new exporter builder with a given controller
    pub fn new(controller: BasicController) -> Self {
        ExporterBuilder {
            registry: None,
            controller,
            config: Some(Default::default()),
        }
    }

    /// Set the prometheus registry to be used by this exporter
    pub fn with_registry(self, registry: prometheus::Registry) -> Self {
        ExporterBuilder {
            registry: Some(registry),
            ..self
        }
    }

    /// Set config to be used by this exporter
    pub fn with_config(self, config: ExporterConfig) -> Self {
        ExporterBuilder {
            config: Some(config),
            ..self
        }
    }

    /// Sets up a complete export pipeline with the recommended setup, using the
    /// recommended selector and standard processor.
    pub fn try_init(self) -> Result<PrometheusExporter, MetricsError> {
        let config = self.config.unwrap_or_default();

        let registry = self.registry.unwrap_or_else(prometheus::Registry::new);

        let controller = Arc::new(Mutex::new(self.controller));
        let collector = Collector::with_controller(controller.clone())
            .with_disable_scope_info(config.disable_scope_info);
        registry
            .register(Box::new(collector))
            .map_err(|e| MetricsError::Other(e.to_string()))?;

        let exporter = PrometheusExporter {
            registry,
            controller,
        };
        global::set_meter_provider(exporter.meter_provider()?);

        Ok(exporter)
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

/// Config for prometheus exporter
#[derive(Debug)]
pub struct ExporterConfig {
    /// disable the otel_scope_info metric and otel_scope_ labels.
    disable_scope_info: bool,
}

impl Default for ExporterConfig {
    fn default() -> Self {
        ExporterConfig {
            disable_scope_info: false,
        }
    }
}

impl ExporterConfig {
    /// Set disable_scope_info for [`ExporterConfig`].
    /// It's the flag to disable the otel_scope_info metric and otel_scope_ labels.
    pub fn disable_scope_info(mut self) -> Self {
        self.disable_scope_info = true;
        self
    }
}

/// An implementation of `metrics::Exporter` that sends metrics to Prometheus.
///
/// This exporter supports Prometheus pulls, as such it does not
/// implement the export.Exporter interface.
#[derive(Clone, Debug)]
pub struct PrometheusExporter {
    registry: prometheus::Registry,
    controller: Arc<Mutex<BasicController>>,
}

impl PrometheusExporter {
    /// Returns a reference to the current prometheus registry.
    pub fn registry(&self) -> &prometheus::Registry {
        &self.registry
    }

    /// Get this exporter's provider.
    pub fn meter_provider(&self) -> Result<impl MeterProvider, MetricsError> {
        self.controller
            .lock()
            .map_err(Into::into)
            .map(|locked| locked.clone())
    }
}

#[derive(Debug)]
struct Collector {
    controller: Arc<Mutex<BasicController>>,
    disable_scope_info: bool,
}

impl TemporalitySelector for Collector {
    fn temporality_for(&self, descriptor: &Descriptor, kind: &AggregationKind) -> Temporality {
        aggregation::cumulative_temporality_selector().temporality_for(descriptor, kind)
    }
}

impl Collector {
    fn with_controller(controller: Arc<Mutex<BasicController>>) -> Self {
        Collector {
            controller,
            disable_scope_info: false,
        }
    }
    fn with_disable_scope_info(mut self, disable_scope_info: bool) -> Self {
        self.disable_scope_info = disable_scope_info;
        self
    }
}

impl prometheus::core::Collector for Collector {
    /// Unused as descriptors are dynamically registered.
    fn desc(&self) -> Vec<&prometheus::core::Desc> {
        Vec::new()
    }

    /// Collect all otel metrics and convert to prometheus metrics.
    fn collect(&self) -> Vec<prometheus::proto::MetricFamily> {
        if let Ok(controller) = self.controller.lock() {
            let mut metrics = Vec::new();

            if let Err(err) = controller.collect(&Context::current()) {
                global::handle_error(err);
                return metrics;
            }

            if let Err(err) = controller.try_for_each(&mut |library, reader| {
                let mut scope_labels: Vec<prometheus::proto::LabelPair> = Vec::new();
                if !self.disable_scope_info {
                    scope_labels = get_scope_labels(library);
                    metrics.push(build_scope_metric(scope_labels.clone()));
                }
                reader.try_for_each(self, &mut |record| {
                    let agg = record.aggregator().ok_or(MetricsError::NoDataCollected)?;
                    let number_kind = record.descriptor().number_kind();
                    let instrument_kind = record.descriptor().instrument_kind();

                    let desc = get_metric_desc(record);
                    let labels =
                        get_metric_labels(record, controller.resource(), &mut scope_labels.clone());

                    if let Some(hist) = agg.as_any().downcast_ref::<HistogramAggregator>() {
                        metrics.push(build_histogram(hist, number_kind, desc, labels)?);
                    } else if let Some(sum) = agg.as_any().downcast_ref::<SumAggregator>() {
                        let counter = if instrument_kind.monotonic() {
                            build_monotonic_counter(sum, number_kind, desc, labels)?
                        } else {
                            build_non_monotonic_counter(sum, number_kind, desc, labels)?
                        };

                        metrics.push(counter);
                    } else if let Some(last) = agg.as_any().downcast_ref::<LastValueAggregator>() {
                        metrics.push(build_last_value(last, number_kind, desc, labels)?);
                    }

                    Ok(())
                })
            }) {
                global::handle_error(err);
            }

            metrics
        } else {
            Vec::new()
        }
    }
}

fn build_last_value(
    lv: &LastValueAggregator,
    kind: &NumberKind,
    desc: PrometheusMetricDesc,
    labels: Vec<prometheus::proto::LabelPair>,
) -> Result<prometheus::proto::MetricFamily, MetricsError> {
    let (last_value, _) = lv.last_value()?;

    let mut g = prometheus::proto::Gauge::default();
    g.set_value(last_value.to_f64(kind));

    let mut m = prometheus::proto::Metric::default();
    m.set_label(protobuf::RepeatedField::from_vec(labels));
    m.set_gauge(g);

    let mut mf = prometheus::proto::MetricFamily::default();
    mf.set_name(desc.name);
    mf.set_help(desc.help);
    mf.set_field_type(prometheus::proto::MetricType::GAUGE);
    mf.set_metric(protobuf::RepeatedField::from_vec(vec![m]));

    Ok(mf)
}

fn build_non_monotonic_counter(
    sum: &SumAggregator,
    kind: &NumberKind,
    desc: PrometheusMetricDesc,
    labels: Vec<prometheus::proto::LabelPair>,
) -> Result<prometheus::proto::MetricFamily, MetricsError> {
    let sum = sum.sum()?;

    let mut g = prometheus::proto::Gauge::default();
    g.set_value(sum.to_f64(kind));

    let mut m = prometheus::proto::Metric::default();
    m.set_label(protobuf::RepeatedField::from_vec(labels));
    m.set_gauge(g);

    let mut mf = prometheus::proto::MetricFamily::default();
    mf.set_name(desc.name);
    mf.set_help(desc.help);
    mf.set_field_type(prometheus::proto::MetricType::GAUGE);
    mf.set_metric(protobuf::RepeatedField::from_vec(vec![m]));

    Ok(mf)
}

fn build_monotonic_counter(
    sum: &SumAggregator,
    kind: &NumberKind,
    desc: PrometheusMetricDesc,
    labels: Vec<prometheus::proto::LabelPair>,
) -> Result<prometheus::proto::MetricFamily, MetricsError> {
    let sum = sum.sum()?;

    let mut c = prometheus::proto::Counter::default();
    c.set_value(sum.to_f64(kind));

    let mut m = prometheus::proto::Metric::default();
    m.set_label(protobuf::RepeatedField::from_vec(labels));
    m.set_counter(c);

    let mut mf = prometheus::proto::MetricFamily::default();
    mf.set_name(desc.name + MONOTONIC_COUNTER_SUFFIX);
    mf.set_help(desc.help);
    mf.set_field_type(prometheus::proto::MetricType::COUNTER);
    mf.set_metric(protobuf::RepeatedField::from_vec(vec![m]));

    Ok(mf)
}

fn build_histogram(
    hist: &HistogramAggregator,
    kind: &NumberKind,
    desc: PrometheusMetricDesc,
    labels: Vec<prometheus::proto::LabelPair>,
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
    m.set_label(protobuf::RepeatedField::from_vec(labels));
    m.set_histogram(h);

    let mut mf = prometheus::proto::MetricFamily::default();
    mf.set_name(desc.name);
    mf.set_help(desc.help);
    mf.set_field_type(prometheus::proto::MetricType::HISTOGRAM);
    mf.set_metric(protobuf::RepeatedField::from_vec(vec![m]));

    Ok(mf)
}

fn build_scope_metric(
    labels: Vec<prometheus::proto::LabelPair>,
) -> prometheus::proto::MetricFamily {
    let mut g = prometheus::proto::Gauge::new();
    g.set_value(1.0);

    let mut m = prometheus::proto::Metric::default();
    m.set_label(protobuf::RepeatedField::from_vec(labels));
    m.set_gauge(g);

    let mut mf = prometheus::proto::MetricFamily::default();
    mf.set_name(String::from(SCOPE_INFO_METRIC_NAME));
    mf.set_help(String::from(SCOPE_INFO_DESCRIPTION));
    mf.set_field_type(prometheus::proto::MetricType::GAUGE);
    mf.set_metric(protobuf::RepeatedField::from_vec(vec![m]));

    mf
}

fn get_scope_labels(library: &InstrumentationLibrary) -> Vec<prometheus::proto::LabelPair> {
    let mut labels = Vec::new();
    labels.push(build_label_pair(
        &Key::new(OTEL_SCOPE_NAME),
        &Value::String(StringValue::from(library.name.to_owned().to_string())),
    ));
    if let Some(version) = library.version.to_owned() {
        labels.push(build_label_pair(
            &Key::new(OTEL_SCOPE_VERSION),
            &Value::String(StringValue::from(version.to_string())),
        ));
    } else {
        labels.push(build_label_pair(
            &Key::new(OTEL_SCOPE_VERSION),
            &Value::String(StringValue::from("")),
        ));
    }
    labels
}

fn build_label_pair(key: &Key, value: &Value) -> prometheus::proto::LabelPair {
    let mut lp = prometheus::proto::LabelPair::new();
    lp.set_name(sanitize(key.as_str()));
    lp.set_value(value.to_string());

    lp
}

fn get_metric_labels(
    record: &Record<'_>,
    resource: &Resource,
    scope_labels: &mut Vec<prometheus::proto::LabelPair>,
) -> Vec<prometheus::proto::LabelPair> {
    // Duplicate keys are resolved by taking the record label value over
    // the resource value.
    let iter = attributes::merge_iters(record.attributes().iter(), resource.iter());
    let mut labels: Vec<prometheus::proto::LabelPair> = iter
        .map(|(key, value)| build_label_pair(key, value))
        .collect();

    labels.append(scope_labels);
    labels
}

struct PrometheusMetricDesc {
    name: String,
    help: String,
}

fn get_metric_desc(record: &Record<'_>) -> PrometheusMetricDesc {
    let desc = record.descriptor();
    let name = sanitize(desc.name());
    let help = desc
        .description()
        .cloned()
        .unwrap_or_else(|| desc.name().to_string());
    PrometheusMetricDesc { name, help }
}
