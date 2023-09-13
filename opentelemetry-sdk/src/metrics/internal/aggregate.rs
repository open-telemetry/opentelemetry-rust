use std::{marker, sync::Arc};

use once_cell::sync::Lazy;
use opentelemetry::KeyValue;

use crate::{
    metrics::data::{Aggregation, Gauge, Temporality},
    AttributeSet,
};

use super::{
    histogram::Histogram,
    last_value::LastValue,
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
    pub(crate) fn last_value(&self) -> (impl Measure<T>, impl ComputeAggregation) {
        // Delta temporality is the only temporality that makes semantic sense for
        // a last-value aggregate.
        let lv_filter = Arc::new(LastValue::new());
        let lv_agg = Arc::clone(&lv_filter);

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
            self.filter(move |n, a| s.measure(n, a)),
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
            self.filter(move |n, a| s.measure(n, a)),
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
            self.filter(move |n, a| h.measure(n, a)),
            move |dest: Option<&mut dyn Aggregation>| match t {
                Some(Temporality::Delta) => agg_h.delta(dest),
                _ => agg_h.cumulative(dest),
            },
        )
    }
}
