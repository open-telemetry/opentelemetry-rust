//! Types for delivery of pre-aggregated metric time series data.

use std::{any, borrow::Cow, fmt, time::SystemTime};

use opentelemetry_api::{metrics::Unit, KeyValue};

use crate::{attributes::AttributeSet, instrumentation::Scope, Resource};

pub use self::temporality::Temporality;

mod temporality;

/// A collection of [ScopeMetrics] and the associated [Resource] that created them.
#[derive(Debug)]
pub struct ResourceMetrics {
    /// The entity that collected the metrics.
    pub resource: Resource,
    /// The collection of metrics with unique [Scope]s.
    pub scope_metrics: Vec<ScopeMetrics>,
}

/// A collection of metrics produced by a meter.
#[derive(Default, Debug)]
pub struct ScopeMetrics {
    /// The [Scope] that the meter was created with.
    pub scope: Scope,
    /// The list of aggregations created by the meter.
    pub metrics: Vec<Metric>,
}

/// A collection of one or more aggregated time series from an [Instrument].
///
/// [Instrument]: crate::metrics::Instrument
#[derive(Debug)]
pub struct Metric {
    /// The name of the instrument that created this data.
    pub name: Cow<'static, str>,
    /// The description of the instrument, which can be used in documentation.
    pub description: Cow<'static, str>,
    /// The unit in which the instrument reports.
    pub unit: Unit,
    /// The aggregated data from an instrument.
    pub data: Box<dyn Aggregation>,
}

/// The store of data reported by an [Instrument].
///
/// It will be one of: [Gauge], [Sum], or [Histogram].
///
/// [Instrument]: crate::metrics::Instrument
pub trait Aggregation: fmt::Debug + any::Any + Send + Sync {
    /// Support downcasting
    fn as_any(&self) -> &dyn any::Any;
}

/// A measurement of the current value of an instrument.
#[derive(Debug)]
pub struct Gauge<T> {
    /// Represents individual aggregated measurements with unique attributes.
    pub data_points: Vec<DataPoint<T>>,
}

impl<T: fmt::Debug + Send + Sync + 'static> Aggregation for Gauge<T> {
    fn as_any(&self) -> &dyn any::Any {
        self
    }
}

/// Represents the sum of all measurements of values from an instrument.
#[derive(Debug)]
pub struct Sum<T> {
    /// Represents individual aggregated measurements with unique attributes.
    pub data_points: Vec<DataPoint<T>>,
    /// Describes if the aggregation is reported as the change from the last report
    /// time, or the cumulative changes since a fixed start time.
    pub temporality: Temporality,
    /// Whether this aggregation only increases or decreases.
    pub is_monotonic: bool,
}

impl<T: fmt::Debug + Send + Sync + 'static> Aggregation for Sum<T> {
    fn as_any(&self) -> &dyn any::Any {
        self
    }
}

/// DataPoint is a single data point in a time series.
#[derive(Debug)]
pub struct DataPoint<T> {
    /// Attributes is the set of key value pairs that uniquely identify the
    /// time series.
    pub attributes: AttributeSet,
    /// The time when the time series was started.
    pub start_time: Option<SystemTime>,
    /// The time when the time series was recorded.
    pub time: Option<SystemTime>,
    /// The value of this data point.
    pub value: T,
    /// The sampled [Exemplar]s collected during the time series.
    pub exemplars: Vec<Exemplar<T>>,
}

/// Represents the histogram of all measurements of values from an instrument.
#[derive(Debug)]
pub struct Histogram<T> {
    /// Individual aggregated measurements with unique attributes.
    pub data_points: Vec<HistogramDataPoint<T>>,
    /// Describes if the aggregation is reported as the change from the last report
    /// time, or the cumulative changes since a fixed start time.
    pub temporality: Temporality,
}

impl<T: fmt::Debug + Send + Sync + 'static> Aggregation for Histogram<T> {
    fn as_any(&self) -> &dyn any::Any {
        self
    }
}

/// A single histogram data point in a time series.
#[derive(Debug)]
pub struct HistogramDataPoint<T> {
    /// The set of key value pairs that uniquely identify the time series.
    pub attributes: AttributeSet,
    /// The time when the time series was started.
    pub start_time: SystemTime,
    /// The time when the time series was recorded.
    pub time: SystemTime,

    /// The number of updates this histogram has been calculated with.
    pub count: u64,
    /// The upper bounds of the buckets of the histogram.
    ///
    /// Because the last boundary is +infinity this one is implied.
    pub bounds: Vec<f64>,
    /// The count of each of the buckets.
    pub bucket_counts: Vec<u64>,

    /// The minimum value recorded.
    pub min: Option<f64>,
    /// The maximum value recorded.
    pub max: Option<f64>,
    /// The sum of the values recorded.
    pub sum: f64,

    /// The sampled [Exemplar]s collected during the time series.
    pub exemplars: Vec<Exemplar<T>>,
}

/// A measurement sampled from a time series providing a typical example.
#[derive(Debug)]
pub struct Exemplar<T> {
    /// The attributes recorded with the measurement but filtered out of the
    /// time series' aggregated data.
    pub filtered_attributes: Vec<KeyValue>,
    /// The time when the measurement was recorded.
    pub time: SystemTime,
    /// The measured value.
    pub value: T,
    /// The ID of the span that was active during the measurement.
    ///
    /// If no span was active or the span was not sampled this will be empty.
    pub span_id: [u8; 8],
    /// The ID of the trace the active span belonged to during the measurement.
    ///
    /// If no span was active or the span was not sampled this will be empty.
    pub trace_id: [u8; 16],
}
