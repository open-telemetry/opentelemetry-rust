//! An OpenTelemetry exporter for [Prometheus] metrics.
//!
//! [Prometheus]: https://prometheus.io
//!
//! ```
//! use opentelemetry_api::{metrics::MeterProvider as _, Context, KeyValue};
//! use opentelemetry_sdk::metrics::MeterProvider;
//! use prometheus::{Encoder, TextEncoder};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let cx = Context::current();
//!
//! // create a new prometheus registry
//! let registry = prometheus::Registry::new();
//!
//! // configure OpenTelemetry to use this registry
//! let exporter = opentelemetry_prometheus::exporter()
//!     .with_registry(registry.clone())
//!     .build()?;
//!
//! // set up a meter meter to create instruments
//! let provider = MeterProvider::builder().with_reader(exporter).build();
//! let meter = provider.meter("my-app");
//!
//! // Use two instruments
//! let counter = meter
//!     .u64_counter("a.counter")
//!     .with_description("Counts things")
//!     .init();
//! let histogram = meter
//!     .i64_histogram("a.histogram")
//!     .with_description("Records values")
//!     .init();
//!
//! counter.add(&cx, 100, &[KeyValue::new("key", "value")]);
//! histogram.record(&cx, 100, &[KeyValue::new("key", "value")]);
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
use opentelemetry_api::{
    global,
    metrics::{MetricsError, Result, Unit},
    Context, Key, Value,
};
use opentelemetry_sdk::{
    metrics::{
        data::{self, ResourceMetrics, Temporality},
        reader::{AggregationSelector, MetricProducer, MetricReader, TemporalitySelector},
        Aggregation, InstrumentKind, ManualReader, Pipeline,
    },
    Resource, Scope,
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

// prometheus counters MUST have a _total suffix:
// https://github.com/open-telemetry/opentelemetry-specification/blob/v1.19.0/specification/compatibility/prometheus_and_openmetrics.md?plain=1#L282
const COUNTER_SUFFIX: &str = "_total";

mod config;

pub use config::ExporterBuilder;

/// Creates a builder to configure a [PrometheusExporter]
pub fn exporter() -> ExporterBuilder {
    ExporterBuilder::default()
}

/// Prometheus metrics exporter
#[derive(Debug)]
pub struct PrometheusExporter {
    reader: Arc<ManualReader>,
}

impl TemporalitySelector for PrometheusExporter {
    /// Note: Prometheus only supports cumulative temporality so this will always be
    /// [Temporality::Cumulative].
    fn temporality(&self, kind: InstrumentKind) -> Temporality {
        self.reader.temporality(kind)
    }
}

impl AggregationSelector for PrometheusExporter {
    fn aggregation(&self, kind: InstrumentKind) -> Aggregation {
        self.reader.aggregation(kind)
    }
}

impl MetricReader for PrometheusExporter {
    fn register_pipeline(&self, pipeline: Weak<Pipeline>) {
        self.reader.register_pipeline(pipeline)
    }

    fn register_producer(&self, producer: Box<dyn MetricProducer>) {
        self.reader.register_producer(producer)
    }

    fn collect(&self, rm: &mut ResourceMetrics) -> Result<()> {
        self.reader.collect(rm)
    }

    fn force_flush(&self, cx: &Context) -> Result<()> {
        self.reader.force_flush(cx)
    }

    fn shutdown(&self) -> Result<()> {
        self.reader.shutdown()
    }
}

struct Collector {
    reader: Arc<ManualReader>,
    disable_target_info: bool,
    without_units: bool,
    disable_scope_info: bool,
    create_target_info_once: OnceCell<MetricFamily>,
    namespace: Option<String>,
    inner: Mutex<CollectorInner>,
}

#[derive(Default)]
struct CollectorInner {
    scope_infos: HashMap<Scope, MetricFamily>,
    metric_families: HashMap<String, MetricFamily>,
}

impl Collector {
    fn get_name(&self, m: &data::Metric) -> Cow<'static, str> {
        let name = sanitize_name(&m.name);
        match (
            &self.namespace,
            get_unit_suffixes(&m.unit).filter(|_| !self.without_units),
        ) {
            (Some(namespace), Some(suffix)) => Cow::Owned(format!("{namespace}{name}{suffix}")),
            (Some(namespace), None) => Cow::Owned(format!("{namespace}{name}")),
            (None, Some(suffix)) => Cow::Owned(format!("{name}{suffix}")),
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
                global::handle_error(err);
                return Vec::new();
            }
        };

        let mut metrics = ResourceMetrics {
            resource: Resource::empty(),
            scope_metrics: vec![],
        };
        if let Err(err) = self.reader.collect(&mut metrics) {
            global::handle_error(err);
            return vec![];
        }
        let mut res = Vec::with_capacity(metrics.scope_metrics.len() + 1);

        let target_info = self.create_target_info_once.get_or_init(|| {
            // Resource should be immutable, we don't need to compute again
            create_info_metric(TARGET_INFO_NAME, TARGET_INFO_DESCRIPTION, &metrics.resource)
        });
        if !self.disable_target_info {
            res.push(target_info.clone())
        }

        for scope_metrics in metrics.scope_metrics {
            let scope_labels = if !self.disable_scope_info {
                let scope_info = inner
                    .scope_infos
                    .entry(scope_metrics.scope.clone())
                    .or_insert_with_key(create_scope_info_metric);
                res.push(scope_info.clone());

                let mut labels =
                    Vec::with_capacity(1 + scope_metrics.scope.version.is_some() as usize);
                let mut name = LabelPair::new();
                name.set_name(SCOPE_INFO_KEYS[0].into());
                name.set_value(scope_metrics.scope.name.to_string());
                labels.push(name);
                if let Some(version) = &scope_metrics.scope.version {
                    let mut l_version = LabelPair::new();
                    l_version.set_name(SCOPE_INFO_KEYS[1].into());
                    l_version.set_value(version.to_string());
                    labels.push(l_version);
                }

                labels
            } else {
                Vec::new()
            };

            for metrics in scope_metrics.metrics {
                let name = self.get_name(&metrics);
                let description = metrics.description;
                let data = metrics.data.as_any();
                let mfs = &mut inner.metric_families;
                if let Some(hist) = data.downcast_ref::<data::Histogram<i64>>() {
                    add_histogram_metric(&mut res, hist, description, &scope_labels, name, mfs);
                } else if let Some(hist) = data.downcast_ref::<data::Histogram<u64>>() {
                    add_histogram_metric(&mut res, hist, description, &scope_labels, name, mfs);
                } else if let Some(hist) = data.downcast_ref::<data::Histogram<f64>>() {
                    add_histogram_metric(&mut res, hist, description, &scope_labels, name, mfs);
                } else if let Some(sum) = data.downcast_ref::<data::Sum<u64>>() {
                    add_sum_metric(&mut res, sum, description, &scope_labels, name, mfs);
                } else if let Some(sum) = data.downcast_ref::<data::Sum<i64>>() {
                    add_sum_metric(&mut res, sum, description, &scope_labels, name, mfs);
                } else if let Some(sum) = data.downcast_ref::<data::Sum<f64>>() {
                    add_sum_metric(&mut res, sum, description, &scope_labels, name, mfs);
                } else if let Some(g) = data.downcast_ref::<data::Gauge<u64>>() {
                    add_gauge_metric(&mut res, g, description, &scope_labels, name, mfs);
                } else if let Some(g) = data.downcast_ref::<data::Gauge<i64>>() {
                    add_gauge_metric(&mut res, g, description, &scope_labels, name, mfs);
                } else if let Some(g) = data.downcast_ref::<data::Gauge<f64>>() {
                    add_gauge_metric(&mut res, g, description, &scope_labels, name, mfs);
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
        let key = sanitize_prom_kv(key.as_str());
        keys_map
            .entry(key)
            .and_modify(|v| v.push(value.to_string()))
            .or_insert_with(|| vec![value.to_string()]);
    }

    let mut res = Vec::with_capacity(keys_map.len() + extra.len());

    for (key, mut values) in keys_map.into_iter() {
        values.sort_unstable();

        let mut lp = LabelPair::new();
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
            global::handle_error(MetricsError::Other(format!("Instrument type conflict, using existing type definition. Instrument {name}, Existing: {:?}, dropped: {:?}", existing.get_field_type(), metric_type)));
            return (true, None);
        }
        if existing.get_help() != description {
            global::handle_error(MetricsError::Other(format!("Instrument description conflict, using existing. Instrument {name}, Existing: {:?}, dropped: {:?}", existing.get_help(), description)));
            return (false, Some(existing.get_help().to_string()));
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

fn add_histogram_metric<T>(
    res: &mut Vec<MetricFamily>,
    histogram: &data::Histogram<T>,
    mut description: Cow<'static, str>,
    extra: &[LabelPair],
    name: Cow<'static, str>,
    mfs: &mut HashMap<String, MetricFamily>,
) {
    // Consider supporting exemplars when `prometheus` crate has the feature
    // See: https://github.com/tikv/rust-prometheus/issues/393
    let (drop, help) = validate_metrics(&name, &description, MetricType::HISTOGRAM, mfs);
    if drop {
        return;
    }
    if let Some(help) = help {
        description = help.into();
    }

    for dp in &histogram.data_points {
        let kvs = get_attrs(&mut dp.attributes.iter(), extra);
        let bounds_len = dp.bounds.len();
        let (bucket, _) = dp.bounds.iter().enumerate().fold(
            (Vec::with_capacity(bounds_len), 0),
            |(mut acc, mut count), (i, bound)| {
                count += dp.bucket_counts[i];

                let mut b = prometheus::proto::Bucket::default();
                b.set_upper_bound(*bound);
                b.set_cumulative_count(count);
                acc.push(b);
                (acc, count)
            },
        );

        let mut h = prometheus::proto::Histogram::default();
        h.set_sample_sum(dp.sum);
        h.set_sample_count(dp.count);
        h.set_bucket(protobuf::RepeatedField::from_vec(bucket));
        let mut pm = prometheus::proto::Metric::default();
        pm.set_label(protobuf::RepeatedField::from_vec(kvs));
        pm.set_histogram(h);

        let mut mf = prometheus::proto::MetricFamily::default();
        mf.set_name(name.to_string());
        mf.set_help(description.to_string());
        mf.set_field_type(prometheus::proto::MetricType::HISTOGRAM);
        mf.set_metric(protobuf::RepeatedField::from_vec(vec![pm]));
        res.push(mf);
    }
}

fn add_sum_metric<T: Numeric>(
    res: &mut Vec<MetricFamily>,
    sum: &data::Sum<T>,
    mut description: Cow<'static, str>,
    extra: &[LabelPair],
    mut name: Cow<'static, str>,
    mfs: &mut HashMap<String, MetricFamily>,
) {
    let metric_type;
    if sum.is_monotonic {
        name = format!("{name}{COUNTER_SUFFIX}").into();
        metric_type = MetricType::COUNTER;
    } else {
        metric_type = MetricType::GAUGE;
    }

    let (drop, help) = validate_metrics(&name, &description, metric_type, mfs);
    if drop {
        return;
    }
    if let Some(help) = help {
        description = help.into();
    }

    for dp in &sum.data_points {
        let kvs = get_attrs(&mut dp.attributes.iter(), extra);

        let mut pm = prometheus::proto::Metric::default();
        pm.set_label(protobuf::RepeatedField::from_vec(kvs));

        if sum.is_monotonic {
            let mut c = prometheus::proto::Counter::default();
            c.set_value(dp.value.as_f64());
            pm.set_counter(c);
        } else {
            let mut g = prometheus::proto::Gauge::default();
            g.set_value(dp.value.as_f64());
            pm.set_gauge(g);
        }

        let mut mf = prometheus::proto::MetricFamily::default();
        mf.set_name(name.to_string());
        mf.set_help(description.to_string());
        mf.set_field_type(metric_type);
        mf.set_metric(protobuf::RepeatedField::from_vec(vec![pm]));
        res.push(mf);
    }
}

fn add_gauge_metric<T: Numeric>(
    res: &mut Vec<MetricFamily>,
    gauge: &data::Gauge<T>,
    mut description: Cow<'static, str>,
    extra: &[LabelPair],
    name: Cow<'static, str>,
    mfs: &mut HashMap<String, MetricFamily>,
) {
    let (drop, help) = validate_metrics(&name, &description, MetricType::GAUGE, mfs);
    if drop {
        return;
    }
    if let Some(help) = help {
        description = help.into();
    }

    for dp in &gauge.data_points {
        let kvs = get_attrs(&mut dp.attributes.iter(), extra);

        let mut g = prometheus::proto::Gauge::default();
        g.set_value(dp.value.as_f64());
        let mut pm = prometheus::proto::Metric::default();
        pm.set_label(protobuf::RepeatedField::from_vec(kvs));
        pm.set_gauge(g);

        let mut mf = prometheus::proto::MetricFamily::default();
        mf.set_name(name.to_string());
        mf.set_help(description.to_string());
        mf.set_field_type(MetricType::GAUGE);
        mf.set_metric(protobuf::RepeatedField::from_vec(vec![pm]));
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
    m.set_label(protobuf::RepeatedField::from_vec(get_attrs(
        &mut resource.iter(),
        &[],
    )));
    m.set_gauge(g);

    let mut mf = MetricFamily::default();
    mf.set_name(target_info_name.into());
    mf.set_help(target_info_description.into());
    mf.set_field_type(MetricType::GAUGE);
    mf.set_metric(protobuf::RepeatedField::from_vec(vec![m]));
    mf
}

fn create_scope_info_metric(scope: &Scope) -> MetricFamily {
    let mut g = prometheus::proto::Gauge::default();
    g.set_value(1.0);

    let mut labels = Vec::with_capacity(1 + scope.version.is_some() as usize);
    let mut name = LabelPair::new();
    name.set_name(SCOPE_INFO_KEYS[0].into());
    name.set_value(scope.name.to_string());
    labels.push(name);
    if let Some(version) = &scope.version {
        let mut v_label = LabelPair::new();
        v_label.set_name(SCOPE_INFO_KEYS[1].into());
        v_label.set_value(version.to_string());
        labels.push(v_label);
    }

    let mut m = prometheus::proto::Metric::default();
    m.set_label(protobuf::RepeatedField::from_vec(labels));
    m.set_gauge(g);

    let mut mf = MetricFamily::default();
    mf.set_name(SCOPE_INFO_METRIC_NAME.into());
    mf.set_help(SCOPE_INFO_DESCRIPTION.into());
    mf.set_field_type(MetricType::GAUGE);
    mf.set_metric(protobuf::RepeatedField::from_vec(vec![m]));
    mf
}

fn get_unit_suffixes(unit: &Unit) -> Option<&'static str> {
    match unit.as_str() {
        "1" => Some("_ratio"),
        "By" => Some("_bytes"),
        "ms" => Some("_milliseconds"),
        _ => None,
    }
}

#[allow(clippy::ptr_arg)]
fn sanitize_name(s: &Cow<'static, str>) -> Cow<'static, str> {
    // prefix chars to add in case name starts with number
    let mut prefix = "";

    // Find first invalid char
    if let Some((replace_idx, _)) = s.char_indices().find(|(i, c)| {
        if *i == 0 && c.is_ascii_digit() {
            // first char is number, add prefix and replace reset of chars
            prefix = "_";
            true
        } else {
            // keep checking
            !c.is_alphanumeric() && *c != '_' && *c != ':'
        }
    }) {
        // up to `replace_idx` have been validated, convert the rest
        let (valid, rest) = s.split_at(replace_idx);
        Cow::Owned(
            prefix
                .chars()
                .chain(valid.chars())
                .chain(rest.chars().map(|c| {
                    if c.is_ascii_alphanumeric() || c == '_' || c == ':' {
                        c
                    } else {
                        '_'
                    }
                }))
                .collect(),
        )
    } else {
        s.clone() // no invalid chars found, return existing
    }
}

fn sanitize_prom_kv(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == ':' {
                c
            } else {
                '_'
            }
        })
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_sanitization() {
        let tests = vec![
            ("nam€_with_3_width_rune", "nam__with_3_width_rune"),
            ("`", "_"),
            (
              r##"! "#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWKYZ[]\^_abcdefghijklmnopqrstuvwkyz{|}~"##,
              "________________0123456789:______ABCDEFGHIJKLMNOPQRSTUVWKYZ_____abcdefghijklmnopqrstuvwkyz____",
            ),

            ("Avalid_23name", "Avalid_23name"),
            ("_Avalid_23name", "_Avalid_23name"),
            ("1valid_23name", "_1valid_23name"),
            ("avalid_23name", "avalid_23name"),
            ("Ava:lid_23name", "Ava:lid_23name"),
            ("a lid_23name", "a_lid_23name"),
            (":leading_colon", ":leading_colon"),
            ("colon:in:the:middle", "colon:in:the:middle"),
            ("", ""),
          ];

        for (input, want) in tests {
            assert_eq!(want, sanitize_name(&input.into()), "input: {input}")
        }
    }
}
