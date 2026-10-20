//! Types for delivery of pre-aggregated metric time series data.

use std::{borrow::Cow, time::SystemTime};

use opentelemetry::{InstrumentationScope, KeyValue};

use crate::Resource;

use super::Temporality;

#[cfg(feature = "experimental_metrics_producer")]
use super::instrument::validate_bucket_boundaries;
#[cfg(feature = "experimental_metrics_producer")]
use crate::error::OTelSdkError;

/// A collection of [ScopeMetrics] and the associated [Resource] that created them.
#[derive(Debug)]
pub struct ResourceMetrics {
    /// The entity that collected the metrics.
    pub(crate) resource: Resource,
    /// The collection of metrics with unique [InstrumentationScope]s.
    pub(crate) scope_metrics: Vec<ScopeMetrics>,
}

impl Default for ResourceMetrics {
    fn default() -> Self {
        Self {
            resource: Resource::empty(),
            scope_metrics: Vec::new(),
        }
    }
}

impl ResourceMetrics {
    /// Returns a reference to the [Resource] in [ResourceMetrics].
    pub fn resource(&self) -> &Resource {
        &self.resource
    }

    /// Returns an iterator over the [ScopeMetrics] in [ResourceMetrics].
    pub fn scope_metrics(&self) -> impl Iterator<Item = &ScopeMetrics> {
        self.scope_metrics.iter()
    }
}

/// A collection of metrics produced by a meter.
#[derive(Default, Debug)]
pub struct ScopeMetrics {
    /// The [InstrumentationScope] that the meter was created with.
    pub(crate) scope: InstrumentationScope,
    /// The list of aggregations created by the meter.
    pub(crate) metrics: Vec<Metric>,
}

impl ScopeMetrics {
    /// Returns a reference to the [InstrumentationScope] in [ScopeMetrics].
    pub fn scope(&self) -> &InstrumentationScope {
        &self.scope
    }

    /// Returns an iterator over the [Metric]s in [ScopeMetrics].
    pub fn metrics(&self) -> impl Iterator<Item = &Metric> {
        self.metrics.iter()
    }
}

/// A collection of one or more aggregated time series from an [Instrument].
///
/// [Instrument]: crate::metrics::Instrument
#[derive(Debug)]
pub struct Metric {
    /// The name of the instrument that created this data.
    pub(crate) name: Cow<'static, str>,
    /// The description of the instrument, which can be used in documentation.
    pub(crate) description: Cow<'static, str>,
    /// The unit in which the instrument reports.
    pub(crate) unit: Cow<'static, str>,
    /// The aggregated data from an instrument.
    pub(crate) data: AggregatedMetrics,
}

impl Metric {
    /// Returns the name of the instrument that created this data.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the description of the instrument.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns the unit in which the instrument reports.
    pub fn unit(&self) -> &str {
        &self.unit
    }

    /// Returns the aggregated data from the instrument.
    pub fn data(&self) -> &AggregatedMetrics {
        &self.data
    }
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
    pub(crate) attributes: Vec<KeyValue>,
    /// The value of this data point.
    pub(crate) value: T,
    /// The sampled [Exemplar]s collected during the time series.
    pub(crate) exemplars: Vec<Exemplar<T>>,
}

impl<T> GaugeDataPoint<T> {
    /// Returns an iterator over the attributes in [GaugeDataPoint].
    pub fn attributes(&self) -> impl Iterator<Item = &KeyValue> {
        self.attributes.iter()
    }

    /// Returns an iterator over the [Exemplar]s in [GaugeDataPoint].
    pub fn exemplars(&self) -> impl Iterator<Item = &Exemplar<T>> {
        self.exemplars.iter()
    }
}

impl<T: Copy> GaugeDataPoint<T> {
    /// Returns the value of this data point.
    pub fn value(&self) -> T {
        self.value
    }
}

/// A measurement of the current value of an instrument.
#[derive(Debug, Clone)]
pub struct Gauge<T> {
    /// Represents individual aggregated measurements with unique attributes.
    pub(crate) data_points: Vec<GaugeDataPoint<T>>,
    /// The time when the time series was started.
    pub(crate) start_time: Option<SystemTime>,
    /// The time when the time series was recorded.
    pub(crate) time: SystemTime,
}

impl<T> Gauge<T> {
    /// Returns an iterator over the [GaugeDataPoint]s in [Gauge].
    pub fn data_points(&self) -> impl Iterator<Item = &GaugeDataPoint<T>> {
        self.data_points.iter()
    }

    /// Returns the time when the time series was started.
    pub fn start_time(&self) -> Option<SystemTime> {
        self.start_time
    }

    /// Returns the time when the time series was recorded.
    pub fn time(&self) -> SystemTime {
        self.time
    }
}

/// DataPoint is a single data point in a time series.
#[derive(Debug, Clone, PartialEq)]
pub struct SumDataPoint<T> {
    /// Attributes is the set of key value pairs that uniquely identify the
    /// time series.
    pub(crate) attributes: Vec<KeyValue>,
    /// The value of this data point.
    pub(crate) value: T,
    /// The sampled [Exemplar]s collected during the time series.
    pub(crate) exemplars: Vec<Exemplar<T>>,
}

impl<T> SumDataPoint<T> {
    /// Returns an iterator over the attributes in [SumDataPoint].
    pub fn attributes(&self) -> impl Iterator<Item = &KeyValue> {
        self.attributes.iter()
    }

    /// Returns an iterator over the [Exemplar]s in [SumDataPoint].
    pub fn exemplars(&self) -> impl Iterator<Item = &Exemplar<T>> {
        self.exemplars.iter()
    }
}

impl<T: Copy> SumDataPoint<T> {
    /// Returns the value of this data point.
    pub fn value(&self) -> T {
        self.value
    }
}

/// Represents the sum of all measurements of values from an instrument.
#[derive(Debug, Clone)]
pub struct Sum<T> {
    /// Represents individual aggregated measurements with unique attributes.
    pub(crate) data_points: Vec<SumDataPoint<T>>,
    /// The time when the time series was started.
    pub(crate) start_time: SystemTime,
    /// The time when the time series was recorded.
    pub(crate) time: SystemTime,
    /// Describes if the aggregation is reported as the change from the last report
    /// time, or the cumulative changes since a fixed start time.
    pub(crate) temporality: Temporality,
    /// Whether this aggregation only increases or decreases.
    pub(crate) is_monotonic: bool,
}

impl<T> Sum<T> {
    /// Returns an iterator over the [SumDataPoint]s in [Sum].
    pub fn data_points(&self) -> impl Iterator<Item = &SumDataPoint<T>> {
        self.data_points.iter()
    }

    /// Returns the time when the time series was started.
    pub fn start_time(&self) -> SystemTime {
        self.start_time
    }

    /// Returns the time when the time series was recorded.
    pub fn time(&self) -> SystemTime {
        self.time
    }

    /// Returns the temporality describing if the aggregation is reported as the change
    /// from the last report time, or the cumulative changes since a fixed start time.
    pub fn temporality(&self) -> Temporality {
        self.temporality
    }

    /// Returns whether this aggregation only increases or decreases.
    pub fn is_monotonic(&self) -> bool {
        self.is_monotonic
    }
}

/// Represents the histogram of all measurements of values from an instrument.
#[derive(Debug, Clone)]
pub struct Histogram<T> {
    /// Individual aggregated measurements with unique attributes.
    pub(crate) data_points: Vec<HistogramDataPoint<T>>,
    /// The time when the time series was started.
    pub(crate) start_time: SystemTime,
    /// The time when the time series was recorded.
    pub(crate) time: SystemTime,
    /// Describes if the aggregation is reported as the change from the last report
    /// time, or the cumulative changes since a fixed start time.
    pub(crate) temporality: Temporality,
}

impl<T> Histogram<T> {
    /// Returns an iterator over the [HistogramDataPoint]s in [Histogram].
    pub fn data_points(&self) -> impl Iterator<Item = &HistogramDataPoint<T>> {
        self.data_points.iter()
    }

    /// Returns the time when the time series was started.
    pub fn start_time(&self) -> SystemTime {
        self.start_time
    }

    /// Returns the time when the time series was recorded.
    pub fn time(&self) -> SystemTime {
        self.time
    }

    /// Returns the temporality describing if the aggregation is reported as the change
    /// from the last report time, or the cumulative changes since a fixed start time.
    pub fn temporality(&self) -> Temporality {
        self.temporality
    }
}

/// A single histogram data point in a time series.
#[derive(Debug, Clone, PartialEq)]
pub struct HistogramDataPoint<T> {
    /// The set of key value pairs that uniquely identify the time series.
    pub(crate) attributes: Vec<KeyValue>,
    /// The number of updates this histogram has been calculated with.
    pub(crate) count: u64,
    /// The upper bounds of the buckets of the histogram.
    ///
    /// Because the last boundary is +infinity this one is implied.
    pub(crate) bounds: Vec<f64>,
    /// The count of each of the buckets.
    pub(crate) bucket_counts: Vec<u64>,

    /// The minimum value recorded.
    pub(crate) min: Option<T>,
    /// The maximum value recorded.
    pub(crate) max: Option<T>,
    /// The sum of the values recorded.
    pub(crate) sum: T,

    /// The sampled [Exemplar]s collected during the time series.
    pub(crate) exemplars: Vec<Exemplar<T>>,
}

impl<T> HistogramDataPoint<T> {
    /// Returns an iterator over the attributes in [HistogramDataPoint].
    pub fn attributes(&self) -> impl Iterator<Item = &KeyValue> {
        self.attributes.iter()
    }

    /// Returns an iterator over the exemplars in [HistogramDataPoint].
    pub fn exemplars(&self) -> impl Iterator<Item = &Exemplar<T>> {
        self.exemplars.iter()
    }

    /// Returns an iterator over the bucket boundaries in [HistogramDataPoint].
    pub fn bounds(&self) -> impl Iterator<Item = f64> + '_ {
        self.bounds.iter().copied()
    }

    /// Returns an iterator over the bucket counts in [HistogramDataPoint].
    pub fn bucket_counts(&self) -> impl Iterator<Item = u64> + '_ {
        self.bucket_counts.iter().copied()
    }

    /// Returns the number of updates this histogram has been calculated with.
    pub fn count(&self) -> u64 {
        self.count
    }
}

impl<T: Copy> HistogramDataPoint<T> {
    /// Returns the minimum value recorded.
    pub fn min(&self) -> Option<T> {
        self.min
    }

    /// Returns the maximum value recorded.
    pub fn max(&self) -> Option<T> {
        self.max
    }

    /// Returns the sum of the values recorded.
    pub fn sum(&self) -> T {
        self.sum
    }
}

/// The histogram of all measurements of values from an instrument.
#[derive(Debug, Clone)]
pub struct ExponentialHistogram<T> {
    /// The individual aggregated measurements with unique attributes.
    pub(crate) data_points: Vec<ExponentialHistogramDataPoint<T>>,
    /// When the time series was started.
    pub(crate) start_time: SystemTime,
    /// The time when the time series was recorded.
    pub(crate) time: SystemTime,
    /// Describes if the aggregation is reported as the change from the last report
    /// time, or the cumulative changes since a fixed start time.
    pub(crate) temporality: Temporality,
}

impl<T> ExponentialHistogram<T> {
    /// Returns an iterator over the [ExponentialHistogramDataPoint]s in [ExponentialHistogram].
    pub fn data_points(&self) -> impl Iterator<Item = &ExponentialHistogramDataPoint<T>> {
        self.data_points.iter()
    }

    /// Returns the time when the time series was started.
    pub fn start_time(&self) -> SystemTime {
        self.start_time
    }

    /// Returns the time when the time series was recorded.
    pub fn time(&self) -> SystemTime {
        self.time
    }

    /// Returns the temporality describing if the aggregation is reported as the change
    /// from the last report time, or the cumulative changes since a fixed start time.
    pub fn temporality(&self) -> Temporality {
        self.temporality
    }
}

/// A single exponential histogram data point in a time series.
#[derive(Debug, Clone, PartialEq)]
pub struct ExponentialHistogramDataPoint<T> {
    /// The set of key value pairs that uniquely identify the time series.
    pub(crate) attributes: Vec<KeyValue>,

    /// The number of updates this histogram has been calculated with.
    pub(crate) count: usize,
    /// The minimum value recorded.
    pub(crate) min: Option<T>,
    /// The maximum value recorded.
    pub(crate) max: Option<T>,
    /// The sum of the values recorded.
    pub(crate) sum: T,

    /// Describes the resolution of the histogram.
    ///
    /// Boundaries are located at powers of the base, where:
    ///
    ///   base = 2 ^ (2 ^ -scale)
    pub(crate) scale: i8,

    /// The number of values whose absolute value is less than or equal to
    /// `zero_threshold`.
    ///
    /// When `zero_threshold` is `0`, this is the number of values that cannot be
    /// expressed using the standard exponential formula as well as values that have
    /// been rounded to zero.
    pub(crate) zero_count: u64,

    /// The range of positive value bucket counts.
    pub(crate) positive_bucket: ExponentialBucket,
    /// The range of negative value bucket counts.
    pub(crate) negative_bucket: ExponentialBucket,

    /// The width of the zero region.
    ///
    /// Where the zero region is defined as the closed interval
    /// [-zero_threshold, zero_threshold].
    pub(crate) zero_threshold: f64,

    /// The sampled exemplars collected during the time series.
    pub(crate) exemplars: Vec<Exemplar<T>>,
}

impl<T> ExponentialHistogramDataPoint<T> {
    /// Returns an iterator over the attributes in [ExponentialHistogramDataPoint].
    pub fn attributes(&self) -> impl Iterator<Item = &KeyValue> {
        self.attributes.iter()
    }

    /// Returns an iterator over the exemplars in [ExponentialHistogramDataPoint].
    pub fn exemplars(&self) -> impl Iterator<Item = &Exemplar<T>> {
        self.exemplars.iter()
    }

    /// Returns the number of updates this histogram has been calculated with.
    pub fn count(&self) -> usize {
        self.count
    }

    /// Returns the resolution of the histogram.
    pub fn scale(&self) -> i8 {
        self.scale
    }

    /// Returns the number of values whose absolute value is less than or equal to zero_threshold.
    pub fn zero_count(&self) -> u64 {
        self.zero_count
    }

    /// Returns the range of positive value bucket counts.
    pub fn positive_bucket(&self) -> &ExponentialBucket {
        &self.positive_bucket
    }

    /// Returns the range of negative value bucket counts.
    pub fn negative_bucket(&self) -> &ExponentialBucket {
        &self.negative_bucket
    }

    /// Returns the width of the zero region.
    pub fn zero_threshold(&self) -> f64 {
        self.zero_threshold
    }
}

impl<T: Copy> ExponentialHistogramDataPoint<T> {
    /// Returns the minimum value recorded.
    pub fn min(&self) -> Option<T> {
        self.min
    }

    /// Returns the maximum value recorded.
    pub fn max(&self) -> Option<T> {
        self.max
    }

    /// Returns the sum of the values recorded.
    pub fn sum(&self) -> T {
        self.sum
    }
}

/// A set of bucket counts, encoded in a contiguous array of counts.
#[derive(Debug, Clone, PartialEq)]
pub struct ExponentialBucket {
    /// The bucket index of the first entry in the `counts` vec.
    pub(crate) offset: i32,

    /// A vec where `counts[i]` carries the count of the bucket at index `offset + i`.
    ///
    /// `counts[i]` is the count of values greater than base^(offset+i) and less than
    /// or equal to base^(offset+i+1).
    pub(crate) counts: Vec<u64>,
}

impl ExponentialBucket {
    /// Returns the bucket index of the first entry in the counts vec.
    pub fn offset(&self) -> i32 {
        self.offset
    }

    /// Returns an iterator over the counts.
    pub fn counts(&self) -> impl Iterator<Item = u64> + '_ {
        self.counts.iter().copied()
    }
}

/// A measurement sampled from a time series providing a typical example.
#[derive(Debug, Clone, PartialEq)]
pub struct Exemplar<T> {
    /// The attributes recorded with the measurement but filtered out of the
    /// time series' aggregated data.
    pub(crate) filtered_attributes: Vec<KeyValue>,
    /// The time when the measurement was recorded.
    pub(crate) time: SystemTime,
    /// The measured value.
    pub value: T,
    /// The ID of the span that was active during the measurement.
    ///
    /// If no span was active or the span was not sampled this will be empty.
    pub(crate) span_id: [u8; 8],
    /// The ID of the trace the active span belonged to during the measurement.
    ///
    /// If no span was active or the span was not sampled this will be empty.
    pub(crate) trace_id: [u8; 16],
}

impl<T> Exemplar<T> {
    /// Returns an iterator over the filtered attributes in [Exemplar].
    pub fn filtered_attributes(&self) -> impl Iterator<Item = &KeyValue> {
        self.filtered_attributes.iter()
    }

    /// Returns the time when the measurement was recorded.
    pub fn time(&self) -> SystemTime {
        self.time
    }

    /// Returns the ID of the span that was active during the measurement.
    pub fn span_id(&self) -> &[u8; 8] {
        &self.span_id
    }

    /// Returns the ID of the trace the active span belonged to during the measurement.
    pub fn trace_id(&self) -> &[u8; 16] {
        &self.trace_id
    }
}

// Constructors, for [MetricProducer] implementations that hand the SDK metrics
// aggregated elsewhere.
//
// [MetricProducer]: crate::metrics::MetricProducer

/// Sums histogram bucket counts, or fails if the total overflows.
#[cfg(feature = "experimental_metrics_producer")]
fn total_count(counts: impl IntoIterator<Item = u64>) -> Result<u64, OTelSdkError> {
    counts
        .into_iter()
        .try_fold(0u64, u64::checked_add)
        .ok_or_else(|| OTelSdkError::InternalFailure("histogram count overflows u64".into()))
}

#[cfg(feature = "experimental_metrics_producer")]
impl ScopeMetrics {
    /// Creates the metrics of one instrumentation scope.
    pub fn new(scope: InstrumentationScope, metrics: Vec<Metric>) -> Self {
        Self { scope, metrics }
    }
}

#[cfg(feature = "experimental_metrics_producer")]
impl Metric {
    /// Creates a metric from its instrument's name, description and unit, and its
    /// aggregated data.
    pub fn new(
        name: impl Into<Cow<'static, str>>,
        description: impl Into<Cow<'static, str>>,
        unit: impl Into<Cow<'static, str>>,
        data: impl Into<AggregatedMetrics>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            unit: unit.into(),
            data: data.into(),
        }
    }
}

#[cfg(feature = "experimental_metrics_producer")]
impl<T> Gauge<T> {
    /// Creates a gauge from its data points.
    pub fn new(
        data_points: Vec<GaugeDataPoint<T>>,
        start_time: Option<SystemTime>,
        time: SystemTime,
    ) -> Self {
        Self {
            data_points,
            start_time,
            time,
        }
    }
}

#[cfg(feature = "experimental_metrics_producer")]
impl<T> GaugeDataPoint<T> {
    /// Creates a gauge data point, with no exemplars.
    pub fn new(attributes: Vec<KeyValue>, value: T) -> Self {
        Self {
            attributes,
            value,
            exemplars: Vec::new(),
        }
    }

    /// Sets the exemplars.
    pub fn with_exemplars(mut self, exemplars: Vec<Exemplar<T>>) -> Self {
        self.exemplars = exemplars;
        self
    }
}

#[cfg(feature = "experimental_metrics_producer")]
impl<T> Sum<T> {
    /// Creates a sum from its data points.
    pub fn new(
        data_points: Vec<SumDataPoint<T>>,
        start_time: SystemTime,
        time: SystemTime,
        temporality: Temporality,
        is_monotonic: bool,
    ) -> Self {
        Self {
            data_points,
            start_time,
            time,
            temporality,
            is_monotonic,
        }
    }
}

#[cfg(feature = "experimental_metrics_producer")]
impl<T> SumDataPoint<T> {
    /// Creates a sum data point, with no exemplars.
    pub fn new(attributes: Vec<KeyValue>, value: T) -> Self {
        Self {
            attributes,
            value,
            exemplars: Vec::new(),
        }
    }

    /// Sets the exemplars.
    pub fn with_exemplars(mut self, exemplars: Vec<Exemplar<T>>) -> Self {
        self.exemplars = exemplars;
        self
    }
}

#[cfg(feature = "experimental_metrics_producer")]
impl<T> Histogram<T> {
    /// Creates a histogram from its data points.
    pub fn new(
        data_points: Vec<HistogramDataPoint<T>>,
        start_time: SystemTime,
        time: SystemTime,
        temporality: Temporality,
    ) -> Self {
        Self {
            data_points,
            start_time,
            time,
            temporality,
        }
    }
}

#[cfg(feature = "experimental_metrics_producer")]
impl<T> HistogramDataPoint<T> {
    /// Creates a histogram data point, with no min, max or exemplars.
    ///
    /// `bounds` must be finite and strictly increasing. `bucket_counts` has one more
    /// entry than `bounds`: the last one counts the values above the last bound. The
    /// count is the sum of `bucket_counts`.
    ///
    /// Returns an error if `bounds` or the length of `bucket_counts` is invalid, or
    /// if the count overflows.
    pub fn new(
        attributes: Vec<KeyValue>,
        bounds: Vec<f64>,
        bucket_counts: Vec<u64>,
        sum: T,
    ) -> Result<Self, OTelSdkError> {
        validate_bucket_boundaries(&bounds).map_err(OTelSdkError::InternalFailure)?;
        if bucket_counts.len() != bounds.len() + 1 {
            return Err(OTelSdkError::InternalFailure(format!(
                "histogram has {} bounds, so it needs {} bucket counts, not {}",
                bounds.len(),
                bounds.len() + 1,
                bucket_counts.len()
            )));
        }
        Ok(Self {
            attributes,
            count: total_count(bucket_counts.iter().copied())?,
            bounds,
            bucket_counts,
            min: None,
            max: None,
            sum,
            exemplars: Vec::new(),
        })
    }

    /// Sets the minimum value recorded.
    pub fn with_min(mut self, min: T) -> Self {
        self.min = Some(min);
        self
    }

    /// Sets the maximum value recorded.
    pub fn with_max(mut self, max: T) -> Self {
        self.max = Some(max);
        self
    }

    /// Sets the exemplars.
    pub fn with_exemplars(mut self, exemplars: Vec<Exemplar<T>>) -> Self {
        self.exemplars = exemplars;
        self
    }
}

#[cfg(feature = "experimental_metrics_producer")]
impl<T> ExponentialHistogram<T> {
    /// Creates an exponential histogram from its data points.
    pub fn new(
        data_points: Vec<ExponentialHistogramDataPoint<T>>,
        start_time: SystemTime,
        time: SystemTime,
        temporality: Temporality,
    ) -> Self {
        Self {
            data_points,
            start_time,
            time,
            temporality,
        }
    }
}

#[cfg(feature = "experimental_metrics_producer")]
impl<T> ExponentialHistogramDataPoint<T> {
    /// Creates an exponential histogram data point, with no min, max or exemplars.
    ///
    /// The count is `zero_count` plus the counts of both buckets.
    ///
    /// Returns an error if the count overflows `usize`.
    pub fn new(
        attributes: Vec<KeyValue>,
        scale: i8,
        zero_count: u64,
        zero_threshold: f64,
        positive_bucket: ExponentialBucket,
        negative_bucket: ExponentialBucket,
        sum: T,
    ) -> Result<Self, OTelSdkError> {
        let count = total_count(
            std::iter::once(zero_count)
                .chain(positive_bucket.counts.iter().copied())
                .chain(negative_bucket.counts.iter().copied()),
        )?;
        let count = usize::try_from(count).map_err(|_| {
            OTelSdkError::InternalFailure(format!("histogram count {count} overflows usize"))
        })?;
        Ok(Self {
            attributes,
            count,
            min: None,
            max: None,
            sum,
            scale,
            zero_count,
            positive_bucket,
            negative_bucket,
            zero_threshold,
            exemplars: Vec::new(),
        })
    }

    /// Sets the minimum value recorded.
    pub fn with_min(mut self, min: T) -> Self {
        self.min = Some(min);
        self
    }

    /// Sets the maximum value recorded.
    pub fn with_max(mut self, max: T) -> Self {
        self.max = Some(max);
        self
    }

    /// Sets the exemplars.
    pub fn with_exemplars(mut self, exemplars: Vec<Exemplar<T>>) -> Self {
        self.exemplars = exemplars;
        self
    }
}

#[cfg(feature = "experimental_metrics_producer")]
impl ExponentialBucket {
    /// Creates a bucket range where `counts[i]` is the count of bucket `offset + i`.
    pub fn new(offset: i32, counts: Vec<u64>) -> Self {
        Self { offset, counts }
    }
}

#[cfg(feature = "experimental_metrics_producer")]
impl<T> Exemplar<T> {
    /// Creates an exemplar. Use all-zero IDs when no sampled span was active.
    pub fn new(
        filtered_attributes: Vec<KeyValue>,
        time: SystemTime,
        value: T,
        span_id: [u8; 8],
        trace_id: [u8; 16],
    ) -> Self {
        Self {
            filtered_attributes,
            time,
            value,
            span_id,
            trace_id,
        }
    }
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

    #[cfg(feature = "experimental_metrics_producer")]
    #[test]
    fn constructors_set_every_field() {
        use super::{
            AggregatedMetrics, ExponentialBucket, ExponentialHistogram, Gauge, GaugeDataPoint,
            Histogram, Metric, MetricData, ScopeMetrics, Sum,
        };
        use crate::metrics::Temporality;
        use opentelemetry::InstrumentationScope;

        let (start, time) = (now(), now());
        let attrs = vec![KeyValue::new("key", "value")];

        let exemplar = Exemplar::new(attrs.clone(), time, 5u64, [1; 8], [2; 16]);
        assert!(exemplar.filtered_attributes().eq(&attrs));
        assert_eq!((exemplar.time(), exemplar.value), (time, 5));
        assert_eq!(
            (exemplar.span_id(), exemplar.trace_id()),
            (&[1; 8], &[2; 16])
        );

        let point = GaugeDataPoint::new(attrs.clone(), 5u64).with_exemplars(vec![exemplar.clone()]);
        assert!(point.attributes().eq(&attrs));
        assert!(point.exemplars().eq([&exemplar]));
        assert_eq!(point.value(), 5);
        let gauge = Gauge::new(vec![point], Some(start), time);
        assert_eq!(gauge.data_points().count(), 1);
        assert_eq!((gauge.start_time(), gauge.time()), (Some(start), time));

        let point = SumDataPoint::new(attrs.clone(), 5u64).with_exemplars(vec![exemplar.clone()]);
        assert!(point.attributes().eq(&attrs));
        assert!(point.exemplars().eq([&exemplar]));
        assert_eq!(point.value(), 5);
        let sum = Sum::new(vec![point], start, time, Temporality::Delta, true);
        assert_eq!(sum.data_points().count(), 1);
        assert_eq!((sum.start_time(), sum.time()), (start, time));
        assert_eq!(
            (sum.temporality(), sum.is_monotonic()),
            (Temporality::Delta, true)
        );

        let point = HistogramDataPoint::new(attrs.clone(), vec![1.0], vec![2, 3], 9u64)
            .unwrap()
            .with_min(1)
            .with_max(6)
            .with_exemplars(vec![exemplar.clone()]);
        assert!(point.attributes().eq(&attrs));
        assert!(point.exemplars().eq([&exemplar]));
        assert!(point.bounds().eq([1.0]));
        assert!(point.bucket_counts().eq([2, 3]));
        assert_eq!((point.count(), point.sum()), (5, 9));
        assert_eq!((point.min(), point.max()), (Some(1), Some(6)));
        let histogram = Histogram::new(vec![point], start, time, Temporality::Cumulative);
        assert_eq!(histogram.data_points().count(), 1);
        assert_eq!((histogram.start_time(), histogram.time()), (start, time));
        assert_eq!(histogram.temporality(), Temporality::Cumulative);

        let positive = ExponentialBucket::new(-1, vec![1, 2]);
        assert_eq!(positive.offset(), -1);
        assert!(positive.counts().eq([1, 2]));
        let negative = ExponentialBucket::new(0, vec![3]);
        let point = ExponentialHistogramDataPoint::new(
            attrs.clone(),
            2,
            4,
            0.5,
            positive.clone(),
            negative.clone(),
            9u64,
        )
        .unwrap()
        .with_min(1)
        .with_max(6)
        .with_exemplars(vec![exemplar.clone()]);
        assert!(point.attributes().eq(&attrs));
        assert!(point.exemplars().eq([&exemplar]));
        assert_eq!(
            (point.scale(), point.zero_count(), point.zero_threshold()),
            (2, 4, 0.5)
        );
        assert_eq!(
            (point.positive_bucket(), point.negative_bucket()),
            (&positive, &negative)
        );
        assert_eq!((point.count(), point.sum()), (10, 9));
        assert_eq!((point.min(), point.max()), (Some(1), Some(6)));
        let exponential = ExponentialHistogram::new(vec![point], start, time, Temporality::Delta);
        assert_eq!(exponential.data_points().count(), 1);
        assert_eq!(
            (exponential.start_time(), exponential.time()),
            (start, time)
        );
        assert_eq!(exponential.temporality(), Temporality::Delta);

        let metric = Metric::new("name", "description", "unit", MetricData::from(sum));
        assert_eq!(
            (metric.name(), metric.description(), metric.unit()),
            ("name", "description", "unit")
        );
        assert!(matches!(
            metric.data(),
            AggregatedMetrics::U64(MetricData::Sum(_))
        ));
        let scope = InstrumentationScope::builder("scope").build();
        let scope_metrics = ScopeMetrics::new(scope.clone(), vec![metric]);
        assert_eq!(scope_metrics.scope(), &scope);
        assert_eq!(scope_metrics.metrics().count(), 1);
    }

    #[cfg(feature = "experimental_metrics_producer")]
    #[test]
    fn histogram_constructors_reject_invalid_data() {
        use super::ExponentialBucket;

        let histogram = |bounds: Vec<f64>, bucket_counts: Vec<u64>| {
            HistogramDataPoint::new(vec![], bounds, bucket_counts, 0u64)
        };
        assert!(histogram(vec![1.0, 2.0], vec![1, 2, 3]).is_ok());
        assert!(histogram(vec![], vec![1]).is_ok());
        // Bounds that are not finite, or not strictly increasing.
        assert!(histogram(vec![1.0, f64::NAN], vec![1, 2, 3]).is_err());
        assert!(histogram(vec![1.0, f64::INFINITY], vec![1, 2, 3]).is_err());
        assert!(histogram(vec![2.0, 1.0], vec![1, 2, 3]).is_err());
        assert!(histogram(vec![1.0, 1.0], vec![1, 2, 3]).is_err());
        // One bucket count too few or too many.
        assert!(histogram(vec![1.0, 2.0], vec![1, 2]).is_err());
        assert!(histogram(vec![1.0, 2.0], vec![1, 2, 3, 4]).is_err());
        // A count that overflows.
        assert!(histogram(vec![1.0], vec![u64::MAX, 1]).is_err());

        let exponential = |zero_count: u64, counts: Vec<u64>| {
            ExponentialHistogramDataPoint::new(
                vec![],
                0,
                zero_count,
                0.0,
                ExponentialBucket::new(0, counts),
                ExponentialBucket::new(0, vec![]),
                0u64,
            )
        };
        assert_eq!(exponential(1, vec![2, 3]).unwrap().count(), 6);
        assert!(exponential(1, vec![u64::MAX]).is_err());
    }
}
