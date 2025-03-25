//! Types for delivery of pre-aggregated metric time series data.

use std::{borrow::Cow, time::SystemTime};

use opentelemetry::{InstrumentationScope, KeyValue};

use crate::Resource;

use super::Temporality;

/// A collection of [ScopeMetrics] and the associated [Resource] that created them.
#[derive(Debug)]
pub struct ResourceMetrics {
    /// The entity that collected the metrics.
    pub resource: Resource,
    /// The collection of metrics with unique [InstrumentationScope]s.
    pub scope_metrics: Vec<ScopeMetrics>,
}

/// A collection of metrics produced by a meter.
#[derive(Default, Debug)]
pub struct ScopeMetrics {
    /// The [InstrumentationScope] that the meter was created with.
    pub scope: InstrumentationScope,
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
    pub unit: Cow<'static, str>,
    /// The aggregated data from an instrument.
    pub data: AggregatedMetrics,
}

/// Aggregated metrics data from an instrument
#[derive(Debug)]
pub enum AggregatedMetrics {
    /// All metric data with `f64` value type
    F64(MetricData<f64>),
    /// All metric data with `u64` value type
    U64(MetricData<u64>),
    /// All metric data with `i64` value type
    I64(MetricData<i64>),
}

/// Metric data for all types
#[derive(Debug)]
pub enum MetricData<T> {
    /// Metric data for Gauge
    Gauge(Gauge<T>),
    /// Metric data for Sum
    Sum(Sum<T>),
    /// Metric data for Histogram
    Histogram(Histogram<T>),
    /// Metric data for ExponentialHistogram
    ExponentialHistogram(ExponentialHistogram<T>),
}

impl From<MetricData<f64>> for AggregatedMetrics {
    fn from(value: MetricData<f64>) -> Self {
        AggregatedMetrics::F64(value)
    }
}

impl From<MetricData<i64>> for AggregatedMetrics {
    fn from(value: MetricData<i64>) -> Self {
        AggregatedMetrics::I64(value)
    }
}

impl From<MetricData<u64>> for AggregatedMetrics {
    fn from(value: MetricData<u64>) -> Self {
        AggregatedMetrics::U64(value)
    }
}

impl<T> From<Gauge<T>> for MetricData<T> {
    fn from(value: Gauge<T>) -> Self {
        MetricData::Gauge(value)
    }
}

impl<T> From<Sum<T>> for MetricData<T> {
    fn from(value: Sum<T>) -> Self {
        MetricData::Sum(value)
    }
}

impl<T> From<Histogram<T>> for MetricData<T> {
    fn from(value: Histogram<T>) -> Self {
        MetricData::Histogram(value)
    }
}

impl<T> From<ExponentialHistogram<T>> for MetricData<T> {
    fn from(value: ExponentialHistogram<T>) -> Self {
        MetricData::ExponentialHistogram(value)
    }
}

/// DataPoint is a single data point in a time series.
#[derive(Debug, Clone, PartialEq)]
pub struct GaugeDataPoint<T> {
    /// Attributes is the set of key value pairs that uniquely identify the
    /// time series.
    pub attributes: Vec<KeyValue>,
    /// The value of this data point.
    pub value: T,
    /// The sampled [Exemplar]s collected during the time series.
    pub exemplars: Vec<Exemplar<T>>,
}

/// A measurement of the current value of an instrument.
#[derive(Debug, Clone)]
pub struct Gauge<T> {
    /// Represents individual aggregated measurements with unique attributes.
    pub data_points: Vec<GaugeDataPoint<T>>,
    /// The time when the time series was started.
    pub start_time: Option<SystemTime>,
    /// The time when the time series was recorded.
    pub time: SystemTime,
}

/// DataPoint is a single data point in a time series.
#[derive(Debug, Clone, PartialEq)]
pub struct SumDataPoint<T> {
    /// Attributes is the set of key value pairs that uniquely identify the
    /// time series.
    pub attributes: Vec<KeyValue>,
    /// The value of this data point.
    pub value: T,
    /// The sampled [Exemplar]s collected during the time series.
    pub exemplars: Vec<Exemplar<T>>,
}

/// Represents the sum of all measurements of values from an instrument.
#[derive(Debug, Clone)]
pub struct Sum<T> {
    /// Represents individual aggregated measurements with unique attributes.
    pub data_points: Vec<SumDataPoint<T>>,
    /// The time when the time series was started.
    pub start_time: SystemTime,
    /// The time when the time series was recorded.
    pub time: SystemTime,
    /// Describes if the aggregation is reported as the change from the last report
    /// time, or the cumulative changes since a fixed start time.
    pub temporality: Temporality,
    /// Whether this aggregation only increases or decreases.
    pub is_monotonic: bool,
}

/// Represents the histogram of all measurements of values from an instrument.
#[derive(Debug, Clone)]
pub struct Histogram<T> {
    /// Individual aggregated measurements with unique attributes.
    pub data_points: Vec<HistogramDataPoint<T>>,
    /// The time when the time series was started.
    pub start_time: SystemTime,
    /// The time when the time series was recorded.
    pub time: SystemTime,
    /// Describes if the aggregation is reported as the change from the last report
    /// time, or the cumulative changes since a fixed start time.
    pub temporality: Temporality,
}

/// A single histogram data point in a time series.
#[derive(Debug, Clone, PartialEq)]
pub struct HistogramDataPoint<T> {
    /// The set of key value pairs that uniquely identify the time series.
    pub attributes: Vec<KeyValue>,
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

/// The histogram of all measurements of values from an instrument.
#[derive(Debug, Clone)]
pub struct ExponentialHistogram<T> {
    /// The individual aggregated measurements with unique attributes.
    pub data_points: Vec<ExponentialHistogramDataPoint<T>>,
    /// When the time series was started.
    pub start_time: SystemTime,
    /// The time when the time series was recorded.
    pub time: SystemTime,
    /// Describes if the aggregation is reported as the change from the last report
    /// time, or the cumulative changes since a fixed start time.
    pub temporality: Temporality,
}

/// A single exponential histogram data point in a time series.
#[derive(Debug, Clone, PartialEq)]
pub struct ExponentialHistogramDataPoint<T> {
    /// The set of key value pairs that uniquely identify the time series.
    pub attributes: Vec<KeyValue>,

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
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
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

#[cfg(test)]
mod tests {

    use super::{Exemplar, ExponentialHistogramDataPoint, HistogramDataPoint, SumDataPoint};

    use opentelemetry::time::now;
    use opentelemetry::KeyValue;

    #[test]
    fn validate_cloning_data_points() {
        let data_type = SumDataPoint {
            attributes: vec![KeyValue::new("key", "value")],
            value: 0u32,
            exemplars: vec![Exemplar {
                filtered_attributes: vec![],
                time: now(),
                value: 0u32,
                span_id: [0; 8],
                trace_id: [0; 16],
            }],
        };
        assert_eq!(data_type.clone(), data_type);

        let histogram_data_point = HistogramDataPoint {
            attributes: vec![KeyValue::new("key", "value")],
            count: 0,
            bounds: vec![],
            bucket_counts: vec![],
            min: None,
            max: None,
            sum: 0u32,
            exemplars: vec![Exemplar {
                filtered_attributes: vec![],
                time: now(),
                value: 0u32,
                span_id: [0; 8],
                trace_id: [0; 16],
            }],
        };
        assert_eq!(histogram_data_point.clone(), histogram_data_point);

        let exponential_histogram_data_point = ExponentialHistogramDataPoint {
            attributes: vec![KeyValue::new("key", "value")],
            count: 0,
            min: None,
            max: None,
            sum: 0u32,
            scale: 0,
            zero_count: 0,
            positive_bucket: super::ExponentialBucket {
                offset: 0,
                counts: vec![],
            },
            negative_bucket: super::ExponentialBucket {
                offset: 0,
                counts: vec![],
            },
            zero_threshold: 0.0,
            exemplars: vec![Exemplar {
                filtered_attributes: vec![],
                time: now(),
                value: 0u32,
                span_id: [0; 8],
                trace_id: [0; 16],
            }],
        };
        assert_eq!(
            exponential_histogram_data_point.clone(),
            exponential_histogram_data_point
        );
    }
}
