//! Types for delivery of pre-aggregated metric time series data.

use std::{borrow::Cow, time::SystemTime};

use opentelemetry::{InstrumentationScope, KeyValue};

use crate::Resource;

use super::Temporality;

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
    /// Create a new builder to create an [ResourceMetrics]
    pub fn builder() -> ResourceMetricsBuilder {
        ResourceMetricsBuilder {
            resource: Resource::empty(),
            scope_metrics: Vec::new(),
        }
    }

    /// Returns a reference to the [Resource] in [ResourceMetrics].
    pub fn resource(&self) -> &Resource {
        &self.resource
    }

    /// Returns an iterator over the [ScopeMetrics] in [ResourceMetrics].
    pub fn scope_metrics(&self) -> impl Iterator<Item = &ScopeMetrics> {
        self.scope_metrics.iter()
    }
}

/// Configuration option for [ResourceMetrics]
#[derive(Debug)]
pub struct ResourceMetricsBuilder {
    pub(crate) resource: Resource,
    pub(crate) scope_metrics: Vec<ScopeMetrics>,
}

impl ResourceMetricsBuilder {
    /// Sets the [Resource] for this [ResourceMetrics]
    pub fn with_resource(mut self, resource: Resource) -> Self {
        self.resource = resource;
        self
    }

    /// Sets a [Vec] of [ScopeMetrics] for this [ResourceMetrics]
    pub fn with_scope_metrics(mut self, scope_metrics: Vec<ScopeMetrics>) -> Self {
        self.scope_metrics = scope_metrics;
        self
    }
    /// Create a new [ResourceMetrics] from this configuration
    pub fn build(self) -> ResourceMetrics {
        ResourceMetrics {
            resource: self.resource,
            scope_metrics: self.scope_metrics,
        }
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
    /// Create a new builder to create a [ScopeMetrics]
    pub fn builder() -> ScopeMetricsBuilder {
        ScopeMetricsBuilder {
            scope: InstrumentationScope::default(),
            metrics: Vec::new(),
        }
    }

    /// Returns a reference to the [InstrumentationScope] in [ScopeMetrics].
    pub fn scope(&self) -> &InstrumentationScope {
        &self.scope
    }

    /// Returns an iterator over the [Metric]s in [ScopeMetrics].
    pub fn metrics(&self) -> impl Iterator<Item = &Metric> {
        self.metrics.iter()
    }
}

/// Configuration option for [ScopeMetrics]
#[derive(Debug)]
pub struct ScopeMetricsBuilder {
    scope: InstrumentationScope,
    metrics: Vec<Metric>,
}

impl ScopeMetricsBuilder {
    /// Sets the [InstrumentationScope] for this [ScopeMetrics]
    pub fn with_scope(mut self, scope: InstrumentationScope) -> Self {
        self.scope = scope;
        self
    }

    /// Sets a [Vec] of [Metric] for this [ScopeMetrics]
    pub fn with_metrics(mut self, metrics: Vec<Metric>) -> Self {
        self.metrics = metrics;
        self
    }

    /// Create a new [ScopeMetrics] from this configuration
    pub fn build(self) -> ScopeMetrics {
        ScopeMetrics {
            scope: self.scope,
            metrics: self.metrics,
        }
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
    /// Create a new builder to create a [Metric]
    pub fn builder(name: impl Into<Cow<'static, str>>, data: AggregatedMetrics) -> MetricBuilder {
        MetricBuilder {
            name: name.into(),
            description: Cow::Borrowed(""),
            unit: Cow::Borrowed(""),
            data,
        }
    }

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

/// Configuration option for [Metric]
#[derive(Debug)]
pub struct MetricBuilder {
    name: Cow<'static, str>,
    description: Cow<'static, str>,
    unit: Cow<'static, str>,
    data: AggregatedMetrics,
}

impl MetricBuilder {
    /// Sets the description for this [Metric]
    pub fn with_description(mut self, description: impl Into<Cow<'static, str>>) -> Self {
        self.description = description.into();
        self
    }

    /// Sets the unit for this [Metric]
    pub fn with_unit(mut self, unit: impl Into<Cow<'static, str>>) -> Self {
        self.unit = unit.into();
        self
    }

    /// Create a new [Metric] from this configuration
    pub fn build(self) -> Metric {
        Metric {
            name: self.name,
            description: self.description,
            unit: self.unit,
            data: self.data,
        }
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
    /// Create a new builder to create a [GaugeDataPoint]
    pub fn builder(value: T) -> GaugeDataPointBuilder<T> {
        GaugeDataPointBuilder {
            attributes: Vec::new(),
            value,
            exemplars: Vec::new(),
        }
    }

    /// Returns an iterator over the attributes in [GaugeDataPoint].
    pub fn attributes(&self) -> impl Iterator<Item = &KeyValue> {
        self.attributes.iter()
    }

    /// Returns an iterator over the [Exemplar]s in [GaugeDataPoint].
    pub fn exemplars(&self) -> impl Iterator<Item = &Exemplar<T>> {
        self.exemplars.iter()
    }
}

/// Configuration option for [GaugeDataPoint]
#[derive(Debug)]
pub struct GaugeDataPointBuilder<T> {
    attributes: Vec<KeyValue>,
    value: T,
    exemplars: Vec<Exemplar<T>>,
}

impl<T> GaugeDataPointBuilder<T> {
    /// Sets the attributes for this [GaugeDataPoint]
    pub fn with_attributes(mut self, attributes: Vec<KeyValue>) -> Self {
        self.attributes = attributes;
        self
    }

    /// Sets the exemplars for this [GaugeDataPoint]
    pub fn with_exemplars(mut self, exemplars: Vec<Exemplar<T>>) -> Self {
        self.exemplars = exemplars;
        self
    }

    /// Create a new [GaugeDataPoint] from this configuration
    pub fn build(self) -> GaugeDataPoint<T> {
        GaugeDataPoint {
            attributes: self.attributes,
            value: self.value,
            exemplars: self.exemplars,
        }
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
    /// Create a new builder to create a [Gauge]
    pub fn builder(data_points: Vec<GaugeDataPoint<T>>, time: SystemTime) -> GaugeBuilder<T> {
        GaugeBuilder {
            data_points,
            start_time: None,
            time,
        }
    }

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

/// Configuration option for [Gauge]
#[derive(Debug)]
pub struct GaugeBuilder<T> {
    data_points: Vec<GaugeDataPoint<T>>,
    start_time: Option<SystemTime>,
    time: SystemTime,
}

impl<T> GaugeBuilder<T> {
    /// Sets the start time for this [Gauge]
    pub fn with_start_time(mut self, start_time: SystemTime) -> Self {
        self.start_time = Some(start_time);
        self
    }

    /// Create a new [Gauge] from this configuration
    pub fn build(self) -> Gauge<T> {
        Gauge {
            data_points: self.data_points,
            start_time: self.start_time,
            time: self.time,
        }
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
    /// Create a new builder to create a [SumDataPoint]
    pub fn builder(value: T) -> SumDataPointBuilder<T> {
        SumDataPointBuilder {
            attributes: Vec::new(),
            value,
            exemplars: Vec::new(),
        }
    }

    /// Returns an iterator over the attributes in [SumDataPoint].
    pub fn attributes(&self) -> impl Iterator<Item = &KeyValue> {
        self.attributes.iter()
    }

    /// Returns an iterator over the [Exemplar]s in [SumDataPoint].
    pub fn exemplars(&self) -> impl Iterator<Item = &Exemplar<T>> {
        self.exemplars.iter()
    }
}

/// Configuration option for [SumDataPoint]
#[derive(Debug)]
pub struct SumDataPointBuilder<T> {
    attributes: Vec<KeyValue>,
    value: T,
    exemplars: Vec<Exemplar<T>>,
}

impl<T> SumDataPointBuilder<T> {
    /// Sets the attributes for this [SumDataPoint]
    pub fn with_attributes(mut self, attributes: Vec<KeyValue>) -> Self {
        self.attributes = attributes;
        self
    }

    /// Sets the exemplars for this [SumDataPoint]
    pub fn with_exemplars(mut self, exemplars: Vec<Exemplar<T>>) -> Self {
        self.exemplars = exemplars;
        self
    }

    /// Create a new [SumDataPoint] from this configuration
    pub fn build(self) -> SumDataPoint<T> {
        SumDataPoint {
            attributes: self.attributes,
            value: self.value,
            exemplars: self.exemplars,
        }
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
    /// Create a new builder to create a [Sum]
    pub fn builder(
        data_points: Vec<SumDataPoint<T>>,
        temporality: Temporality,
        is_monotonic: bool,
        start_time: SystemTime,
        time: SystemTime,
    ) -> SumBuilder<T> {
        SumBuilder {
            data_points,
            start_time,
            time,
            temporality,
            is_monotonic,
        }
    }

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

/// Configuration option for [Sum]
#[derive(Debug)]
pub struct SumBuilder<T> {
    data_points: Vec<SumDataPoint<T>>,
    start_time: SystemTime,
    time: SystemTime,
    temporality: Temporality,
    is_monotonic: bool,
}

impl<T> SumBuilder<T> {
    /// Create a new [Sum] from this configuration
    pub fn build(self) -> Sum<T> {
        Sum {
            data_points: self.data_points,
            start_time: self.start_time,
            time: self.time,
            temporality: self.temporality,
            is_monotonic: self.is_monotonic,
        }
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
    /// Create a new builder to create a [Histogram]
    pub fn builder(
        data_points: Vec<HistogramDataPoint<T>>,
        temporality: Temporality,
        start_time: SystemTime,
        time: SystemTime,
    ) -> HistogramBuilder<T> {
        HistogramBuilder {
            data_points,
            start_time,
            time,
            temporality,
        }
    }

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

/// Configuration option for [Histogram]
#[derive(Debug)]
pub struct HistogramBuilder<T> {
    data_points: Vec<HistogramDataPoint<T>>,
    start_time: SystemTime,
    time: SystemTime,
    temporality: Temporality,
}

impl<T> HistogramBuilder<T> {
    /// Create a new [Histogram] from this configuration
    pub fn build(self) -> Histogram<T> {
        Histogram {
            data_points: self.data_points,
            start_time: self.start_time,
            time: self.time,
            temporality: self.temporality,
        }
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
    /// Create a new builder to create a [HistogramDataPoint]
    pub fn builder(
        count: u64,
        sum: T,
        bounds: Vec<f64>,
        bucket_counts: Vec<u64>,
    ) -> HistogramDataPointBuilder<T> {
        HistogramDataPointBuilder {
            attributes: Vec::new(),
            count,
            bounds,
            bucket_counts,
            min: None,
            max: None,
            sum,
            exemplars: Vec::new(),
        }
    }

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

/// Configuration option for [HistogramDataPoint]
#[derive(Debug)]
pub struct HistogramDataPointBuilder<T> {
    attributes: Vec<KeyValue>,
    count: u64,
    bounds: Vec<f64>,
    bucket_counts: Vec<u64>,
    min: Option<T>,
    max: Option<T>,
    sum: T,
    exemplars: Vec<Exemplar<T>>,
}

impl<T> HistogramDataPointBuilder<T> {
    /// Sets the attributes for this [HistogramDataPoint]
    pub fn with_attributes(mut self, attributes: Vec<KeyValue>) -> Self {
        self.attributes = attributes;
        self
    }

    /// Sets the minimum value for this [HistogramDataPoint]
    pub fn with_min(mut self, min: T) -> Self {
        self.min = Some(min);
        self
    }

    /// Sets the maximum value for this [HistogramDataPoint]
    pub fn with_max(mut self, max: T) -> Self {
        self.max = Some(max);
        self
    }

    /// Sets the exemplars for this [HistogramDataPoint]
    pub fn with_exemplars(mut self, exemplars: Vec<Exemplar<T>>) -> Self {
        self.exemplars = exemplars;
        self
    }

    /// Create a new [HistogramDataPoint] from this configuration
    pub fn build(self) -> HistogramDataPoint<T> {
        HistogramDataPoint {
            attributes: self.attributes,
            count: self.count,
            bounds: self.bounds,
            bucket_counts: self.bucket_counts,
            min: self.min,
            max: self.max,
            sum: self.sum,
            exemplars: self.exemplars,
        }
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
    /// Create a new builder to create an [ExponentialHistogram]
    pub fn builder(
        data_points: Vec<ExponentialHistogramDataPoint<T>>,
        temporality: Temporality,
        start_time: SystemTime,
        time: SystemTime,
    ) -> ExponentialHistogramBuilder<T> {
        ExponentialHistogramBuilder {
            data_points,
            start_time,
            time,
            temporality,
        }
    }

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

/// Configuration option for [ExponentialHistogram]
#[derive(Debug)]
pub struct ExponentialHistogramBuilder<T> {
    data_points: Vec<ExponentialHistogramDataPoint<T>>,
    start_time: SystemTime,
    time: SystemTime,
    temporality: Temporality,
}

impl<T> ExponentialHistogramBuilder<T> {
    /// Create a new [ExponentialHistogram] from this configuration
    pub fn build(self) -> ExponentialHistogram<T> {
        ExponentialHistogram {
            data_points: self.data_points,
            start_time: self.start_time,
            time: self.time,
            temporality: self.temporality,
        }
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
    /// Create a new builder to create an [ExponentialHistogramDataPoint]
    pub fn builder(
        count: usize,
        sum: T,
        scale: i8,
        zero_count: u64,
        positive_bucket: ExponentialBucket,
        negative_bucket: ExponentialBucket,
    ) -> ExponentialHistogramDataPointBuilder<T> {
        ExponentialHistogramDataPointBuilder {
            attributes: Vec::new(),
            count,
            min: None,
            max: None,
            sum,
            scale,
            zero_count,
            positive_bucket,
            negative_bucket,
            zero_threshold: 0.0,
            exemplars: Vec::new(),
        }
    }

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

/// Configuration option for [ExponentialHistogramDataPoint]
#[derive(Debug)]
pub struct ExponentialHistogramDataPointBuilder<T> {
    attributes: Vec<KeyValue>,
    count: usize,
    min: Option<T>,
    max: Option<T>,
    sum: T,
    scale: i8,
    zero_count: u64,
    positive_bucket: ExponentialBucket,
    negative_bucket: ExponentialBucket,
    zero_threshold: f64,
    exemplars: Vec<Exemplar<T>>,
}

impl<T> ExponentialHistogramDataPointBuilder<T> {
    /// Sets the attributes for this [ExponentialHistogramDataPoint]
    pub fn with_attributes(mut self, attributes: Vec<KeyValue>) -> Self {
        self.attributes = attributes;
        self
    }

    /// Sets the minimum value for this [ExponentialHistogramDataPoint]
    pub fn with_min(mut self, min: T) -> Self {
        self.min = Some(min);
        self
    }

    /// Sets the maximum value for this [ExponentialHistogramDataPoint]
    pub fn with_max(mut self, max: T) -> Self {
        self.max = Some(max);
        self
    }

    /// Sets the zero threshold for this [ExponentialHistogramDataPoint]
    pub fn with_zero_threshold(mut self, zero_threshold: f64) -> Self {
        self.zero_threshold = zero_threshold;
        self
    }

    /// Sets the exemplars for this [ExponentialHistogramDataPoint]
    pub fn with_exemplars(mut self, exemplars: Vec<Exemplar<T>>) -> Self {
        self.exemplars = exemplars;
        self
    }

    /// Create a new [ExponentialHistogramDataPoint] from this configuration
    pub fn build(self) -> ExponentialHistogramDataPoint<T> {
        ExponentialHistogramDataPoint {
            attributes: self.attributes,
            count: self.count,
            min: self.min,
            max: self.max,
            sum: self.sum,
            scale: self.scale,
            zero_count: self.zero_count,
            positive_bucket: self.positive_bucket,
            negative_bucket: self.negative_bucket,
            zero_threshold: self.zero_threshold,
            exemplars: self.exemplars,
        }
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
    /// Create a new [ExponentialBucket]
    pub fn new(offset: i32, counts: Vec<u64>) -> Self {
        Self { offset, counts }
    }

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
    /// Create a new builder to create an [Exemplar]
    pub fn builder(value: T, time: SystemTime) -> ExemplarBuilder<T> {
        ExemplarBuilder {
            filtered_attributes: Vec::new(),
            time,
            value,
            span_id: [0; 8],
            trace_id: [0; 16],
        }
    }

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

/// Configuration option for [Exemplar]
#[derive(Debug)]
pub struct ExemplarBuilder<T> {
    filtered_attributes: Vec<KeyValue>,
    time: SystemTime,
    value: T,
    span_id: [u8; 8],
    trace_id: [u8; 16],
}

impl<T> ExemplarBuilder<T> {
    /// Sets the filtered attributes for this [Exemplar]
    pub fn with_filtered_attributes(mut self, filtered_attributes: Vec<KeyValue>) -> Self {
        self.filtered_attributes = filtered_attributes;
        self
    }

    /// Sets the span ID for this [Exemplar]
    pub fn with_span_id(mut self, span_id: [u8; 8]) -> Self {
        self.span_id = span_id;
        self
    }

    /// Sets the trace ID for this [Exemplar]
    pub fn with_trace_id(mut self, trace_id: [u8; 16]) -> Self {
        self.trace_id = trace_id;
        self
    }

    /// Create a new [Exemplar] from this configuration
    pub fn build(self) -> Exemplar<T> {
        Exemplar {
            filtered_attributes: self.filtered_attributes,
            time: self.time,
            value: self.value,
            span_id: self.span_id,
            trace_id: self.trace_id,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use opentelemetry::time::now;
    use opentelemetry::KeyValue;

    #[test]
    fn build_resource_metrics_full_tree() {
        let time = now();
        let exemplar = Exemplar::builder(1.0_f64, time)
            .with_filtered_attributes(vec![KeyValue::new("filtered", "attr")])
            .with_span_id([1; 8])
            .with_trace_id([2; 16])
            .build();

        let gauge_dp = GaugeDataPoint::builder(42.0_f64)
            .with_attributes(vec![KeyValue::new("host", "localhost")])
            .with_exemplars(vec![exemplar])
            .build();

        let gauge = Gauge::builder(vec![gauge_dp], time)
            .with_start_time(time)
            .build();

        let metric = Metric::builder("my_gauge", AggregatedMetrics::F64(MetricData::Gauge(gauge)))
            .with_description("a test gauge")
            .with_unit("ms")
            .build();

        let scope_metrics = ScopeMetrics::builder().with_metrics(vec![metric]).build();

        let rm = ResourceMetrics::builder()
            .with_resource(Resource::builder().build())
            .with_scope_metrics(vec![scope_metrics])
            .build();

        assert_eq!(rm.scope_metrics().count(), 1);
        let sm = rm.scope_metrics().next().unwrap();
        assert_eq!(sm.metrics().count(), 1);
        let m = sm.metrics().next().unwrap();
        assert_eq!(m.name(), "my_gauge");
        assert_eq!(m.description(), "a test gauge");
        assert_eq!(m.unit(), "ms");
    }

    #[test]
    fn build_sum_with_data_points() {
        let time = now();
        let dp = SumDataPoint::builder(100_i64)
            .with_attributes(vec![KeyValue::new("key", "value")])
            .build();

        let sum = Sum::builder(vec![dp], Temporality::Cumulative, true, time, time).build();

        assert_eq!(sum.data_points().count(), 1);
        assert!(sum.is_monotonic());
        assert_eq!(sum.temporality(), Temporality::Cumulative);
        let dp = sum.data_points().next().unwrap();
        assert_eq!(dp.value(), 100);
        assert_eq!(dp.attributes().count(), 1);
    }

    #[test]
    fn build_histogram_with_optional_fields() {
        let time = now();
        let dp = HistogramDataPoint::builder(10, 42.0_f64, vec![5.0, 10.0], vec![3, 5, 2])
            .with_attributes(vec![KeyValue::new("key", "val")])
            .with_min(1.0)
            .with_max(20.0)
            .build();

        let histogram = Histogram::builder(vec![dp], Temporality::Delta, time, time).build();

        assert_eq!(histogram.data_points().count(), 1);
        assert_eq!(histogram.temporality(), Temporality::Delta);
        let dp = histogram.data_points().next().unwrap();
        assert_eq!(dp.count(), 10);
        assert_eq!(dp.sum(), 42.0);
        assert_eq!(dp.min(), Some(1.0));
        assert_eq!(dp.max(), Some(20.0));
        assert_eq!(dp.bounds().collect::<Vec<_>>(), vec![5.0, 10.0]);
        assert_eq!(dp.bucket_counts().collect::<Vec<_>>(), vec![3, 5, 2]);
    }

    #[test]
    fn build_exponential_histogram() {
        let time = now();
        let dp = ExponentialHistogramDataPoint::builder(
            5,
            100.0_f64,
            3,
            1,
            ExponentialBucket::new(0, vec![1, 2, 3]),
            ExponentialBucket::new(0, vec![4, 5]),
        )
        .with_attributes(vec![KeyValue::new("key", "val")])
        .with_min(10.0)
        .with_max(50.0)
        .with_zero_threshold(0.001)
        .build();

        let eh =
            ExponentialHistogram::builder(vec![dp], Temporality::Cumulative, time, time).build();

        assert_eq!(eh.data_points().count(), 1);
        let dp = eh.data_points().next().unwrap();
        assert_eq!(dp.count(), 5);
        assert_eq!(dp.scale(), 3);
        assert_eq!(dp.zero_count(), 1);
        assert_eq!(dp.sum(), 100.0);
        assert_eq!(dp.min(), Some(10.0));
        assert_eq!(dp.max(), Some(50.0));
        assert!((dp.zero_threshold() - 0.001).abs() < f64::EPSILON);
        assert_eq!(dp.positive_bucket().offset(), 0);
        assert_eq!(
            dp.positive_bucket().counts().collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        assert_eq!(
            dp.negative_bucket().counts().collect::<Vec<_>>(),
            vec![4, 5]
        );
    }

    #[test]
    fn build_exemplar_with_defaults() {
        let time = now();
        let exemplar = Exemplar::builder(42.0_f64, time).build();

        assert_eq!(exemplar.value, 42.0);
        assert_eq!(exemplar.time(), time);
        assert_eq!(exemplar.span_id(), &[0; 8]);
        assert_eq!(exemplar.trace_id(), &[0; 16]);
        assert_eq!(exemplar.filtered_attributes().count(), 0);
    }

    #[test]
    fn build_defaults_without_optional_fields() {
        let rm = ResourceMetrics::builder().build();
        assert_eq!(rm.scope_metrics().count(), 0);

        let sm = ScopeMetrics::builder().build();
        assert_eq!(sm.metrics().count(), 0);

        let dp = GaugeDataPoint::builder(0.0_f64).build();
        assert_eq!(dp.value(), 0.0);
        assert_eq!(dp.attributes().count(), 0);
        assert_eq!(dp.exemplars().count(), 0);
    }

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
