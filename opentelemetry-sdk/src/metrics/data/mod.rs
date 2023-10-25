//! Types for delivery of pre-aggregated metric time series data.

use std::{any, borrow::Cow, fmt, time::SystemTime};

use opentelemetry::{metrics::Unit, KeyValue};

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
    /// Support downcasting during aggregation
    fn as_mut(&mut self) -> &mut dyn any::Any;
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
    fn as_mut(&mut self) -> &mut dyn any::Any {
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
    fn as_mut(&mut self) -> &mut dyn any::Any {
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

impl<T: Copy> Clone for DataPoint<T> {
    fn clone(&self) -> Self {
        Self {
            attributes: self.attributes.clone(),
            start_time: self.start_time,
            time: self.time,
            value: self.value,
            exemplars: self.exemplars.clone(),
        }
    }
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
    fn as_mut(&mut self) -> &mut dyn any::Any {
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
    pub min: Option<T>,
    /// The maximum value recorded.
    pub max: Option<T>,
    /// The sum of the values recorded.
    pub sum: T,

    /// The sampled [Exemplar]s collected during the time series.
    pub exemplars: Vec<Exemplar<T>>,
}

impl<T: Copy> Clone for HistogramDataPoint<T> {
    fn clone(&self) -> Self {
        Self {
            attributes: self.attributes.clone(),
            start_time: self.start_time,
            time: self.time,
            count: self.count,
            bounds: self.bounds.clone(),
            bucket_counts: self.bucket_counts.clone(),
            min: self.min,
            max: self.max,
            sum: self.sum,
            exemplars: self.exemplars.clone(),
        }
    }
}

/// The histogram of all measurements of values from an instrument.
#[derive(Debug)]
pub struct ExponentialHistogram<T> {
    /// The individual aggregated measurements with unique attributes.
    pub data_points: Vec<ExponentialHistogramDataPoint<T>>,

    /// Describes if the aggregation is reported as the change from the last report
    /// time, or the cumulative changes since a fixed start time.
    pub temporality: Temporality,
}

impl<T: fmt::Debug + Send + Sync + 'static> Aggregation for ExponentialHistogram<T> {
    fn as_any(&self) -> &dyn any::Any {
        self
    }
    fn as_mut(&mut self) -> &mut dyn any::Any {
        self
    }
}

/// A single exponential histogram data point in a time series.
#[derive(Debug)]
pub struct ExponentialHistogramDataPoint<T> {
    /// The set of key value pairs that uniquely identify the time series.
    pub attributes: AttributeSet,
    /// When the time series was started.
    pub start_time: SystemTime,
    /// The time when the time series was recorded.
    pub time: SystemTime,

    /// The number of updates this histogram has been calculated with.
    pub count: usize,
    /// The minimum value recorded.
    pub min: Option<T>,
    /// The maximum value recorded.
    pub max: Option<T>,
    /// The sum of the values recorded.
    pub sum: T,

    /// Describes the resolution of the histogram.
    ///
    /// Boundaries are located at powers of the base, where:
    ///
    ///   base = 2 ^ (2 ^ -scale)
    pub scale: i8,

    /// The number of values whose absolute value is less than or equal to
    /// `zero_threshold`.
    ///
    /// When `zero_threshold` is `0`, this is the number of values that cannot be
    /// expressed using the standard exponential formula as well as values that have
    /// been rounded to zero.
    pub zero_count: u64,

    /// The range of positive value bucket counts.
    pub positive_bucket: ExponentialBucket,
    /// The range of negative value bucket counts.
    pub negative_bucket: ExponentialBucket,

    /// The width of the zero region.
    ///
    /// Where the zero region is defined as the closed interval
    /// [-zero_threshold, zero_threshold].
    pub zero_threshold: f64,

    /// The sampled exemplars collected during the time series.
    pub exemplars: Vec<Exemplar<T>>,
}

/// A set of bucket counts, encoded in a contiguous array of counts.
#[derive(Debug, PartialEq)]
pub struct ExponentialBucket {
    /// The bucket index of the first entry in the `counts` vec.
    pub offset: i32,

    /// A vec where `counts[i]` carries the count of the bucket at index `offset + i`.
    ///
    /// `counts[i]` is the count of values greater than base^(offset+i) and less than
    /// or equal to base^(offset+i+1).
    pub counts: Vec<u64>,
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

impl<T: Copy> Clone for Exemplar<T> {
    fn clone(&self) -> Self {
        Self {
            filtered_attributes: self.filtered_attributes.clone(),
            time: self.time,
            value: self.value,
            span_id: self.span_id,
            trace_id: self.trace_id,
        }
    }
}
