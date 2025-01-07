use opentelemetry::KeyValue;

use crate::metrics::data::{self, Aggregation, SumDataPoint};
use crate::metrics::Temporality;

use super::aggregate::{
    AggregateTime, AggregateTimeInitiator, AggregationData, AttributeSetFilter, InitAggregationData,
};
use super::{last_value::Assign, AtomicTracker, Number, ValueMap};
use super::{ComputeAggregation, Measure};
use std::{collections::HashMap, sync::Mutex};

/// Summarizes a set of pre-computed sums as their arithmetic sum.
pub(crate) struct PrecomputedSum<T: Number> {
    value_map: ValueMap<Assign<T>>,
    init_time: AggregateTimeInitiator,
    temporality: Temporality,
    filter: AttributeSetFilter,
    monotonic: bool,
    reported: Mutex<HashMap<Vec<KeyValue>, T>>,
}

impl<T: Number> PrecomputedSum<T> {
    pub(crate) fn new(
        temporality: Temporality,
        filter: AttributeSetFilter,
        monotonic: bool,
    ) -> Self {
        PrecomputedSum {
            value_map: ValueMap::new(()),
            init_time: AggregateTimeInitiator::default(),
            temporality,
            filter,
            monotonic,
            reported: Mutex::new(Default::default()),
        }
    }

    fn delta(&self, dest: Option<&mut dyn Aggregation>) -> (usize, Option<Box<dyn Aggregation>>) {
        let mut s_data = AggregationData::init(self, dest, self.init_time.delta());

        let mut reported = match self.reported.lock() {
            Ok(r) => r,
            Err(_) => return (0, None),
        };
        let mut new_reported = HashMap::with_capacity(reported.len());

        self.value_map
            .collect_and_reset(&mut s_data.data_points, |attributes, aggr| {
                let value = aggr.value.get_value();
                new_reported.insert(attributes.clone(), value);
                let delta = value - *reported.get(&attributes).unwrap_or(&T::default());
                SumDataPoint {
                    attributes,
                    value: delta,
                    exemplars: vec![],
                }
            });

        *reported = new_reported;
        drop(reported); // drop before values guard is dropped

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

impl<T> Measure<T> for PrecomputedSum<T>
where
    T: Number,
{
    fn call(&self, measurement: T, attrs: &[KeyValue]) {
        self.filter.apply(attrs, |filtered| {
            self.value_map.measure(measurement, filtered);
        })
    }
}

impl<T> ComputeAggregation for PrecomputedSum<T>
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

impl<T> InitAggregationData for PrecomputedSum<T>
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
