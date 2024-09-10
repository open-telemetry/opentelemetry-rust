use std::{sync::Mutex, time::SystemTime};

use crate::metrics::data::DataPoint;
use opentelemetry::KeyValue;

use super::{
    collect_data_points_readonly, collect_data_points_reset, Assign, AtomicTracker, Number,
    ValueMap,
};

/// Summarizes a set of measurements as the last one made.
pub(crate) struct LastValue<T: Number<T>> {
    value_map: ValueMap<T, T, Assign>,
    start: Mutex<SystemTime>,
}

impl<T: Number<T>> LastValue<T> {
    pub(crate) fn new() -> Self {
        LastValue {
            value_map: ValueMap::new(),
            start: Mutex::new(SystemTime::now()),
        }
    }

    pub(crate) fn measure(&self, measurement: T, attrs: &[KeyValue]) {
        // The argument index is not applicable to LastValue.
        self.value_map.measure(measurement, attrs, 0);
    }

    pub(crate) fn compute_aggregation_delta(&self, dest: &mut Vec<DataPoint<T>>) {
        let t = SystemTime::now();
        let prev_start = self.start.lock().map(|start| *start).unwrap_or(t);
        dest.clear();

        let Ok(mut trackers) = self.value_map.trackers.write() else {
            return;
        };

        collect_data_points_reset(
            &self.value_map.no_attribs_tracker,
            &mut trackers,
            dest,
            |attributes, tracker| DataPoint {
                attributes,
                start_time: Some(prev_start),
                time: Some(t),
                value: tracker.get_and_reset_value(),
                exemplars: vec![],
            },
        );

        // The delta collection cycle resets.
        if let Ok(mut start) = self.start.lock() {
            *start = t;
        }
    }

    pub(crate) fn compute_aggregation_cumulative(&self, dest: &mut Vec<DataPoint<T>>) {
        let t = SystemTime::now();
        let prev_start = self.start.lock().map(|start| *start).unwrap_or(t);

        dest.clear();

        let Ok(trackers) = self.value_map.trackers.read() else {
            return;
        };

        collect_data_points_readonly(
            &self.value_map.no_attribs_tracker,
            &trackers,
            dest,
            |attributes, tracker| DataPoint {
                attributes,
                start_time: Some(prev_start),
                time: Some(t),
                value: tracker.get_value(),
                exemplars: vec![],
            },
        );
    }
}
