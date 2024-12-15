use std::{
    marker,
    mem::replace,
    ops::DerefMut,
    sync::{Arc, Mutex},
    time::SystemTime,
};

use opentelemetry::KeyValue;

use crate::metrics::{data::Aggregation, Temporality};

use super::{
    exponential_histogram::ExpoHistogram, histogram::Histogram, last_value::LastValue,
    precomputed_sum::PrecomputedSum, sum::Sum, Number,
};

pub(crate) const STREAM_CARDINALITY_LIMIT: usize = 2000;

/// Checks whether aggregator has hit cardinality limit for metric streams
pub(crate) fn is_under_cardinality_limit(size: usize) -> bool {
    size < STREAM_CARDINALITY_LIMIT
}

/// Receives measurements to be aggregated.
pub(crate) trait Measure<T>: Send + Sync + 'static {
    fn call(&self, measurement: T, attrs: &[KeyValue]);
}

impl<F, T> Measure<T> for F
where
    F: Fn(T, &[KeyValue]) + Send + Sync + 'static,
{
    fn call(&self, measurement: T, attrs: &[KeyValue]) {
        self(measurement, attrs)
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

pub(crate) struct AggregateTime {
    pub start: SystemTime,
    pub current: SystemTime,
}

/// Initialized [`AggregateTime`] for specific [`Temporality`]
pub(crate) struct AggregateTimeInitiator(Mutex<SystemTime>);

impl AggregateTimeInitiator {
    pub(crate) fn delta(&self) -> AggregateTime {
        let current_time = SystemTime::now();
        let start_time = self
            .0
            .lock()
            .map(|mut start| replace(start.deref_mut(), current_time))
            .unwrap_or(current_time);
        AggregateTime {
            start: start_time,
            current: current_time,
        }
    }

    pub(crate) fn cumulative(&self) -> AggregateTime {
        let current_time = SystemTime::now();
        let start_time = self.0.lock().map(|start| *start).unwrap_or(current_time);
        AggregateTime {
            start: start_time,
            current: current_time,
        }
    }
}

impl Default for AggregateTimeInitiator {
    fn default() -> Self {
        Self(Mutex::new(SystemTime::now()))
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

impl<T: Number> AggregateBuilder<T> {
    pub(crate) fn new(temporality: Option<Temporality>, filter: Option<Filter>) -> Self {
        AggregateBuilder {
            temporality,
            filter,
            _marker: marker::PhantomData,
        }
    }

    /// Wraps the passed in measure with an attribute filtering function.
    fn filter(&self, f: impl Measure<T>) -> impl Measure<T> {
        let filter = self.filter.clone();
        move |n, attrs: &[KeyValue]| {
            if let Some(filter) = &filter {
                let filtered_attrs: Vec<KeyValue> =
                    attrs.iter().filter(|kv| filter(kv)).cloned().collect();
                f.call(n, &filtered_attrs);
            } else {
                f.call(n, attrs);
            };
        }
    }

    /// Builds a last-value aggregate function input and output.
    pub(crate) fn last_value(&self) -> (impl Measure<T>, impl ComputeAggregation) {
        let lv = Arc::new(LastValue::new());
        let agg_lv = Arc::clone(&lv);
        let t = self.temporality;

        (
            self.filter(move |n, a: &[KeyValue]| lv.measure(n, a)),
            move |dest: Option<&mut dyn Aggregation>| match t {
                Some(Temporality::Delta) => agg_lv.delta(dest),
                _ => agg_lv.cumulative(dest),
            },
        )
    }

    /// Builds a precomputed sum aggregate function input and output.
    pub(crate) fn precomputed_sum(
        &self,
        monotonic: bool,
    ) -> (impl Measure<T>, impl ComputeAggregation) {
        let s = Arc::new(PrecomputedSum::new(monotonic));
        let agg_sum = Arc::clone(&s);
        let t = self.temporality;

        (
            self.filter(move |n, a: &[KeyValue]| s.measure(n, a)),
            move |dest: Option<&mut dyn Aggregation>| match t {
                Some(Temporality::Delta) => agg_sum.delta(dest),
                _ => agg_sum.cumulative(dest),
            },
        )
    }

    /// Builds a sum aggregate function input and output.
    pub(crate) fn sum(&self, monotonic: bool) -> (impl Measure<T>, impl ComputeAggregation) {
        let s = Arc::new(Sum::new(monotonic));
        let agg_sum = Arc::clone(&s);
        let t = self.temporality;

        (
            self.filter(move |n, a: &[KeyValue]| s.measure(n, a)),
            move |dest: Option<&mut dyn Aggregation>| match t {
                Some(Temporality::Delta) => agg_sum.delta(dest),
                _ => agg_sum.cumulative(dest),
            },
        )
    }

    /// Builds a histogram aggregate function input and output.
    pub(crate) fn explicit_bucket_histogram(
        &self,
        boundaries: Vec<f64>,
        record_min_max: bool,
        record_sum: bool,
    ) -> (impl Measure<T>, impl ComputeAggregation) {
        let h = Arc::new(Histogram::new(boundaries, record_min_max, record_sum));
        let agg_h = Arc::clone(&h);
        let t = self.temporality;

        (
            self.filter(move |n, a: &[KeyValue]| h.measure(n, a)),
            move |dest: Option<&mut dyn Aggregation>| match t {
                Some(Temporality::Delta) => agg_h.delta(dest),
                _ => agg_h.cumulative(dest),
            },
        )
    }

    /// Builds an exponential histogram aggregate function input and output.
    pub(crate) fn exponential_bucket_histogram(
        &self,
        max_size: u32,
        max_scale: i8,
        record_min_max: bool,
        record_sum: bool,
    ) -> (impl Measure<T>, impl ComputeAggregation) {
        let h = Arc::new(ExpoHistogram::new(
            max_size,
            max_scale,
            record_min_max,
            record_sum,
        ));
        let agg_h = Arc::clone(&h);
        let t = self.temporality;

        (
            self.filter(move |n, a: &[KeyValue]| h.measure(n, a)),
            move |dest: Option<&mut dyn Aggregation>| match t {
                Some(Temporality::Delta) => agg_h.delta(dest),
                _ => agg_h.cumulative(dest),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::metrics::data::{
        ExponentialBucket, ExponentialHistogram, ExponentialHistogramDataPoint, Gauge,
        GaugeDataPoint, Histogram, HistogramDataPoint, Sum, SumDataPoint,
    };
    use std::{time::SystemTime, vec};

    use super::*;

    #[test]
    fn last_value_aggregation() {
        let (measure, agg) = AggregateBuilder::<u64>::new(None, None).last_value();
        let mut a = Gauge {
            data_points: vec![GaugeDataPoint {
                attributes: vec![KeyValue::new("a", 1)],
                value: 1u64,
                exemplars: vec![],
            }],
            start_time: Some(SystemTime::now()),
            time: SystemTime::now(),
        };
        let new_attributes = [KeyValue::new("b", 2)];
        measure.call(2, &new_attributes[..]);

        let (count, new_agg) = agg.call(Some(&mut a));

        assert_eq!(count, 1);
        assert!(new_agg.is_none());
        assert_eq!(a.data_points.len(), 1);
        assert_eq!(a.data_points[0].attributes, new_attributes.to_vec());
        assert_eq!(a.data_points[0].value, 2);
    }

    #[test]
    fn precomputed_sum_aggregation() {
        for temporality in [Temporality::Delta, Temporality::Cumulative] {
            let (measure, agg) =
                AggregateBuilder::<u64>::new(Some(temporality), None).precomputed_sum(true);
            let mut a = Sum {
                data_points: vec![
                    SumDataPoint {
                        attributes: vec![KeyValue::new("a1", 1)],
                        value: 1u64,
                        exemplars: vec![],
                    },
                    SumDataPoint {
                        attributes: vec![KeyValue::new("a2", 1)],
                        value: 2u64,
                        exemplars: vec![],
                    },
                ],
                start_time: SystemTime::now(),
                time: SystemTime::now(),
                temporality: if temporality == Temporality::Delta {
                    Temporality::Cumulative
                } else {
                    Temporality::Delta
                },
                is_monotonic: false,
            };
            let new_attributes = [KeyValue::new("b", 2)];
            measure.call(3, &new_attributes[..]);

            let (count, new_agg) = agg.call(Some(&mut a));

            assert_eq!(count, 1);
            assert!(new_agg.is_none());
            assert_eq!(a.temporality, temporality);
            assert!(a.is_monotonic);
            assert_eq!(a.data_points.len(), 1);
            assert_eq!(a.data_points[0].attributes, new_attributes.to_vec());
            assert_eq!(a.data_points[0].value, 3);
        }
    }

    #[test]
    fn sum_aggregation() {
        for temporality in [Temporality::Delta, Temporality::Cumulative] {
            let (measure, agg) = AggregateBuilder::<u64>::new(Some(temporality), None).sum(true);
            let mut a = Sum {
                data_points: vec![
                    SumDataPoint {
                        attributes: vec![KeyValue::new("a1", 1)],
                        value: 1u64,
                        exemplars: vec![],
                    },
                    SumDataPoint {
                        attributes: vec![KeyValue::new("a2", 1)],
                        value: 2u64,
                        exemplars: vec![],
                    },
                ],
                start_time: SystemTime::now(),
                time: SystemTime::now(),
                temporality: if temporality == Temporality::Delta {
                    Temporality::Cumulative
                } else {
                    Temporality::Delta
                },
                is_monotonic: false,
            };
            let new_attributes = [KeyValue::new("b", 2)];
            measure.call(3, &new_attributes[..]);

            let (count, new_agg) = agg.call(Some(&mut a));

            assert_eq!(count, 1);
            assert!(new_agg.is_none());
            assert_eq!(a.temporality, temporality);
            assert!(a.is_monotonic);
            assert_eq!(a.data_points.len(), 1);
            assert_eq!(a.data_points[0].attributes, new_attributes.to_vec());
            assert_eq!(a.data_points[0].value, 3);
        }
    }

    #[test]
    fn explicit_bucket_histogram_aggregation() {
        for temporality in [Temporality::Delta, Temporality::Cumulative] {
            let (measure, agg) = AggregateBuilder::<u64>::new(Some(temporality), None)
                .explicit_bucket_histogram(vec![1.0], true, true);
            let mut a = Histogram {
                data_points: vec![HistogramDataPoint {
                    attributes: vec![KeyValue::new("a1", 1)],
                    count: 2,
                    bounds: vec![1.0, 2.0],
                    bucket_counts: vec![0, 1, 1],
                    min: None,
                    max: None,
                    sum: 3u64,
                    exemplars: vec![],
                }],
                start_time: SystemTime::now(),
                time: SystemTime::now(),
                temporality: if temporality == Temporality::Delta {
                    Temporality::Cumulative
                } else {
                    Temporality::Delta
                },
            };
            let new_attributes = [KeyValue::new("b", 2)];
            measure.call(3, &new_attributes[..]);

            let (count, new_agg) = agg.call(Some(&mut a));

            assert_eq!(count, 1);
            assert!(new_agg.is_none());
            assert_eq!(a.temporality, temporality);
            assert_eq!(a.data_points.len(), 1);
            assert_eq!(a.data_points[0].attributes, new_attributes.to_vec());
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
            let (measure, agg) = AggregateBuilder::<u64>::new(Some(temporality), None)
                .exponential_bucket_histogram(4, 20, true, true);
            let mut a = ExponentialHistogram {
                data_points: vec![ExponentialHistogramDataPoint {
                    attributes: vec![KeyValue::new("a1", 1)],
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
                start_time: SystemTime::now(),
                time: SystemTime::now(),
                temporality: if temporality == Temporality::Delta {
                    Temporality::Cumulative
                } else {
                    Temporality::Delta
                },
            };
            let new_attributes = [KeyValue::new("b", 2)];
            measure.call(3, &new_attributes[..]);

            let (count, new_agg) = agg.call(Some(&mut a));

            assert_eq!(count, 1);
            assert!(new_agg.is_none());
            assert_eq!(a.temporality, temporality);
            assert_eq!(a.data_points.len(), 1);
            assert_eq!(a.data_points[0].attributes, new_attributes.to_vec());
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
