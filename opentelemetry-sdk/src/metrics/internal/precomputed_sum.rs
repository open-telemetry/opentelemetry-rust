use opentelemetry::KeyValue;

use crate::metrics::data::{self, Aggregation, DataPoint, Temporality};

use super::{
    collect_data_points_readonly, collect_data_points_reset, Assign, AtomicTracker, Number,
    ValueMap,
};
use std::{collections::HashMap, sync::Mutex, time::SystemTime};

/// Summarizes a set of pre-computed sums as their arithmetic sum.
pub(crate) struct PrecomputedSum<T: Number<T>> {
    value_map: ValueMap<T, T, Assign>,
    monotonic: bool,
    start: Mutex<SystemTime>,
    reported: Mutex<HashMap<Vec<KeyValue>, T>>,
}

impl<T: Number<T>> PrecomputedSum<T> {
    pub(crate) fn new(monotonic: bool) -> Self {
        PrecomputedSum {
            value_map: ValueMap::new(),
            monotonic,
            start: Mutex::new(SystemTime::now()),
            reported: Mutex::new(Default::default()),
        }
    }

    pub(crate) fn measure(&self, measurement: T, attrs: &[KeyValue]) {
        // The argument index is not applicable to PrecomputedSum.
        self.value_map.measure(measurement, attrs, 0);
    }

    pub(crate) fn delta(
        &self,
        dest: Option<&mut dyn Aggregation>,
    ) -> (usize, Option<Box<dyn Aggregation>>) {
        let t = SystemTime::now();
        let prev_start = self.start.lock().map(|start| *start).unwrap_or(t);

        let s_data = dest.and_then(|d| d.as_mut().downcast_mut::<data::Sum<T>>());
        let mut new_agg = if s_data.is_none() {
            Some(data::Sum {
                data_points: vec![],
                temporality: Temporality::Delta,
                is_monotonic: self.monotonic,
            })
        } else {
            None
        };
        let s_data = s_data.unwrap_or_else(|| new_agg.as_mut().expect("present if s_data is none"));
        s_data.data_points.clear();
        s_data.temporality = Temporality::Delta;
        s_data.is_monotonic = self.monotonic;

        let mut reported = match self.reported.lock() {
            Ok(r) => r,
            Err(_) => return (0, None),
        };

        let Ok(mut trackers) = self.value_map.trackers.write() else {
            return (0, None);
        };

        // same logic as in `collect_data_points_drain`
        let mut new_reported = HashMap::with_capacity(trackers.list.len() + 1);

        collect_data_points_reset(
            &self.value_map.no_attribs_tracker,
            &mut trackers,
            &mut s_data.data_points,
            |attributes, tracker| {
                let prev_value = *reported.get(&attributes).unwrap_or(&T::default());
                let curr_value = tracker.get_and_reset_value();
                new_reported.insert(attributes.clone(), curr_value);
                DataPoint {
                    attributes,
                    start_time: Some(prev_start),
                    time: Some(t),
                    value: curr_value - prev_value,
                    exemplars: vec![],
                }
            },
        );

        // The delta collection cycle resets.
        if let Ok(mut start) = self.start.lock() {
            *start = t;
        }

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
        let t = SystemTime::now();
        let prev_start = self.start.lock().map(|start| *start).unwrap_or(t);

        let s_data = dest.and_then(|d| d.as_mut().downcast_mut::<data::Sum<T>>());
        let mut new_agg = if s_data.is_none() {
            Some(data::Sum {
                data_points: vec![],
                temporality: Temporality::Cumulative,
                is_monotonic: self.monotonic,
            })
        } else {
            None
        };
        let s_data = s_data.unwrap_or_else(|| new_agg.as_mut().expect("present if s_data is none"));
        s_data.data_points.clear();
        s_data.temporality = Temporality::Cumulative;
        s_data.is_monotonic = self.monotonic;

        let Ok(trackers) = self.value_map.trackers.read() else {
            return (0, None);
        };

        collect_data_points_readonly(
            &self.value_map.no_attribs_tracker,
            &trackers,
            &mut s_data.data_points,
            |attributes, tracker| DataPoint {
                attributes,
                start_time: Some(prev_start),
                time: Some(t),
                value: tracker.get_value(),
                exemplars: vec![],
            },
        );

        (
            s_data.data_points.len(),
            new_agg.map(|a| Box::new(a) as Box<_>),
        )
    }
}
