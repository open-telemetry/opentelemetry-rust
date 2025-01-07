use crate::metrics::data::{self, Aggregation, SumDataPoint};
use crate::metrics::Temporality;
use opentelemetry::KeyValue;

use super::aggregate::{
    AggregateTime, AggregateTimeInitiator, AggregationData, AttributeSetFilter, InitAggregationData,
};
use super::{Aggregator, AtomicTracker, ComputeAggregation, Measure, Number};
use super::{AtomicallyUpdate, ValueMap};

struct Increment<T>
where
    T: AtomicallyUpdate<T>,
{
    value: T::AtomicTracker,
}

impl<T> Aggregator for Increment<T>
where
    T: Number,
{
    type InitConfig = ();
    type PreComputedValue = T;

    fn create(_init: &()) -> Self {
        Self {
            value: T::new_atomic_tracker(T::default()),
        }
    }

    fn update(&self, value: T) {
        self.value.add(value)
    }

    fn clone_and_reset(&self, _: &()) -> Self {
        Self {
            value: T::new_atomic_tracker(self.value.get_and_reset_value()),
        }
    }
}

/// Summarizes a set of measurements made as their arithmetic sum.
pub(crate) struct Sum<T: Number> {
    value_map: ValueMap<Increment<T>>,
    init_time: AggregateTimeInitiator,
    temporality: Temporality,
    filter: AttributeSetFilter,
    monotonic: bool,
}

impl<T: Number> Sum<T> {
    /// Returns an aggregator that summarizes a set of measurements as their
    /// arithmetic sum.
    ///
    /// Each sum is scoped by attributes and the aggregation cycle the measurements
    /// were made in.
    pub(crate) fn new(
        temporality: Temporality,
        filter: AttributeSetFilter,
        monotonic: bool,
    ) -> Self {
        Sum {
            value_map: ValueMap::new(()),
            init_time: AggregateTimeInitiator::default(),
            temporality,
            filter,
            monotonic,
        }
    }

    fn delta(&self, dest: Option<&mut dyn Aggregation>) -> (usize, Option<Box<dyn Aggregation>>) {
        let mut s_data = AggregationData::init(self, dest, self.init_time.delta());

        self.value_map
            .collect_and_reset(&mut s_data.data_points, |attributes, aggr| SumDataPoint {
                attributes,
                value: aggr.value.get_value(),
                exemplars: vec![],
            });

        (s_data.data_points.len(), s_data.into_new_boxed())
    }

    fn cumulative(
        &self,
        dest: Option<&mut dyn Aggregation>,
    ) -> (usize, Option<Box<dyn Aggregation>>) {
        let mut s_data = AggregationData::init(self, dest, self.init_time.cumulative());

        self.value_map
            .collect_readonly(&mut s_data.data_points, |attributes, aggr| SumDataPoint {
                attributes,
                value: aggr.value.get_value(),
                exemplars: vec![],
            });

        (s_data.data_points.len(), s_data.into_new_boxed())
    }
}

impl<T> Measure<T> for Sum<T>
where
    T: Number,
{
    fn call(&self, measurement: T, attrs: &[KeyValue]) {
        self.filter.apply(attrs, |filtered| {
            self.value_map.measure(measurement, filtered);
        })
    }
}

impl<T> ComputeAggregation for Sum<T>
where
    T: Number,
{
    fn call(&self, dest: Option<&mut dyn Aggregation>) -> (usize, Option<Box<dyn Aggregation>>) {
        match self.temporality {
            Temporality::Delta => self.delta(dest),
            _ => self.cumulative(dest),
        }
    }
}

impl<T> InitAggregationData for Sum<T>
where
    T: Number,
{
    type Aggr = data::Sum<T>;

    fn create_new(&self, time: AggregateTime) -> Self::Aggr {
        data::Sum {
            data_points: vec![],
            start_time: time.start,
            time: time.current,
            temporality: self.temporality,
            is_monotonic: self.monotonic,
        }
    }

    fn reset_existing(&self, existing: &mut Self::Aggr, time: AggregateTime) {
        existing.data_points.clear();
        existing.start_time = time.start;
        existing.time = time.current;
        existing.temporality = self.temporality;
        existing.is_monotonic = self.monotonic;
    }
}
