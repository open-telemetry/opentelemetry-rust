use std::{marker, sync::Arc};

use once_cell::sync::Lazy;
use opentelemetry::KeyValue;

use crate::{
    metrics::data::{Aggregation, Gauge, Temporality},
    AttributeSet,
};

use super::{
    exponential_histogram,
    exponential_histogram::ExpoHistogram,
    histogram,
    histogram::Histogram,
    last_value,
    last_value::LastValue,
    sum,
    sum::{PrecomputedSum, Sum},
    Number,
};

const STREAM_CARDINALITY_LIMIT: u32 = 2000;
pub(crate) static STREAM_OVERFLOW_ATTRIBUTE_SET: Lazy<AttributeSet> = Lazy::new(|| {
    let key_values: [KeyValue; 1] = [KeyValue::new("otel.metric.overflow", "true")];
    AttributeSet::from(&key_values[..])
});

/// Checks whether aggregator has hit cardinality limit for metric streams
pub(crate) fn is_under_cardinality_limit(size: usize) -> bool {
    size < STREAM_CARDINALITY_LIMIT as usize - 1
}

/// Receives measurements to be aggregated.
pub(crate) trait Measure<T>: Send + Sync + 'static {
    fn call(&self, measurement: T, attrs: AttributeSet);
}

impl<F, T> Measure<T> for F
where
    F: Fn(T, AttributeSet) + Send + Sync + 'static,
{
    fn call(&self, measurement: T, attrs: AttributeSet) {
        self(measurement, attrs)
    }
}

/// Allows creating a bounded measure based on the specified attribute set
pub(crate) trait BoundedMeasureGenerator<T>: Send + Sync + 'static {
    fn generate(&self, attrs: AttributeSet) -> Arc<dyn BoundedMeasure<T>>;
}

impl<F, T> BoundedMeasureGenerator<T> for F
where
    F: Fn(AttributeSet) -> Arc<dyn BoundedMeasure<T>> + Send + Sync + 'static,
{
    fn generate(&self, attrs: AttributeSet) -> Arc<dyn BoundedMeasure<T>> {
        self(attrs)
    }
}

/// Receives measurements to be aggregated for a predetermined attribute set
pub(crate) trait BoundedMeasure<T>: Send + Sync + 'static {
    fn call(&self, measurement: T);
}

impl<F, T> BoundedMeasure<T> for F
where
    F: Fn(T) + Send + Sync + 'static,
{
    fn call(&self, measurement: T) {
        self(measurement)
    }
}

/// A measure and it's related bounded measure generator
#[derive(Clone)]
pub(crate) struct MeasureSet<T> {
    pub(crate) measure: Arc<dyn Measure<T>>,
    pub(crate) _bound_measure_generator: Arc<dyn BoundedMeasureGenerator<T>>,
}

impl<T> MeasureSet<T> {
    pub(crate) fn new(
        measure: Arc<dyn Measure<T>>,
        bound_measure_generator: Arc<dyn BoundedMeasureGenerator<T>>,
    ) -> Self {
        MeasureSet {
            measure,
            _bound_measure_generator: bound_measure_generator,
        }
    }
}

/// Stores the aggregate of measurements into the aggregation and returns the number
/// of aggregate data-points output.
pub(crate) trait ComputeAggregation: Send + Sync + 'static {
    /// Compute the new aggregation and store in `dest`.
    ///
    /// If no initial aggregation exists, `dest` will be `None`, in which case the
    /// returned option is expected to contain a new aggregation with the data from
    /// the current collection cycle.
    fn call(&self, dest: Option<&mut dyn Aggregation>) -> (usize, Option<Box<dyn Aggregation>>);
}

impl<T> ComputeAggregation for T
where
    T: Fn(Option<&mut dyn Aggregation>) -> (usize, Option<Box<dyn Aggregation>>)
        + Send
        + Sync
        + 'static,
{
    fn call(&self, dest: Option<&mut dyn Aggregation>) -> (usize, Option<Box<dyn Aggregation>>) {
        self(dest)
    }
}

/// Builds aggregate functions
pub(crate) struct AggregateBuilder<T> {
    /// The temporality used for the returned aggregate functions.
    ///
    /// If this is not provided, a default of cumulative will be used (except for the
    /// last-value aggregate function where delta is the only appropriate
    /// temporality).
    temporality: Option<Temporality>,

    /// The attribute filter the aggregate function will use on the input of
    /// measurements.
    filter: Option<Filter>,

    _marker: marker::PhantomData<T>,
}

type Filter = Arc<dyn Fn(&KeyValue) -> bool + Send + Sync>;

impl<T: Number<T>> AggregateBuilder<T> {
    pub(crate) fn new(temporality: Option<Temporality>, filter: Option<Filter>) -> Self {
        AggregateBuilder {
            temporality,
            filter,
            _marker: marker::PhantomData,
        }
    }

    /// Wraps the passed in measure with an attribute filtering function.
    fn filter(&self, f: impl Measure<T>) -> impl Measure<T> {
        let filter = self.filter.as_ref().map(Arc::clone);
        move |n, mut attrs: AttributeSet| {
            if let Some(filter) = &filter {
                attrs.retain(filter.as_ref());
            }
            f.call(n, attrs)
        }
    }

    /// Builds a last-value aggregate function input and output.
    ///
    /// [Builder::temporality] is ignored and delta is always used.
    pub(crate) fn last_value(
        &self,
    ) -> (
        impl Measure<T>,
        impl ComputeAggregation,
        impl BoundedMeasureGenerator<T>,
    ) {
        // Delta temporality is the only temporality that makes semantic sense for
        // a last-value aggregate.
        let lv_filter = Arc::new(LastValue::new());
        let lv_agg = Arc::clone(&lv_filter);
        let lv_bound = Arc::clone(&lv_filter);

        (
            self.filter(move |n, a| lv_filter.measure(n, a)),
            move |dest: Option<&mut dyn Aggregation>| {
                let g = dest.and_then(|d| d.as_mut().downcast_mut::<Gauge<T>>());
                let mut new_agg = if g.is_none() {
                    Some(Gauge {
                        data_points: vec![],
                    })
                } else {
                    None
                };
                let g = g.unwrap_or_else(|| new_agg.as_mut().expect("present if g is none"));

                lv_agg.compute_aggregation(&mut g.data_points);

                (g.data_points.len(), new_agg.map(|a| Box::new(a) as Box<_>))
            },
            move |attrs: AttributeSet| last_value::generate_bound_measure(&lv_bound, attrs),
        )
    }

    /// Builds a precomputed sum aggregate function input and output.
    pub(crate) fn precomputed_sum(
        &self,
        monotonic: bool,
    ) -> (
        impl Measure<T>,
        impl ComputeAggregation,
        impl BoundedMeasureGenerator<T>,
    ) {
        let s = Arc::new(PrecomputedSum::new(monotonic));
        let agg_sum = Arc::clone(&s);
        let bound_sum = Arc::clone(&s);
        let t = self.temporality;

        (
            self.filter(move |n, a| s.measure(n, a)),
            move |dest: Option<&mut dyn Aggregation>| match t {
                Some(Temporality::Delta) => agg_sum.delta(dest),
                _ => agg_sum.cumulative(dest),
            },
            move |attrs: AttributeSet| {
                sum::generate_bound_measure_precomputed_sum(&bound_sum, attrs)
            },
        )
    }

    /// Builds a sum aggregate function input and output.
    pub(crate) fn sum(
        &self,
        monotonic: bool,
    ) -> (
        impl Measure<T>,
        impl ComputeAggregation,
        impl BoundedMeasureGenerator<T>,
    ) {
        let s = Arc::new(Sum::new(monotonic));
        let agg_sum = Arc::clone(&s);
        let bound_sum = Arc::clone(&s);
        let t = self.temporality;

        (
            self.filter(move |n, a| s.measure(n, a)),
            move |dest: Option<&mut dyn Aggregation>| match t {
                Some(Temporality::Delta) => agg_sum.delta(dest),
                _ => agg_sum.cumulative(dest),
            },
            move |a| sum::generate_bound_measure_sum(&bound_sum, a),
        )
    }

    /// Builds a histogram aggregate function input and output.
    pub(crate) fn explicit_bucket_histogram(
        &self,
        boundaries: Vec<f64>,
        record_min_max: bool,
        record_sum: bool,
    ) -> (
        impl Measure<T>,
        impl ComputeAggregation,
        impl BoundedMeasureGenerator<T>,
    ) {
        let h = Arc::new(Histogram::new(boundaries, record_min_max, record_sum));
        let agg_h = Arc::clone(&h);
        let bound_h = Arc::clone(&h);
        let t = self.temporality;

        (
            self.filter(move |n, a| h.measure(n, a)),
            move |dest: Option<&mut dyn Aggregation>| match t {
                Some(Temporality::Delta) => agg_h.delta(dest),
                _ => agg_h.cumulative(dest),
            },
            move |attrs: AttributeSet| histogram::generate_bound_measure(&bound_h, attrs),
        )
    }

    /// Builds an exponential histogram aggregate function input and output.
    pub(crate) fn exponential_bucket_histogram(
        &self,
        max_size: u32,
        max_scale: i8,
        record_min_max: bool,
        record_sum: bool,
    ) -> (
        impl Measure<T>,
        impl ComputeAggregation,
        impl BoundedMeasureGenerator<T>,
    ) {
        let h = Arc::new(ExpoHistogram::new(
            max_size,
            max_scale,
            record_min_max,
            record_sum,
        ));
        let agg_h = Arc::clone(&h);
        let bound_h = Arc::clone(&h);
        let t = self.temporality;

        (
            self.filter(move |n, a| h.measure(n, a)),
            move |dest: Option<&mut dyn Aggregation>| match t {
                Some(Temporality::Delta) => agg_h.delta(dest),
                _ => agg_h.cumulative(dest),
            },
            move |attrs: AttributeSet| {
                exponential_histogram::generate_bound_measure(&bound_h, attrs)
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::metrics::data::{
        DataPoint, ExponentialBucket, ExponentialHistogram, ExponentialHistogramDataPoint,
        Histogram, HistogramDataPoint, Sum,
    };
    use std::time::SystemTime;

    use super::*;

    #[test]
    fn last_value_aggregation() {
        let (measure, agg, _) = AggregateBuilder::<u64>::new(None, None).last_value();
        let mut a = Gauge {
            data_points: vec![DataPoint {
                attributes: AttributeSet::from(&[KeyValue::new("a", 1)][..]),
                start_time: Some(SystemTime::now()),
                time: Some(SystemTime::now()),
                value: 1u64,
                exemplars: vec![],
            }],
        };
        let new_attributes = [KeyValue::new("b", 2)];
        measure.call(2, AttributeSet::from(&new_attributes[..]));

        let (count, new_agg) = agg.call(Some(&mut a));

        assert_eq!(count, 1);
        assert!(new_agg.is_none());
        assert_eq!(a.data_points.len(), 1);
        assert_eq!(
            a.data_points[0].attributes,
            AttributeSet::from(&new_attributes[..])
        );
        assert_eq!(a.data_points[0].value, 2);
    }

    #[test]
    fn precomputed_sum_aggregation() {
        for temporality in [Temporality::Delta, Temporality::Cumulative] {
            let (measure, agg, _) =
                AggregateBuilder::<u64>::new(Some(temporality), None).precomputed_sum(true);
            let mut a = Sum {
                data_points: vec![
                    DataPoint {
                        attributes: AttributeSet::from(&[KeyValue::new("a1", 1)][..]),
                        start_time: Some(SystemTime::now()),
                        time: Some(SystemTime::now()),
                        value: 1u64,
                        exemplars: vec![],
                    },
                    DataPoint {
                        attributes: AttributeSet::from(&[KeyValue::new("a2", 2)][..]),
                        start_time: Some(SystemTime::now()),
                        time: Some(SystemTime::now()),
                        value: 2u64,
                        exemplars: vec![],
                    },
                ],
                temporality: if temporality == Temporality::Delta {
                    Temporality::Cumulative
                } else {
                    Temporality::Delta
                },
                is_monotonic: false,
            };
            let new_attributes = [KeyValue::new("b", 2)];
            measure.call(3, AttributeSet::from(&new_attributes[..]));

            let (count, new_agg) = agg.call(Some(&mut a));

            assert_eq!(count, 1);
            assert!(new_agg.is_none());
            assert_eq!(a.temporality, temporality);
            assert!(a.is_monotonic);
            assert_eq!(a.data_points.len(), 1);
            assert_eq!(
                a.data_points[0].attributes,
                AttributeSet::from(&new_attributes[..])
            );
            assert_eq!(a.data_points[0].value, 3);
        }
    }

    #[test]
    fn sum_aggregation() {
        for temporality in [Temporality::Delta, Temporality::Cumulative] {
            let (measure, agg, _) = AggregateBuilder::<u64>::new(Some(temporality), None).sum(true);
            let mut a = Sum {
                data_points: vec![
                    DataPoint {
                        attributes: AttributeSet::from(&[KeyValue::new("a1", 1)][..]),
                        start_time: Some(SystemTime::now()),
                        time: Some(SystemTime::now()),
                        value: 1u64,
                        exemplars: vec![],
                    },
                    DataPoint {
                        attributes: AttributeSet::from(&[KeyValue::new("a2", 2)][..]),
                        start_time: Some(SystemTime::now()),
                        time: Some(SystemTime::now()),
                        value: 2u64,
                        exemplars: vec![],
                    },
                ],
                temporality: if temporality == Temporality::Delta {
                    Temporality::Cumulative
                } else {
                    Temporality::Delta
                },
                is_monotonic: false,
            };
            let new_attributes = [KeyValue::new("b", 2)];
            measure.call(3, AttributeSet::from(&new_attributes[..]));

            let (count, new_agg) = agg.call(Some(&mut a));

            assert_eq!(count, 1);
            assert!(new_agg.is_none());
            assert_eq!(a.temporality, temporality);
            assert!(a.is_monotonic);
            assert_eq!(a.data_points.len(), 1);
            assert_eq!(
                a.data_points[0].attributes,
                AttributeSet::from(&new_attributes[..])
            );
            assert_eq!(a.data_points[0].value, 3);
        }
    }

    #[test]
    fn explicit_bucket_histogram_aggregation() {
        for temporality in [Temporality::Delta, Temporality::Cumulative] {
            let (measure, agg, _) = AggregateBuilder::<u64>::new(Some(temporality), None)
                .explicit_bucket_histogram(vec![1.0], true, true);
            let mut a = Histogram {
                data_points: vec![HistogramDataPoint {
                    attributes: AttributeSet::from(&[KeyValue::new("a2", 2)][..]),
                    start_time: SystemTime::now(),
                    time: SystemTime::now(),
                    count: 2,
                    bounds: vec![1.0, 2.0],
                    bucket_counts: vec![0, 1, 1],
                    min: None,
                    max: None,
                    sum: 3u64,
                    exemplars: vec![],
                }],
                temporality: if temporality == Temporality::Delta {
                    Temporality::Cumulative
                } else {
                    Temporality::Delta
                },
            };
            let new_attributes = [KeyValue::new("b", 2)];
            measure.call(3, AttributeSet::from(&new_attributes[..]));

            let (count, new_agg) = agg.call(Some(&mut a));

            assert_eq!(count, 1);
            assert!(new_agg.is_none());
            assert_eq!(a.temporality, temporality);
            assert_eq!(a.data_points.len(), 1);
            assert_eq!(
                a.data_points[0].attributes,
                AttributeSet::from(&new_attributes[..])
            );
            assert_eq!(a.data_points[0].count, 1);
            assert_eq!(a.data_points[0].bounds, vec![1.0]);
            assert_eq!(a.data_points[0].bucket_counts, vec![0, 1]);
            assert_eq!(a.data_points[0].min, Some(3));
            assert_eq!(a.data_points[0].max, Some(3));
            assert_eq!(a.data_points[0].sum, 3);
        }
    }

    #[test]
    fn exponential_histogram_aggregation() {
        for temporality in [Temporality::Delta, Temporality::Cumulative] {
            let (measure, agg, _) = AggregateBuilder::<u64>::new(Some(temporality), None)
                .exponential_bucket_histogram(4, 20, true, true);
            let mut a = ExponentialHistogram {
                data_points: vec![ExponentialHistogramDataPoint {
                    attributes: AttributeSet::from(&[KeyValue::new("a2", 2)][..]),
                    start_time: SystemTime::now(),
                    time: SystemTime::now(),
                    count: 2,
                    min: None,
                    max: None,
                    sum: 3u64,
                    scale: 10,
                    zero_count: 1,
                    positive_bucket: ExponentialBucket {
                        offset: 1,
                        counts: vec![1],
                    },
                    negative_bucket: ExponentialBucket {
                        offset: 1,
                        counts: vec![1],
                    },
                    zero_threshold: 1.0,
                    exemplars: vec![],
                }],
                temporality: if temporality == Temporality::Delta {
                    Temporality::Cumulative
                } else {
                    Temporality::Delta
                },
            };
            let new_attributes = [KeyValue::new("b", 2)];
            measure.call(3, AttributeSet::from(&new_attributes[..]));

            let (count, new_agg) = agg.call(Some(&mut a));

            assert_eq!(count, 1);
            assert!(new_agg.is_none());
            assert_eq!(a.temporality, temporality);
            assert_eq!(a.data_points.len(), 1);
            assert_eq!(
                a.data_points[0].attributes,
                AttributeSet::from(&new_attributes[..])
            );
            assert_eq!(a.data_points[0].count, 1);
            assert_eq!(a.data_points[0].min, Some(3));
            assert_eq!(a.data_points[0].max, Some(3));
            assert_eq!(a.data_points[0].sum, 3);
            assert_eq!(a.data_points[0].zero_count, 0);
            assert_eq!(a.data_points[0].zero_threshold, 0.0);
            assert_eq!(a.data_points[0].positive_bucket.offset, 1661953);
            assert_eq!(a.data_points[0].positive_bucket.counts, vec![1]);
            assert_eq!(a.data_points[0].negative_bucket.offset, 0);
            assert!(a.data_points[0].negative_bucket.counts.is_empty());
        }
    }
}
