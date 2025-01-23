use crate::metrics::{
    data::{self, Aggregation, Gauge, GaugeDataPoint},
    Temporality,
};
use opentelemetry::KeyValue;

use super::{
    aggregate::{AggregateTimeInitiator, AttributeSetFilter},
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

    pub(crate) fn delta(
        &self,
        dest: Option<&mut dyn Aggregation>,
    ) -> (usize, Option<Box<dyn Aggregation>>) {
        let time = self.init_time.delta();

        let s_data = dest.and_then(|d| d.as_mut().downcast_mut::<Gauge<T>>());
        let mut new_agg = if s_data.is_none() {
            Some(data::Gauge {
                data_points: vec![],
                start_time: Some(time.start),
                time: time.current,
            })
        } else {
            None
        };
        let s_data = s_data.unwrap_or_else(|| new_agg.as_mut().expect("present if s_data is none"));
        s_data.start_time = Some(time.start);
        s_data.time = time.current;

        self.value_map
            .collect_and_reset(&mut s_data.data_points, |attributes, aggr| GaugeDataPoint {
                attributes,
                value: aggr.value.get_value(),
                exemplars: vec![],
            });

        (
            s_data.data_points.len(),
            new_agg.map(|a| Box::new(a) as Box<_>),
        )
    }

    pub(crate) fn cumulative(
        &self,
        dest: Option<&mut dyn Aggregation>,
    ) -> (usize, Option<Box<dyn Aggregation>>) {
        let time = self.init_time.cumulative();
        let s_data = dest.and_then(|d| d.as_mut().downcast_mut::<Gauge<T>>());
        let mut new_agg = if s_data.is_none() {
            Some(data::Gauge {
                data_points: vec![],
                start_time: Some(time.start),
                time: time.current,
            })
        } else {
            None
        };
        let s_data = s_data.unwrap_or_else(|| new_agg.as_mut().expect("present if s_data is none"));

        s_data.start_time = Some(time.start);
        s_data.time = time.current;

        self.value_map
            .collect_readonly(&mut s_data.data_points, |attributes, aggr| GaugeDataPoint {
                attributes,
                value: aggr.value.get_value(),
                exemplars: vec![],
            });

        (
            s_data.data_points.len(),
            new_agg.map(|a| Box::new(a) as Box<_>),
        )
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
