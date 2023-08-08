use chrono::{LocalResult, TimeZone, Utc};
use opentelemetry_api::{global, metrics::MetricsError};
use opentelemetry_sdk::metrics::data;
use serde::{Serialize, Serializer};
use std::{
    any::Any,
    borrow::Cow,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::common::{AttributeSet, KeyValue, Resource, Scope};

/// Transformed metrics data that can be serialized
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MetricsData {
    resource_metrics: ResourceMetrics,
}

impl From<&mut data::ResourceMetrics> for MetricsData {
    fn from(value: &mut data::ResourceMetrics) -> Self {
        MetricsData {
            resource_metrics: value.into(),
        }
    }
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResourceMetrics {
    resource: Resource,
    scope_metrics: Vec<ScopeMetrics>,
    #[serde(skip_serializing_if = "Option::is_none")]
    schema_url: Option<String>,
}

impl From<&mut data::ResourceMetrics> for ResourceMetrics {
    fn from(value: &mut data::ResourceMetrics) -> Self {
        ResourceMetrics {
            resource: Resource::from(&value.resource),
            scope_metrics: value.scope_metrics.drain(..).map(Into::into).collect(),
            schema_url: value.resource.schema_url().map(Into::into),
        }
    }
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct ScopeMetrics {
    scope: Scope,
    metrics: Vec<Metric>,
    #[serde(skip_serializing_if = "Option::is_none")]
    schema_url: Option<Cow<'static, str>>,
}

impl From<data::ScopeMetrics> for ScopeMetrics {
    fn from(value: data::ScopeMetrics) -> Self {
        let schema_url = value.scope.schema_url.clone();
        ScopeMetrics {
            scope: value.scope.into(),
            metrics: value.metrics.into_iter().map(Into::into).collect(),
            schema_url,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
struct Unit(Cow<'static, str>);

impl Unit {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<opentelemetry_api::metrics::Unit> for Unit {
    fn from(unit: opentelemetry_api::metrics::Unit) -> Self {
        Unit(unit.as_str().to_string().into())
    }
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Metric {
    name: Cow<'static, str>,
    #[serde(skip_serializing_if = "str::is_empty")]
    description: Cow<'static, str>,
    #[serde(skip_serializing_if = "Unit::is_empty")]
    unit: Unit,
    #[serde(flatten)]
    data: Option<MetricData>,
}

impl From<data::Metric> for Metric {
    fn from(value: data::Metric) -> Self {
        Metric {
            name: value.name,
            description: value.description,
            unit: value.unit.into(),
            data: map_data(value.data.as_any()),
        }
    }
}

fn map_data(data: &dyn Any) -> Option<MetricData> {
    if let Some(hist) = data.downcast_ref::<data::Histogram<i64>>() {
        Some(MetricData::Histogram(hist.into()))
    } else if let Some(hist) = data.downcast_ref::<data::Histogram<u64>>() {
        Some(MetricData::Histogram(hist.into()))
    } else if let Some(hist) = data.downcast_ref::<data::Histogram<f64>>() {
        Some(MetricData::Histogram(hist.into()))
    } else if let Some(sum) = data.downcast_ref::<data::Sum<u64>>() {
        Some(MetricData::Sum(sum.into()))
    } else if let Some(sum) = data.downcast_ref::<data::Sum<i64>>() {
        Some(MetricData::Sum(sum.into()))
    } else if let Some(sum) = data.downcast_ref::<data::Sum<f64>>() {
        Some(MetricData::Sum(sum.into()))
    } else if let Some(gauge) = data.downcast_ref::<data::Gauge<u64>>() {
        Some(MetricData::Gauge(gauge.into()))
    } else if let Some(gauge) = data.downcast_ref::<data::Gauge<i64>>() {
        Some(MetricData::Gauge(gauge.into()))
    } else if let Some(gauge) = data.downcast_ref::<data::Gauge<f64>>() {
        Some(MetricData::Gauge(gauge.into()))
    } else {
        global::handle_error(MetricsError::Other("unknown aggregator".into()));
        None
    }
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
enum MetricData {
    Gauge(Gauge),
    Sum(Sum),
    Histogram(Histogram),
}

#[derive(Serialize, Debug, Clone)]
#[serde(untagged)]
enum DataValue {
    F64(f64),
    I64(i64),
    U64(u64),
}

impl From<f64> for DataValue {
    fn from(value: f64) -> Self {
        DataValue::F64(value)
    }
}

impl From<i64> for DataValue {
    fn from(value: i64) -> Self {
        DataValue::I64(value)
    }
}

impl From<u64> for DataValue {
    fn from(value: u64) -> Self {
        DataValue::U64(value)
    }
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Gauge {
    data_points: Vec<DataPoint>,
}

impl<T: Into<DataValue> + Copy> From<&data::Gauge<T>> for Gauge {
    fn from(value: &data::Gauge<T>) -> Self {
        Gauge {
            data_points: value.data_points.iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Temporality {
    #[allow(dead_code)]
    Unspecified = 0, // explicitly never used
    Delta = 1,
    Cumulative = 2,
}

impl Serialize for Temporality {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(*self as u32 as u8)
    }
}

impl From<data::Temporality> for Temporality {
    fn from(value: data::Temporality) -> Self {
        match value {
            data::Temporality::Cumulative => Temporality::Cumulative,
            data::Temporality::Delta => Temporality::Delta,
            _ => panic!("unexpected temporality"),
        }
    }
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Sum {
    data_points: Vec<DataPoint>,
    aggregation_temporality: Temporality,
    is_monotonic: bool,
}

impl<T: Into<DataValue> + Copy> From<&data::Sum<T>> for Sum {
    fn from(value: &data::Sum<T>) -> Self {
        Sum {
            data_points: value.data_points.iter().map(Into::into).collect(),
            aggregation_temporality: value.temporality.into(),
            is_monotonic: value.is_monotonic,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct DataPoint {
    attributes: AttributeSet,
    #[serde(serialize_with = "as_opt_human_readable")]
    start_time: Option<SystemTime>,
    #[serde(serialize_with = "as_opt_human_readable")]
    time: Option<SystemTime>,
    value: DataValue,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    exemplars: Vec<Exemplar>,
    #[serde(skip_serializing_if = "is_zero_u8")]
    flags: u8,
}

fn is_zero_u8(v: &u8) -> bool {
    *v == 0
}

impl<T: Into<DataValue> + Copy> From<&data::DataPoint<T>> for DataPoint {
    fn from(value: &data::DataPoint<T>) -> Self {
        DataPoint {
            attributes: AttributeSet::from(&value.attributes),
            start_time: value.start_time,
            time: value.time,
            value: value.value.into(),
            exemplars: value.exemplars.iter().map(Into::into).collect(),
            flags: 0,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Histogram {
    data_points: Vec<HistogramDataPoint>,
    aggregation_temporality: Temporality,
}

impl<T: Into<DataValue> + Copy> From<&data::Histogram<T>> for Histogram {
    fn from(value: &data::Histogram<T>) -> Self {
        Histogram {
            data_points: value.data_points.iter().map(Into::into).collect(),
            aggregation_temporality: value.temporality.into(),
        }
    }
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct HistogramDataPoint {
    attributes: AttributeSet,
    #[serde(serialize_with = "as_human_readable")]
    start_time: SystemTime,
    #[serde(serialize_with = "as_human_readable")]
    time: SystemTime,
    count: u64,
    explicit_bounds: Vec<f64>,
    bucket_counts: Vec<u64>,
    min: Option<DataValue>,
    max: Option<DataValue>,
    sum: DataValue,
    exemplars: Vec<Exemplar>,
    flags: u8,
}

impl<T: Into<DataValue> + Copy> From<&data::HistogramDataPoint<T>> for HistogramDataPoint {
    fn from(value: &data::HistogramDataPoint<T>) -> Self {
        HistogramDataPoint {
            attributes: AttributeSet::from(&value.attributes),
            start_time: value.start_time,
            time: value.time,
            count: value.count,
            explicit_bounds: value.bounds.clone(),
            bucket_counts: value.bucket_counts.clone(),
            min: value.min.map(Into::into),
            max: value.max.map(Into::into),
            sum: value.sum.into(),
            exemplars: value.exemplars.iter().map(Into::into).collect(),
            flags: 0,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Exemplar {
    filtered_attributes: Vec<KeyValue>,
    #[serde(serialize_with = "as_human_readable")]
    time: SystemTime,
    value: DataValue,
    span_id: String,
    trace_id: String,
}

fn as_human_readable<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let duration_since_epoch = time.duration_since(UNIX_EPOCH).unwrap_or_default();

    match Utc.timestamp_opt(
        duration_since_epoch.as_secs() as i64,
        duration_since_epoch.subsec_nanos(),
    ) {
        LocalResult::Single(datetime) => {
            let human_readable_date = datetime.format("%Y-%m-%d %H:%M:%S.%3f").to_string();
            serializer.serialize_str(&human_readable_date)
        }
        _ => Err(serde::ser::Error::custom("Invalid Timestamp.")),
    }
}

fn as_opt_human_readable<S>(time: &Option<SystemTime>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match time {
        None => serializer.serialize_none(),
        Some(time) => as_human_readable(time, serializer),
    }
}

impl<T: Into<DataValue> + Copy> From<&data::Exemplar<T>> for Exemplar {
    fn from(value: &data::Exemplar<T>) -> Self {
        Exemplar {
            filtered_attributes: value.filtered_attributes.iter().map(Into::into).collect(),
            time: value.time,
            value: value.value.into(),
            span_id: format!("{:016x}", u64::from_be_bytes(value.span_id)),
            trace_id: format!("{:032x}", u128::from_be_bytes(value.trace_id)),
        }
    }
}
