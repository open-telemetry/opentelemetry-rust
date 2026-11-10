//! An OpenTelemetry exporter for [Prometheus] metrics.
//!
//! [Prometheus]: https://prometheus.io
//!
//! ```
//! use opentelemetry::{metrics::MeterProvider, KeyValue};
//! use opentelemetry_sdk::metrics::SdkMeterProvider;
//! use prometheus_client::{encoding::text::encode, registry::Registry};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//!
//! // create a new prometheus registry
//! let mut registry = Registry::default();
//!
//! // configure OpenTelemetry to use this registry
//! let exporter = opentelemetry_prometheus::exporter().build(&mut registry)?;
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
//! // Encode data in the OpenMetrics text format
//! let mut result = String::new();
//! encode(&mut result, &registry)?;
//!
//! // result now contains encoded metrics:
//! //
//! // # HELP a_counter Counts things
//! // # TYPE a_counter counter
//! // a_counter_total{key="value",otel_scope_name="my-app"} 100.0
//! // # HELP a_histogram Records values
//! // # TYPE a_histogram histogram
//! // a_histogram_sum{key="value",otel_scope_name="my-app"} 100.0
//! // a_histogram_count{key="value",otel_scope_name="my-app"} 1
//! // a_histogram_bucket{le="0.0",key="value",otel_scope_name="my-app"} 0
//! // a_histogram_bucket{le="5.0",key="value",otel_scope_name="my-app"} 0
//! // a_histogram_bucket{le="10.0",key="value",otel_scope_name="my-app"} 0
//! // a_histogram_bucket{le="25.0",key="value",otel_scope_name="my-app"} 0
//! // a_histogram_bucket{le="50.0",key="value",otel_scope_name="my-app"} 0
//! // a_histogram_bucket{le="75.0",key="value",otel_scope_name="my-app"} 0
//! // a_histogram_bucket{le="100.0",key="value",otel_scope_name="my-app"} 1
//! // a_histogram_bucket{le="250.0",key="value",otel_scope_name="my-app"} 1
//! // a_histogram_bucket{le="500.0",key="value",otel_scope_name="my-app"} 1
//! // a_histogram_bucket{le="750.0",key="value",otel_scope_name="my-app"} 1
//! // a_histogram_bucket{le="1000.0",key="value",otel_scope_name="my-app"} 1
//! // a_histogram_bucket{le="2500.0",key="value",otel_scope_name="my-app"} 1
//! // a_histogram_bucket{le="5000.0",key="value",otel_scope_name="my-app"} 1
//! // a_histogram_bucket{le="7500.0",key="value",otel_scope_name="my-app"} 1
//! // a_histogram_bucket{le="10000.0",key="value",otel_scope_name="my-app"} 1
//! // a_histogram_bucket{le="+Inf",key="value",otel_scope_name="my-app"} 1
//! // # HELP target_info Target metadata
//! // # TYPE target_info gauge
//! // target_info{service_name="unknown_service"} 1.0
//! // # EOF
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
#![cfg_attr(docsrs, feature(doc_cfg), deny(rustdoc::broken_intra_doc_links))]
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
};
use prometheus_client::{
    encoding::{DescriptorEncoder, NoLabelSet},
    metrics::MetricType,
    registry::Unit,
};
use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    sync::{Arc, Mutex},
};
use std::{fmt, sync::Weak};

const TARGET_INFO_NAME: &str = "target_info";
const TARGET_INFO_DESCRIPTION: &str = "Target metadata";

const SCOPE_NAME_LABEL: &str = "otel_scope_name";
const SCOPE_VERSION_LABEL: &str = "otel_scope_version";
const SCOPE_SCHEMA_URL_LABEL: &str = "otel_scope_schema_url";
const SCOPE_ATTRIBUTE_PREFIX: &str = "otel_scope_";
const RESERVED_SCOPE_LABELS: [&str; 3] = [
    SCOPE_NAME_LABEL,
    SCOPE_VERSION_LABEL,
    SCOPE_SCHEMA_URL_LABEL,
];

// prometheus counters MUST have a _total suffix:
// https://github.com/open-telemetry/opentelemetry-specification/blob/v1.20.0/specification/compatibility/prometheus_and_openmetrics.md
// The OpenMetrics encoder appends it to counter samples; it is only used here to
// detect name conflicts between exposed sample names.
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

/// A set of Prometheus label name/value pairs.
type Labels = Vec<(String, String)>;

#[derive(Debug)]
enum PointValue {
    Counter(f64),
    Gauge(f64),
    Histogram {
        sum: f64,
        count: u64,
        /// Non-cumulative `(upper_bound, count)` pairs, ending with `+Inf`.
        buckets: Vec<(f64, u64)>,
    },
}

/// A metric family to be encoded, holding all of its data points.
#[derive(Debug)]
struct Family {
    /// Name without the unit suffix, which the encoder appends.
    name: String,
    help: String,
    unit: Option<Unit>,
    kind: MetricType,
    points: Vec<(Labels, PointValue)>,
}

#[derive(Debug)]
struct Collector {
    reader: Arc<ManualReader>,
    disable_target_info: bool,
    without_units: bool,
    scope_info_enabled: bool,
    create_target_info_once: OnceCell<Labels>,
    resource_labels_once: OnceCell<Labels>,
    namespace: Option<String>,
    inner: Mutex<CollectorInner>,
    resource_selector: ResourceSelector,
}

#[derive(Debug, Default)]
struct CollectorInner {
    /// Type and description of every exposed metric name seen so far.
    metric_families: HashMap<String, (MetricType, String)>,
}

impl Collector {
    /// Returns the metric name (with namespace, without unit) and the unit suffix.
    fn get_name(&self, m: &data::Metric) -> (Cow<'static, str>, Option<Cow<'static, str>>) {
        let name: Cow<'static, str> = Cow::Owned(m.name().to_string());
        let name = utils::sanitize_name(&name);
        let unit_suffix = if self.without_units {
            None
        } else {
            utils::get_unit_suffixes(m.unit())
        };
        let name = match &self.namespace {
            Some(namespace) => Cow::Owned(format!("{namespace}{name}")),
            None => name,
        };
        (name, unit_suffix)
    }

    /// Collects metrics from the reader and groups them into families keyed by name.
    fn collect_families(&self) -> BTreeMap<String, Family> {
        let mut families = BTreeMap::new();

        let mut inner = match self.inner.lock() {
            Ok(guard) => guard,
            Err(err) => {
                otel_error!(
                    name: "MetricScrapeFailed",
                    message = err.to_string(),
                );
                return families;
            }
        };

        let mut metrics = ResourceMetrics::default();
        if let Err(err) = self.reader.collect(&mut metrics) {
            otel_error!(
                name: "MetricScrapeFailed",
                message = err.to_string(),
            );
            return families;
        }

        let target_info = self.create_target_info_once.get_or_init(|| {
            // Resource should be immutable, we don't need to compute again
            get_attrs(&mut metrics.resource().iter(), &[])
        });

        if !self.disable_target_info && !metrics.resource().is_empty() {
            insert_into_family(
                &mut families,
                TARGET_INFO_NAME.to_string(),
                TARGET_INFO_NAME,
                TARGET_INFO_DESCRIPTION.to_string(),
                None,
                MetricType::Gauge,
                (target_info.clone(), PointValue::Gauge(1.0)),
            );
        }

        let resource_labels = self
            .resource_labels_once
            .get_or_init(|| self.resource_selector.select(metrics.resource()));

        for scope_metrics in metrics.scope_metrics() {
            let mut scope_labels = if self.scope_info_enabled {
                get_scope_labels(scope_metrics.scope())
            } else {
                Vec::new()
            };
            scope_labels.extend(resource_labels.iter().cloned());

            for metric in scope_metrics.metrics() {
                let Some(kind) = prometheus_metric_type(metric) else {
                    continue;
                };
                let (name, unit_suffix) = self.get_name(metric);

                let family_name = match &unit_suffix {
                    Some(suffix) => format!("{name}_{suffix}"),
                    None => name.to_string(),
                };
                // Validate against the exposed sample name, which includes the
                // `_total` suffix for counters.
                let exposed_name = if matches!(kind, MetricType::Counter) {
                    format!("{family_name}{COUNTER_SUFFIX}")
                } else {
                    family_name.clone()
                };

                let mfs = &mut inner.metric_families;
                let (drop, help) = validate_metrics(&exposed_name, metric.description(), kind, mfs);
                if drop {
                    continue;
                }

                let description = help.unwrap_or_else(|| metric.description().into());
                let unit = unit_suffix.map(|suffix| Unit::Other(suffix.into_owned()));

                let mut add = |point| {
                    insert_into_family(
                        &mut families,
                        family_name.clone(),
                        &name,
                        description.clone(),
                        unit.clone(),
                        kind,
                        point,
                    )
                };

                match metric.data() {
                    data::AggregatedMetrics::F64(metric_data) => {
                        for_each_point(metric_data, &scope_labels, &mut add)
                    }
                    data::AggregatedMetrics::I64(metric_data) => {
                        for_each_point(metric_data, &scope_labels, &mut add)
                    }
                    data::AggregatedMetrics::U64(metric_data) => {
                        for_each_point(metric_data, &scope_labels, &mut add)
                    }
                }
            }
        }

        families
    }
}

impl prometheus_client::collector::Collector for Collector {
    fn encode(&self, mut encoder: DescriptorEncoder<'_>) -> Result<(), fmt::Error> {
        for mut family in self.collect_families().into_values() {
            // Sort data points lexicographically by their label values for a
            // stable output.
            family.points.sort_by(|(left, _), (right, _)| {
                left.len().cmp(&right.len()).then_with(|| {
                    left.iter()
                        .map(|(_, v)| v)
                        .cmp(right.iter().map(|(_, v)| v))
                })
            });

            let mut metric_encoder = encoder.encode_descriptor(
                &family.name,
                &family.help,
                family.unit.as_ref(),
                family.kind,
            )?;

            for (labels, value) in &family.points {
                let mut point_encoder = metric_encoder.encode_family(labels)?;
                match value {
                    PointValue::Counter(v) => {
                        point_encoder.encode_counter::<NoLabelSet, _, f64>(v, None)?
                    }
                    PointValue::Gauge(v) => point_encoder.encode_gauge(v)?,
                    PointValue::Histogram {
                        sum,
                        count,
                        buckets,
                    } => {
                        point_encoder.encode_histogram::<NoLabelSet>(*sum, *count, buckets, None)?
                    }
                }
            }
        }

        Ok(())
    }
}

/// Maps an OpenTelemetry metric to its Prometheus metric type.
///
/// Returns `None` for metrics that cannot be exported (exponential histograms).
fn prometheus_metric_type(m: &data::Metric) -> Option<MetricType> {
    fn kind<T>(metric_data: &data::MetricData<T>) -> Option<MetricType> {
        match metric_data {
            data::MetricData::Histogram(_) => Some(MetricType::Histogram),
            data::MetricData::Gauge(_) => Some(MetricType::Gauge),
            data::MetricData::Sum(sum) if sum.is_monotonic() => Some(MetricType::Counter),
            data::MetricData::Sum(_) => Some(MetricType::Gauge),
            data::MetricData::ExponentialHistogram(_) => None,
        }
    }

    match m.data() {
        data::AggregatedMetrics::F64(metric_data) => kind(metric_data),
        data::AggregatedMetrics::I64(metric_data) => kind(metric_data),
        data::AggregatedMetrics::U64(metric_data) => kind(metric_data),
    }
}

/// Adds a data point to the family keyed by `family_name` (the name including the
/// unit suffix), creating the family if needed.
///
/// If a family with the same name but a different type already exists, the data
/// point is dropped, as a family can only have a single type.
fn insert_into_family(
    families: &mut BTreeMap<String, Family>,
    family_name: String,
    name: &str,
    help: String,
    unit: Option<Unit>,
    kind: MetricType,
    point: (Labels, PointValue),
) {
    let family = families.entry(family_name).or_insert_with(|| Family {
        name: name.to_string(),
        help,
        unit,
        kind,
        points: Vec::new(),
    });

    if family.kind.as_str() != kind.as_str() {
        otel_warn!(
            name: "MetricValidationFailed",
            message = "Instrument type conflict, using existing type definition",
            metric_type = format!("Instrument {}, Existing: {:?}, dropped: {:?}", family.name, family.kind, kind).as_str(),
        );
        return;
    }

    family.points.push(point);
}

/// Converts each data point of a metric into a Prometheus data point and passes it to `add`.
fn for_each_point<T: Numeric + Copy>(
    metric_data: &data::MetricData<T>,
    extra: &[(String, String)],
    add: &mut dyn FnMut((Labels, PointValue)),
) {
    match metric_data {
        data::MetricData::Histogram(histogram) => {
            // Consider supporting exemplars, which `prometheus-client` supports.
            for dp in histogram.data_points() {
                let labels = get_attrs(&mut dp.attributes().map(|kv| (&kv.key, &kv.value)), extra);
                // OTel bucket counts are non-cumulative, with one more bucket than
                // bounds; the last bucket is the `+Inf` overflow bucket.
                let buckets = dp
                    .bounds()
                    .chain(std::iter::once(f64::INFINITY))
                    .zip(dp.bucket_counts())
                    .collect();
                add((
                    labels,
                    PointValue::Histogram {
                        sum: dp.sum().as_f64(),
                        count: dp.count(),
                        buckets,
                    },
                ));
            }
        }
        data::MetricData::Sum(sum) => {
            for dp in sum.data_points() {
                let labels = get_attrs(&mut dp.attributes().map(|kv| (&kv.key, &kv.value)), extra);
                let value = if sum.is_monotonic() {
                    PointValue::Counter(dp.value().as_f64())
                } else {
                    PointValue::Gauge(dp.value().as_f64())
                };
                add((labels, value));
            }
        }
        data::MetricData::Gauge(gauge) => {
            for dp in gauge.data_points() {
                let labels = get_attrs(&mut dp.attributes().map(|kv| (&kv.key, &kv.value)), extra);
                add((labels, PointValue::Gauge(dp.value().as_f64())));
            }
        }
        data::MetricData::ExponentialHistogram(_) => {}
    }
}

/// Maps attributes into Prometheus-style label pairs.
///
/// It sanitizes invalid characters and handles duplicate keys (due to
/// sanitization) by sorting and concatenating the values following the spec.
fn get_attrs(kvs: &mut dyn Iterator<Item = (&Key, &Value)>, extra: &[(String, String)]) -> Labels {
    let mut keys_map = BTreeMap::<String, Vec<(String, String)>>::new();
    let mut extra_keys_map = BTreeMap::<String, Vec<(String, String)>>::new();
    let mut extra_key_order = Vec::with_capacity(extra.len());

    for (key, value) in extra {
        if !extra_keys_map.contains_key(key) {
            extra_key_order.push(key.clone());
        }
        extra_keys_map
            .entry(key.clone())
            .and_modify(|values| values.push((key.clone(), value.clone())))
            .or_insert_with(|| vec![(key.clone(), value.clone())]);
    }

    for (key, value) in kvs {
        let original_key = key.as_str().to_string();
        let sanitized_key = utils::sanitize_prom_kv(key.as_str());

        if let Some(values) = extra_keys_map.get_mut(&sanitized_key) {
            values.push((original_key, value.to_string()));
        } else {
            keys_map
                .entry(sanitized_key)
                .and_modify(|values| values.push((original_key.clone(), value.to_string())))
                .or_insert_with(|| vec![(original_key, value.to_string())]);
        }
    }

    let mut res = Vec::with_capacity(keys_map.len() + extra.len());

    for (key, mut values) in keys_map.into_iter() {
        values.sort_by(|(left_key, _), (right_key, _)| left_key.cmp(right_key));
        res.push(label_pair(
            key,
            values
                .into_iter()
                .map(|(_, value)| value)
                .collect::<Vec<_>>()
                .join(";"),
        ));
    }

    for key in extra_key_order {
        if let Some(mut values) = extra_keys_map.remove(&key) {
            values.sort_by(|(left_key, _), (right_key, _)| left_key.cmp(right_key));
            res.push(label_pair(
                key,
                values
                    .into_iter()
                    .map(|(_, value)| value)
                    .collect::<Vec<_>>()
                    .join(";"),
            ));
        }
    }

    res
}

fn get_scope_labels(scope: &InstrumentationScope) -> Labels {
    let mut labels = Vec::with_capacity(
        1 + scope.version().is_some() as usize
            + scope.schema_url().is_some() as usize
            + scope.attributes().count(),
    );
    labels.push(label_pair(SCOPE_NAME_LABEL, scope.name()));

    if let Some(version) = scope.version() {
        labels.push(label_pair(SCOPE_VERSION_LABEL, version));
    }

    if let Some(schema_url) = scope.schema_url() {
        labels.push(label_pair(SCOPE_SCHEMA_URL_LABEL, schema_url));
    }

    let mut attr_labels = BTreeMap::<String, Vec<String>>::new();
    for kv in scope.attributes() {
        if matches!(kv.key.as_str(), "name" | "version" | "schema_url") {
            continue;
        }

        let label_name = utils::sanitize_prom_kv(&format!("{SCOPE_ATTRIBUTE_PREFIX}{}", kv.key));
        if RESERVED_SCOPE_LABELS.contains(&label_name.as_str()) {
            continue;
        }
        attr_labels
            .entry(label_name)
            .and_modify(|values| values.push(kv.value.to_string()))
            .or_insert_with(|| vec![kv.value.to_string()]);
    }

    for (label_name, values) in attr_labels {
        labels.push(label_pair(label_name, values.join(";")));
    }

    labels
}

fn label_pair(name: impl Into<String>, value: impl Into<String>) -> (String, String) {
    (name.into(), value.into())
}

fn validate_metrics(
    name: &str,
    description: &str,
    metric_kind: MetricType,
    mfs: &mut HashMap<String, (MetricType, String)>,
) -> (bool, Option<String>) {
    if let Some((existing_kind, existing_help)) = mfs.get(name) {
        if existing_kind.as_str() != metric_kind.as_str() {
            otel_warn!(
                name: "MetricValidationFailed",
                message = "Instrument type conflict, using existing type definition",
                metric_type = format!("Instrument {name}, Existing: {:?}, dropped: {:?}", existing_kind, metric_kind).as_str(),
            );
            return (true, None);
        }
        if existing_help != description {
            otel_warn!(
                name: "MetricValidationFailed",
                message = "Instrument description conflict, using existing",
                metric_description = format!("Instrument {name}, Existing: {:?}, dropped: {:?}", existing_help, description).as_str(),
            );
            return (false, Some(existing_help.clone()));
        }
        (false, None)
    } else {
        mfs.insert(name.to_string(), (metric_kind, description.to_string()));

        (false, None)
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
