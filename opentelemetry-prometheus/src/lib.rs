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
    html_logo_url = "https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo.svg"
)]
#![cfg_attr(test, deny(warnings))]

use opentelemetry::global;
use opentelemetry::sdk::{
    export::metrics::{CheckpointSet, ExportKindSelector, Histogram, LastValue, Record, Sum},
    metrics::{
        aggregators::{HistogramAggregator, LastValueAggregator, SumAggregator},
        controllers,
        selectors::simple::Selector,
        PullController,
    },
    Resource,
};
use opentelemetry::{
    labels,
    metrics::{registry::RegistryMeterProvider, MetricsError, NumberKind},
    Key, Value,
};
use std::env;
use std::sync::{Arc, Mutex};
use std::time::Duration;

mod sanitize;

use sanitize::sanitize;

/// Cache disabled by default.
const DEFAULT_CACHE_PERIOD: Duration = Duration::from_secs(0);

const EXPORT_KIND_SELECTOR: ExportKindSelector = ExportKindSelector::Cumulative;

/// Default host used by the Prometheus Exporter when env variable not found
const DEFAULT_EXPORTER_HOST: &str = "0.0.0.0";

/// Default port used by the Prometheus Exporter when env variable not found
const DEFAULT_EXPORTER_PORT: &str = "9464";

/// The hostname for the Promtheus Exporter
const ENV_EXPORTER_HOST: &str = "OTEL_EXPORTER_PROMETHEUS_HOST";

/// The port for the Prometheus Exporter
const ENV_EXPORTER_PORT: &str = "OTEL_EXPORTER_PROMETHEUS_PORT";

/// Create a new prometheus exporter builder.
pub fn exporter() -> ExporterBuilder {
    ExporterBuilder::default()
}

/// Configuration for the prometheus exporter.
#[derive(Debug)]
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

    /// The host used by the prometheus exporter
    ///
    /// If not set it will be defaulted to all addresses "0.0.0.0"
    host: Option<String>,

    /// The port used by the prometheus exporter
    ///
    /// If not set it will be defaulted to port "9464"
    port: Option<String>,
}

impl Default for ExporterBuilder {
    fn default() -> Self {
        ExporterBuilder {
            resource: None,
            cache_period: None,
            default_histogram_boundaries: None,
            default_summary_quantiles: None,
            registry: None,
            host: env::var(ENV_EXPORTER_HOST).ok().filter(|s| !s.is_empty()),
            port: env::var(ENV_EXPORTER_PORT).ok().filter(|s| !s.is_empty()),
        }
    }
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

    /// Set the host for the prometheus exporter
    pub fn with_host(self, host: String) -> Self {
        ExporterBuilder {
            host: Some(host),
            ..self
        }
    }

    /// Set the port for the prometheus exporter
    pub fn with_port(self, port: String) -> Self {
        ExporterBuilder {
            port: Some(port),
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
        let mut controller_builder = controllers::pull(selector, Box::new(EXPORT_KIND_SELECTOR))
            .with_cache_period(self.cache_period.unwrap_or(DEFAULT_CACHE_PERIOD))
            .with_memory(true);
        if let Some(resource) = self.resource {
            controller_builder = controller_builder.with_resource(resource);
        }
        let controller = controller_builder.build();

        global::set_meter_provider(controller.provider());

        let host = self
            .host
            .unwrap_or_else(|| DEFAULT_EXPORTER_HOST.to_string());
        let port = self
            .port
            .unwrap_or_else(|| DEFAULT_EXPORTER_PORT.to_string());

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
            host,
            port,
        })
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
    host: String,
    port: String,
}

impl PrometheusExporter {
    #[deprecated(
        since = "0.9.0",
        note = "Please use the ExporterBuilder to initialize a PrometheusExporter"
    )]
    /// Create a new prometheus exporter
    pub fn new(
        registry: prometheus::Registry,
        controller: PullController,
        default_summary_quantiles: Vec<f64>,
        default_histogram_boundaries: Vec<f64>,
        host: String,
        port: String,
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
            host,
            port,
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

    /// Get the exporters host for prometheus.
    pub fn host(&self) -> &str {
        self.host.as_str()
    }

    /// Get the exporters port for prometheus.
    pub fn port(&self) -> &str {
        self.port.as_str()
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

            if let Err(err) = controller.try_for_each(&EXPORT_KIND_SELECTOR, &mut |record| {
                let agg = record.aggregator().ok_or(MetricsError::NoDataCollected)?;
                let number_kind = record.descriptor().number_kind();
                let instrument_kind = record.descriptor().instrument_kind();

                let desc = get_metric_desc(record);
                let labels = get_metric_labels(record);

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
    mf.set_name(desc.name);
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

fn build_label_pair(key: &Key, value: &Value) -> prometheus::proto::LabelPair {
    let mut lp = prometheus::proto::LabelPair::new();
    lp.set_name(sanitize(key.as_str()));
    lp.set_value(value.to_string());

    lp
}

fn get_metric_labels(record: &Record<'_>) -> Vec<prometheus::proto::LabelPair> {
    // Duplicate keys are resolved by taking the record label value over
    // the resource value.
    let iter = labels::merge_iters(record.labels().iter(), record.resource().iter());
    iter.map(|(key, value)| build_label_pair(key, value))
        .collect()
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

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[test]
    fn test_exporter_builder_default() {
        env::remove_var(ENV_EXPORTER_HOST);
        env::remove_var(ENV_EXPORTER_PORT);
        let exporter = ExporterBuilder::default().init();
        assert_eq!(exporter.host(), "0.0.0.0");
        assert_eq!(exporter.port(), "9464");

        env::set_var(ENV_EXPORTER_HOST, "prometheus-test");
        env::set_var(ENV_EXPORTER_PORT, "9000");
        let exporter = ExporterBuilder::default().init();
        assert_eq!(exporter.host(), "prometheus-test");
        assert_eq!(exporter.port(), "9000");

        env::set_var(ENV_EXPORTER_HOST, "");
        env::set_var(ENV_EXPORTER_PORT, "");
        let exporter = ExporterBuilder::default().init();
        assert_eq!(exporter.host(), "0.0.0.0");
        assert_eq!(exporter.port(), "9464");
    }
}
