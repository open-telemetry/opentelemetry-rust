//! An OpenTelemetry exporter for [Prometheus] metrics.
//!
//! <div class="warning">
//! <strong>Warning: This crate is no longer recommended for use.</strong><br><br>
//! Development of the Prometheus exporter has been discontinued. See the related
//! [issue](https://github.com/open-telemetry/opentelemetry-rust/issues/2451).
//! This crate depends on the unmaintained `protobuf` crate and has unresolved
//! security vulnerabilities. Version 0.29 will be the final release.
//!
//! For Prometheus integration, we strongly recommend using the [OTLP] exporter instead.
//! Prometheus [natively supports OTLP](https://prometheus.io/docs/guides/opentelemetry/#enable-the-otlp-receiver),
//! providing a more stable and actively maintained solution.
//! </div>
//!
//! [Prometheus]: https://prometheus.io
//! [OTLP]: https://docs.rs/opentelemetry-otlp/latest/opentelemetry_otlp/
//!
//! ```
//! use opentelemetry::{metrics::MeterProvider, KeyValue};
//! use opentelemetry_sdk::metrics::SdkMeterProvider;
//! use prometheus::{Encoder, TextEncoder};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//!
//! // create a new prometheus registry
//! let registry = prometheus::Registry::new();
//!
//! // configure OpenTelemetry to use this registry
//! let exporter = opentelemetry_prometheus::exporter()
//!     .with_registry(registry.clone())
//!     .build()?;
//!
//! // set up a meter to create instruments
//! let provider = SdkMeterProvider::builder().with_reader(exporter).build();
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
//! // Encode data as text or protobuf
//! let encoder = TextEncoder::new();
//! let metric_families = registry.gather();
//! let mut result = Vec::new();
//! encoder.encode(&metric_families, &mut result)?;
//!
//! // result now contains encoded metrics:
//! //
//! // # HELP a_counter_total Counts things
//! // # TYPE a_counter_total counter
//! // a_counter_total{key="value",otel_scope_name="my-app"} 100
//! // # HELP a_histogram Records values
//! // # TYPE a_histogram histogram
//! // a_histogram_bucket{key="value",otel_scope_name="my-app",le="0"} 0
//! // a_histogram_bucket{key="value",otel_scope_name="my-app",le="5"} 0
//! // a_histogram_bucket{key="value",otel_scope_name="my-app",le="10"} 0
//! // a_histogram_bucket{key="value",otel_scope_name="my-app",le="25"} 0
//! // a_histogram_bucket{key="value",otel_scope_name="my-app",le="50"} 0
//! // a_histogram_bucket{key="value",otel_scope_name="my-app",le="75"} 0
//! // a_histogram_bucket{key="value",otel_scope_name="my-app",le="100"} 1
//! // a_histogram_bucket{key="value",otel_scope_name="my-app",le="250"} 1
//! // a_histogram_bucket{key="value",otel_scope_name="my-app",le="500"} 1
//! // a_histogram_bucket{key="value",otel_scope_name="my-app",le="750"} 1
//! // a_histogram_bucket{key="value",otel_scope_name="my-app",le="1000"} 1
//! // a_histogram_bucket{key="value",otel_scope_name="my-app",le="2500"} 1
//! // a_histogram_bucket{key="value",otel_scope_name="my-app",le="5000"} 1
//! // a_histogram_bucket{key="value",otel_scope_name="my-app",le="7500"} 1
//! // a_histogram_bucket{key="value",otel_scope_name="my-app",le="10000"} 1
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

use once_cell::sync::OnceCell;
use opentelemetry::{otel_error, otel_warn, InstrumentationScope, Key, Value};
use opentelemetry_sdk::{
    error::OTelSdkResult,
    metrics::{
        data::{self, ResourceMetrics},
        reader::MetricReader,
        InstrumentKind, ManualReader, Pipeline, Temporality,
    },
    Resource,
};
use prometheus::{
    core::Desc,
    proto::{LabelPair, MetricFamily, MetricType},
};
use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    sync::{Arc, Mutex},
};
use std::{fmt, sync::Weak};

const TARGET_INFO_NAME: &str = "target_info";
const TARGET_INFO_DESCRIPTION: &str = "Target metadata";

const SCOPE_INFO_METRIC_NAME: &str = "otel_scope_info";
const SCOPE_INFO_DESCRIPTION: &str = "Instrumentation Scope metadata";

const SCOPE_INFO_KEYS: [&str; 2] = ["otel_scope_name", "otel_scope_version"];

// prometheus counters MUST have a _total suffix by default:
// https://github.com/open-telemetry/opentelemetry-specification/blob/v1.20.0/specification/compatibility/prometheus_and_openmetrics.md
const COUNTER_SUFFIX: &str = "_total";

mod config;
mod resource_selector;
mod utils;

pub use config::ExporterBuilder;
pub use resource_selector::ResourceSelector;

/// Creates a builder to configure a [PrometheusExporter]
pub fn exporter() -> ExporterBuilder {
    ExporterBuilder::default()
}

/// Prometheus metrics exporter
#[derive(Debug)]
pub struct PrometheusExporter {
    reader: Arc<ManualReader>,
}

impl MetricReader for PrometheusExporter {
    fn register_pipeline(&self, pipeline: Weak<Pipeline>) {
        self.reader.register_pipeline(pipeline)
    }

    fn collect(&self, rm: &mut ResourceMetrics) -> OTelSdkResult {
        self.reader.collect(rm)
    }

    fn force_flush(&self) -> OTelSdkResult {
        self.reader.force_flush()
    }

    fn shutdown_with_timeout(&self, timeout: std::time::Duration) -> OTelSdkResult {
        self.reader.shutdown_with_timeout(timeout)
    }

    /// Note: Prometheus only supports cumulative temporality, so this will always be
    /// [Temporality::Cumulative].
    fn temporality(&self, _kind: InstrumentKind) -> Temporality {
        Temporality::Cumulative
    }
}

struct Collector {
    reader: Arc<ManualReader>,
    disable_target_info: bool,
    without_units: bool,
    without_counter_suffixes: bool,
    disable_scope_info: bool,
    create_target_info_once: OnceCell<MetricFamily>,
    resource_labels_once: OnceCell<Vec<LabelPair>>,
    namespace: Option<String>,
    inner: Mutex<CollectorInner>,
    resource_selector: ResourceSelector,
}

#[derive(Default)]
struct CollectorInner {
    scope_infos: HashMap<InstrumentationScope, MetricFamily>,
    metric_families: HashMap<String, MetricFamily>,
}

impl Collector {
    fn metric_type_and_name(&self, m: &data::Metric) -> Option<(MetricType, Cow<'static, str>)> {
        let mut name = self.get_name(m);

        let result = match m.data() {
            data::AggregatedMetrics::F64(metric_data) => match metric_data {
                data::MetricData::Histogram(_) => Some(MetricType::HISTOGRAM),
                data::MetricData::Gauge(_) => Some(MetricType::GAUGE),
                data::MetricData::Sum(sum) => {
                    if sum.is_monotonic() {
                        if !self.without_counter_suffixes {
                            name = format!("{name}{COUNTER_SUFFIX}").into();
                        }
                        Some(MetricType::COUNTER)
                    } else {
                        Some(MetricType::GAUGE)
                    }
                }
                data::MetricData::ExponentialHistogram(_) => None,
            },
            data::AggregatedMetrics::I64(metric_data) => match metric_data {
                data::MetricData::Histogram(_) => Some(MetricType::HISTOGRAM),
                data::MetricData::Gauge(_) => Some(MetricType::GAUGE),
                data::MetricData::Sum(sum) => {
                    if sum.is_monotonic() {
                        if !self.without_counter_suffixes {
                            name = format!("{name}{COUNTER_SUFFIX}").into();
                        }
                        Some(MetricType::COUNTER)
                    } else {
                        Some(MetricType::GAUGE)
                    }
                }
                data::MetricData::ExponentialHistogram(_) => None,
            },
            data::AggregatedMetrics::U64(metric_data) => match metric_data {
                data::MetricData::Histogram(_) => Some(MetricType::HISTOGRAM),
                data::MetricData::Gauge(_) => Some(MetricType::GAUGE),
                data::MetricData::Sum(sum) => {
                    if sum.is_monotonic() {
                        if !self.without_counter_suffixes {
                            name = format!("{name}{COUNTER_SUFFIX}").into();
                        }
                        Some(MetricType::COUNTER)
                    } else {
                        Some(MetricType::GAUGE)
                    }
                }
                data::MetricData::ExponentialHistogram(_) => None,
            },
        };

        result.map(|metric_type| (metric_type, name))
    }

    fn get_name(&self, m: &data::Metric) -> Cow<'static, str> {
        let name: Cow<'static, str> = Cow::Owned(m.name().to_string());
        let name = utils::sanitize_name(&name);
        let unit_suffixes = if self.without_units {
            None
        } else {
            utils::get_unit_suffixes(m.unit())
        };
        match (&self.namespace, unit_suffixes) {
            (Some(namespace), Some(suffix)) => Cow::Owned(format!("{namespace}{name}_{suffix}")),
            (Some(namespace), None) => Cow::Owned(format!("{namespace}{name}")),
            (None, Some(suffix)) => Cow::Owned(format!("{name}_{suffix}")),
            (None, None) => name,
        }
    }
}

impl prometheus::core::Collector for Collector {
    fn desc(&self) -> Vec<&Desc> {
        Vec::new()
    }

    fn collect(&self) -> Vec<MetricFamily> {
        let mut inner = match self.inner.lock() {
            Ok(guard) => guard,
            Err(err) => {
                otel_error!(
                    name: "MetricScrapeFailed",
                    message = err.to_string(),
                );
                return Vec::new();
            }
        };

        let mut metrics = ResourceMetrics::default();
        if let Err(err) = self.reader.collect(&mut metrics) {
            otel_error!(
                name: "MetricScrapeFailed",
                message = err.to_string(),
            );
            return vec![];
        }
        let mut res = Vec::with_capacity(metrics.scope_metrics().count() + 1);

        let target_info = self.create_target_info_once.get_or_init(|| {
            // Resource should be immutable, we don't need to compute again
            create_info_metric(
                TARGET_INFO_NAME,
                TARGET_INFO_DESCRIPTION,
                metrics.resource(),
            )
        });

        if !self.disable_target_info && !metrics.resource().is_empty() {
            res.push(target_info.clone())
        }

        let resource_labels = self
            .resource_labels_once
            .get_or_init(|| self.resource_selector.select(metrics.resource()));

        for scope_metrics in metrics.scope_metrics() {
            let scope_labels = if !self.disable_scope_info {
                if scope_metrics.scope().attributes().count() > 0 {
                    let scope_info = inner
                        .scope_infos
                        .entry(scope_metrics.scope().clone())
                        .or_insert_with_key(create_scope_info_metric);
                    res.push(scope_info.clone());
                }

                let mut labels =
                    Vec::with_capacity(1 + scope_metrics.scope().version().is_some() as usize);
                let mut name = LabelPair::default();
                name.set_name(SCOPE_INFO_KEYS[0].into());
                name.set_value(scope_metrics.scope().name().to_string());
                labels.push(name);
                if let Some(version) = &scope_metrics.scope().version() {
                    let mut l_version = LabelPair::default();
                    l_version.set_name(SCOPE_INFO_KEYS[1].into());
                    l_version.set_value(version.to_string());
                    labels.push(l_version);
                }

                if !resource_labels.is_empty() {
                    labels.extend(resource_labels.iter().cloned());
                }
                labels
            } else {
                Vec::new()
            };

            for metrics in scope_metrics.metrics() {
                let (metric_type, name) = match self.metric_type_and_name(metrics) {
                    Some((metric_type, name)) => (metric_type, name),
                    _ => continue,
                };

                let mfs = &mut inner.metric_families;
                let (drop, help) = validate_metrics(&name, metrics.description(), metric_type, mfs);
                if drop {
                    continue;
                }

                let description = help.unwrap_or_else(|| metrics.description().into());

                match metrics.data() {
                    data::AggregatedMetrics::F64(metric_data) => match metric_data {
                        data::MetricData::Histogram(hist) => {
                            add_histogram_metric(&mut res, hist, description, &scope_labels, name);
                        }
                        data::MetricData::Sum(sum) => {
                            add_sum_metric(&mut res, sum, description, &scope_labels, name);
                        }
                        data::MetricData::Gauge(gauge) => {
                            add_gauge_metric(&mut res, gauge, description, &scope_labels, name);
                        }
                        data::MetricData::ExponentialHistogram(_) => {}
                    },
                    data::AggregatedMetrics::I64(metric_data) => match metric_data {
                        data::MetricData::Histogram(hist) => {
                            add_histogram_metric(&mut res, hist, description, &scope_labels, name);
                        }
                        data::MetricData::Sum(sum) => {
                            add_sum_metric(&mut res, sum, description, &scope_labels, name);
                        }
                        data::MetricData::Gauge(gauge) => {
                            add_gauge_metric(&mut res, gauge, description, &scope_labels, name);
                        }
                        data::MetricData::ExponentialHistogram(_) => {}
                    },
                    data::AggregatedMetrics::U64(metric_data) => match metric_data {
                        data::MetricData::Histogram(hist) => {
                            add_histogram_metric(&mut res, hist, description, &scope_labels, name);
                        }
                        data::MetricData::Sum(sum) => {
                            add_sum_metric(&mut res, sum, description, &scope_labels, name);
                        }
                        data::MetricData::Gauge(gauge) => {
                            add_gauge_metric(&mut res, gauge, description, &scope_labels, name);
                        }
                        data::MetricData::ExponentialHistogram(_) => {}
                    },
                }
            }
        }

        res
    }
}

/// Maps attributes into Prometheus-style label pairs.
///
/// It sanitizes invalid characters and handles duplicate keys (due to
/// sanitization) by sorting and concatenating the values following the spec.
fn get_attrs(kvs: &mut dyn Iterator<Item = (&Key, &Value)>, extra: &[LabelPair]) -> Vec<LabelPair> {
    let mut keys_map = BTreeMap::<String, Vec<String>>::new();
    for (key, value) in kvs {
        let key = utils::sanitize_prom_kv(key.as_str());
        keys_map
            .entry(key)
            .and_modify(|v| v.push(value.to_string()))
            .or_insert_with(|| vec![value.to_string()]);
    }

    let mut res = Vec::with_capacity(keys_map.len() + extra.len());

    for (key, mut values) in keys_map.into_iter() {
        values.sort_unstable();

        let mut lp = LabelPair::default();
        lp.set_name(key);
        lp.set_value(values.join(";"));
        res.push(lp);
    }

    if !extra.is_empty() {
        res.extend(&mut extra.iter().cloned());
    }

    res
}

fn validate_metrics(
    name: &str,
    description: &str,
    metric_type: MetricType,
    mfs: &mut HashMap<String, MetricFamily>,
) -> (bool, Option<String>) {
    if let Some(existing) = mfs.get(name) {
        if existing.get_field_type() != metric_type {
            otel_warn!(
                name: "MetricValidationFailed",
                message = "Instrument type conflict, using existing type definition",
                metric_type = format!("Instrument {name}, Existing: {:?}, dropped: {:?}", existing.get_field_type(), metric_type).as_str(),
            );
            return (true, None);
        }
        if existing.help() != description {
            otel_warn!(
                name: "MetricValidationFailed",
                message = "Instrument description conflict, using existing",
                metric_description = format!("Instrument {name}, Existing: {:?}, dropped: {:?}", existing.help().to_string(), description.to_string()).as_str(),
            );
            return (false, Some(existing.help().to_string()));
        }
        (false, None)
    } else {
        let mut mf = MetricFamily::default();
        mf.set_name(name.into());
        mf.set_help(description.to_string());
        mf.set_field_type(metric_type);
        mfs.insert(name.to_string(), mf);

        (false, None)
    }
}

fn add_histogram_metric<T: Numeric + Copy>(
    res: &mut Vec<MetricFamily>,
    histogram: &data::Histogram<T>,
    description: String,
    extra: &[LabelPair],
    name: Cow<'static, str>,
) {
    // Consider supporting exemplars when `prometheus` crate has the feature
    // See: https://github.com/tikv/rust-prometheus/issues/393

    for dp in histogram.data_points() {
        let kvs = get_attrs(&mut dp.attributes().map(|kv| (&kv.key, &kv.value)), extra);
        let bounds: Vec<f64> = dp.bounds().collect();
        let bucket_counts: Vec<u64> = dp.bucket_counts().collect();
        let bounds_len = bounds.len();
        let (bucket, _) = bounds.iter().enumerate().fold(
            (Vec::with_capacity(bounds_len), 0),
            |(mut acc, mut count), (i, bound)| {
                count += bucket_counts[i];

                let mut b = prometheus::proto::Bucket::default();
                b.set_upper_bound(*bound);
                b.set_cumulative_count(count);
                acc.push(b);
                (acc, count)
            },
        );

        let mut h = prometheus::proto::Histogram::default();
        h.set_sample_sum(dp.sum().as_f64());
        h.set_sample_count(dp.count());
        h.set_bucket(bucket);
        let mut pm = prometheus::proto::Metric::default();
        pm.set_label(kvs);
        pm.set_histogram(h);

        let mut mf = prometheus::proto::MetricFamily::default();
        mf.set_name(name.to_string());
        mf.set_help(description.clone());
        mf.set_field_type(prometheus::proto::MetricType::HISTOGRAM);
        mf.set_metric(vec![pm]);
        res.push(mf);
    }
}

fn add_sum_metric<T: Numeric + Copy>(
    res: &mut Vec<MetricFamily>,
    sum: &data::Sum<T>,
    description: String,
    extra: &[LabelPair],
    name: Cow<'static, str>,
) {
    let metric_type = if sum.is_monotonic() {
        MetricType::COUNTER
    } else {
        MetricType::GAUGE
    };

    for dp in sum.data_points() {
        let kvs = get_attrs(&mut dp.attributes().map(|kv| (&kv.key, &kv.value)), extra);

        let mut pm = prometheus::proto::Metric::default();
        pm.set_label(kvs);

        if sum.is_monotonic() {
            let mut c = prometheus::proto::Counter::default();
            c.set_value(dp.value().as_f64());
            pm.set_counter(c);
        } else {
            let mut g = prometheus::proto::Gauge::default();
            g.set_value(dp.value().as_f64());
            pm.set_gauge(g);
        }

        let mut mf = prometheus::proto::MetricFamily::default();
        mf.set_name(name.to_string());
        mf.set_help(description.clone());
        mf.set_field_type(metric_type);
        mf.set_metric(vec![pm]);
        res.push(mf);
    }
}

fn add_gauge_metric<T: Numeric + Copy>(
    res: &mut Vec<MetricFamily>,
    gauge: &data::Gauge<T>,
    description: String,
    extra: &[LabelPair],
    name: Cow<'static, str>,
) {
    for dp in gauge.data_points() {
        let kvs = get_attrs(&mut dp.attributes().map(|kv| (&kv.key, &kv.value)), extra);

        let mut g = prometheus::proto::Gauge::default();
        g.set_value(dp.value().as_f64());
        let mut pm = prometheus::proto::Metric::default();
        pm.set_label(kvs);
        pm.set_gauge(g);

        let mut mf = prometheus::proto::MetricFamily::default();
        mf.set_name(name.to_string());
        mf.set_help(description.to_string());
        mf.set_field_type(MetricType::GAUGE);
        mf.set_metric(vec![pm]);
        res.push(mf);
    }
}

fn create_info_metric(
    target_info_name: &str,
    target_info_description: &str,
    resource: &Resource,
) -> MetricFamily {
    let mut g = prometheus::proto::Gauge::default();
    g.set_value(1.0);

    let mut m = prometheus::proto::Metric::default();
    m.set_label(get_attrs(&mut resource.iter(), &[]));
    m.set_gauge(g);

    let mut mf = MetricFamily::default();
    mf.set_name(target_info_name.into());
    mf.set_help(target_info_description.into());
    mf.set_field_type(MetricType::GAUGE);
    mf.set_metric(vec![m]);
    mf
}

fn create_scope_info_metric(scope: &InstrumentationScope) -> MetricFamily {
    let mut g = prometheus::proto::Gauge::default();
    g.set_value(1.0);

    let mut labels = Vec::with_capacity(1 + scope.version().is_some() as usize);
    let mut name = LabelPair::default();
    name.set_name(SCOPE_INFO_KEYS[0].into());
    name.set_value(scope.name().to_string());
    labels.push(name);
    if let Some(version) = &scope.version() {
        let mut v_label = LabelPair::default();
        v_label.set_name(SCOPE_INFO_KEYS[1].into());
        v_label.set_value(version.to_string());
        labels.push(v_label);
    }

    let mut m = prometheus::proto::Metric::default();
    m.set_label(labels);
    m.set_gauge(g);

    let mut mf = MetricFamily::default();
    mf.set_name(SCOPE_INFO_METRIC_NAME.into());
    mf.set_help(SCOPE_INFO_DESCRIPTION.into());
    mf.set_field_type(MetricType::GAUGE);
    mf.set_metric(vec![m]);
    mf
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
