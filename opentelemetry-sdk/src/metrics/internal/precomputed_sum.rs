use opentelemetry::KeyValue;

use crate::metrics::data::{self, Aggregation, SumDataPoint};
use crate::metrics::Temporality;

use super::aggregate::AggregateTimeInitiator;
use super::ComputeAggregation;
use super::{last_value::Assign, AtomicTracker, Number, ValueMap};
use std::sync::Arc;
use std::{collections::HashMap, sync::Mutex};

/// Summarizes a set of pre-computed sums as their arithmetic sum.
pub(crate) struct PrecomputedSum<T: Number> {
    value_map: ValueMap<Assign<T>>,
    init_time: AggregateTimeInitiator,
    temporality: Temporality,
    monotonic: bool,
    reported: Mutex<HashMap<Vec<KeyValue>, T>>,
}

impl<T: Number> PrecomputedSum<T> {
    pub(crate) fn new(temporality: Temporality, monotonic: bool) -> Self {
        PrecomputedSum {
            value_map: ValueMap::new(()),
            init_time: AggregateTimeInitiator::default(),
            temporality,
            monotonic,
            reported: Mutex::new(Default::default()),
        }
    }

    pub(crate) fn measure(&self, measurement: T, attrs: &[KeyValue]) {
        // The argument index is not applicable to PrecomputedSum.
        self.value_map.measure(measurement, attrs);
    }

    pub(crate) fn delta(
        &self,
        dest: Option<&mut dyn Aggregation>,
    ) -> (usize, Option<Box<dyn Aggregation>>) {
        let time = self.init_time.delta();

        let s_data = dest.and_then(|d| d.as_mut().downcast_mut::<data::Sum<T>>());
        let mut new_agg = if s_data.is_none() {
            Some(data::Sum {
                data_points: vec![],
                start_time: time.start,
                time: time.current,
                temporality: Temporality::Delta,
                is_monotonic: self.monotonic,
            })
        } else {
            None
        };
        let s_data = s_data.unwrap_or_else(|| new_agg.as_mut().expect("present if s_data is none"));
        s_data.start_time = time.start;
        s_data.time = time.current;
        s_data.temporality = Temporality::Delta;
        s_data.is_monotonic = self.monotonic;

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

        let s_data = dest.and_then(|d| d.as_mut().downcast_mut::<data::Sum<T>>());
        let mut new_agg = if s_data.is_none() {
            Some(data::Sum {
                data_points: vec![],
                start_time: time.start,
                time: time.current,
                temporality: Temporality::Cumulative,
                is_monotonic: self.monotonic,
            })
        } else {
            None
        };
        let s_data = s_data.unwrap_or_else(|| new_agg.as_mut().expect("present if s_data is none"));
        s_data.start_time = time.start;
        s_data.time = time.current;
        s_data.temporality = Temporality::Cumulative;
        s_data.is_monotonic = self.monotonic;

        self.value_map
            .collect_readonly(&mut s_data.data_points, |attributes, aggr| SumDataPoint {
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

impl<T> ComputeAggregation for Arc<PrecomputedSum<T>>
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
