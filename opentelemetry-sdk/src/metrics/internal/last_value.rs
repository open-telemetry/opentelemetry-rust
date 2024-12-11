use std::{mem::replace, ops::DerefMut, sync::Mutex, time::SystemTime};

use crate::metrics::data::{self, Aggregation, GaugeDataPoint};
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

    pub(crate) fn delta(
        &self,
        dest: Option<&mut dyn Aggregation>,
    ) -> (usize, Option<Box<dyn Aggregation>>) {
        let time = SystemTime::now();
        let start_time = self
            .start
            .lock()
            .map(|mut start| replace(start.deref_mut(), time))
            .unwrap_or(time);

        let s_data = dest.and_then(|d| d.as_mut().downcast_mut::<data::Gauge<T>>());
        let mut new_agg = if s_data.is_none() {
            Some(data::Gauge {
                data_points: vec![],
                start_time: Some(start_time),
                time,
            })
        } else {
            None
        };
        let s_data = s_data.unwrap_or_else(|| new_agg.as_mut().expect("present if s_data is none"));
        s_data.start_time = Some(start_time);
        s_data.time = time;

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
        let time = SystemTime::now();
        let start_time = self.start.lock().map(|start| *start).unwrap_or(time);
        let s_data = dest.and_then(|d| d.as_mut().downcast_mut::<data::Gauge<T>>());
        let mut new_agg = if s_data.is_none() {
            Some(data::Gauge {
                data_points: vec![],
                start_time: Some(start_time),
                time,
            })
        } else {
            None
        };
        let s_data = s_data.unwrap_or_else(|| new_agg.as_mut().expect("present if s_data is none"));

        s_data.start_time = Some(start_time);
        s_data.time = time;

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
