use crate::metrics::{
    data::{self, Aggregation, GaugeDataPoint},
    Temporality,
};
use opentelemetry::KeyValue;

use super::{
    aggregate::{
        AggregateTime, AggregateTimeInitiator, AggregationData, AttributeSetFilter,
        InitAggregationData,
    },
    Aggregator, AtomicTracker, AtomicallyUpdate, ComputeAggregation, Measure, Number, ValueMap,
};

/// this is reused by PrecomputedSum
pub(crate) struct Assign<T>
where
    T: AtomicallyUpdate<T>,
{
    pub(crate) value: T::AtomicTracker,
}

impl<T> Aggregator for Assign<T>
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
        self.value.store(value)
    }

    fn clone_and_reset(&self, _: &()) -> Self {
        Self {
            value: T::new_atomic_tracker(self.value.get_and_reset_value()),
        }
    }
}

/// Summarizes a set of measurements as the last one made.
pub(crate) struct LastValue<T: Number> {
    value_map: ValueMap<Assign<T>>,
    init_time: AggregateTimeInitiator,
    temporality: Temporality,
    filter: AttributeSetFilter,
}

impl<T: Number> LastValue<T> {
    pub(crate) fn new(temporality: Temporality, filter: AttributeSetFilter) -> Self {
        LastValue {
            value_map: ValueMap::new(()),
            init_time: AggregateTimeInitiator::default(),
            temporality,
            filter,
        }
    }

    fn delta(&self, dest: Option<&mut dyn Aggregation>) -> (usize, Option<Box<dyn Aggregation>>) {
        let mut s_data = AggregationData::init(self, dest, self.init_time.delta());

        self.value_map
            .collect_and_reset(&mut s_data.data_points, |attributes, aggr| GaugeDataPoint {
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
            .collect_readonly(&mut s_data.data_points, |attributes, aggr| GaugeDataPoint {
                attributes,
                value: aggr.value.get_value(),
                exemplars: vec![],
            });

        (s_data.data_points.len(), s_data.into_new_boxed())
    }
}

impl<T> Measure<T> for LastValue<T>
where
    T: Number,
{
    fn call(&self, measurement: T, attrs: &[KeyValue]) {
        self.filter.apply(attrs, |filtered| {
            self.value_map.measure(measurement, filtered);
        })
    }
}

impl<T> ComputeAggregation for LastValue<T>
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

impl<T> InitAggregationData for LastValue<T>
where
    T: Number,
{
    type Aggr = data::Gauge<T>;

    fn create_new(&self, time: AggregateTime) -> Self::Aggr {
        data::Gauge {
            data_points: vec![],
            start_time: Some(time.start),
            time: time.current,
        }
    }

    fn reset_existing(&self, existing: &mut Self::Aggr, time: AggregateTime) {
        existing.data_points.clear();
        existing.start_time = Some(time.start);
        existing.time = time.current;
    }
}
