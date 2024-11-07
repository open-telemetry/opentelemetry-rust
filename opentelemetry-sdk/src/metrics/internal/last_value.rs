use std::{mem::replace, ops::DerefMut, sync::Mutex, time::SystemTime};

use crate::metrics::data::DataPoint;
use opentelemetry::KeyValue;

use super::{Aggregator, AtomicTracker, AtomicallyUpdate, Number, ValueMap};

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
    start: Mutex<SystemTime>,
}

impl<T: Number> LastValue<T> {
    pub(crate) fn new() -> Self {
        LastValue {
            value_map: ValueMap::new(()),
            start: Mutex::new(SystemTime::now()),
        }
    }

    pub(crate) fn measure(&self, measurement: T, attrs: &[KeyValue]) {
        // The argument index is not applicable to LastValue.
        self.value_map.measure(measurement, attrs);
    }

    pub(crate) fn compute_aggregation_delta(&self, dest: &mut Vec<DataPoint<T>>) {
        let t = SystemTime::now();
        let prev_start = self
            .start
            .lock()
            .map(|mut start| replace(start.deref_mut(), t))
            .unwrap_or(t);
        self.value_map
            .collect_and_reset(dest, |attributes, aggr| DataPoint {
                attributes,
                start_time: Some(prev_start),
                time: Some(t),
                value: aggr.value.get_value(),
                exemplars: vec![],
            });
    }

    pub(crate) fn compute_aggregation_cumulative(&self, dest: &mut Vec<DataPoint<T>>) {
        let t = SystemTime::now();
        let prev_start = self.start.lock().map(|start| *start).unwrap_or(t);
        self.value_map
            .collect_readonly(dest, |attributes, aggr| DataPoint {
                attributes,
                start_time: Some(prev_start),
                time: Some(t),
                value: aggr.value.get_value(),
                exemplars: vec![],
            });
    }
}
