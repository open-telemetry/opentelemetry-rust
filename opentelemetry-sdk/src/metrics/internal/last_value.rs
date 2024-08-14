use std::{
    collections::HashSet,
    sync::{atomic::Ordering, Arc, Mutex},
    time::SystemTime,
};

use crate::metrics::data::DataPoint;
use opentelemetry::KeyValue;

use super::{Assign, AtomicTracker, Number, ValueMap};

/// Summarizes a set of measurements as the last one made.
pub(crate) struct LastValue<T: Number<T>> {
    value_map: ValueMap<T, Assign>,
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
        self.value_map.measure(measurement, attrs);
    }

    pub(crate) fn compute_aggregation_delta(&self, dest: &mut Vec<DataPoint<T>>) {
        let t = SystemTime::now();
        let prev_start = self.start.lock().map(|start| *start).unwrap_or(t);
        dest.clear();

        // Max number of data points need to account for the special casing
        // of the no attribute value + overflow attribute.
        let n = self.value_map.count.load(Ordering::SeqCst) + 2;
        if n > dest.capacity() {
            dest.reserve_exact(n - dest.capacity());
        }

        if self
            .value_map
            .has_no_attribute_value
            .swap(false, Ordering::AcqRel)
        {
            dest.push(DataPoint {
                attributes: vec![],
                start_time: Some(prev_start),
                time: Some(t),
                value: self.value_map.no_attribute_tracker.get_and_reset_value(),
                exemplars: vec![],
            });
        }

        let mut trackers = match self.value_map.trackers.write() {
            Ok(v) => v,
            _ => return,
        };

        let mut seen = HashSet::new();
        for (attrs, tracker) in trackers.drain() {
            if seen.insert(Arc::as_ptr(&tracker)) {
                dest.push(DataPoint {
                    attributes: attrs.clone(),
                    start_time: Some(prev_start),
                    time: Some(t),
                    value: tracker.get_value(),
                    exemplars: vec![],
                });
            }
        }

        // The delta collection cycle resets.
        if let Ok(mut start) = self.start.lock() {
            *start = t;
        }
        self.value_map.count.store(0, Ordering::SeqCst);
    }

    pub(crate) fn compute_aggregation_cumulative(&self, dest: &mut Vec<DataPoint<T>>) {
        let t = SystemTime::now();
        let prev_start = self.start.lock().map(|start| *start).unwrap_or(t);

        dest.clear();

        // Max number of data points need to account for the special casing
        // of the no attribute value + overflow attribute.
        let n = self.value_map.count.load(Ordering::SeqCst) + 2;
        if n > dest.capacity() {
            dest.reserve_exact(n - dest.capacity());
        }

        if self
            .value_map
            .has_no_attribute_value
            .load(Ordering::Acquire)
        {
            dest.push(DataPoint {
                attributes: vec![],
                start_time: Some(prev_start),
                time: Some(t),
                value: self.value_map.no_attribute_tracker.get_value(),
                exemplars: vec![],
            });
        }

        let trackers = match self.value_map.trackers.write() {
            Ok(v) => v,
            _ => return,
        };

        let mut seen = HashSet::new();
        for (attrs, tracker) in trackers.iter() {
            if seen.insert(Arc::as_ptr(tracker)) {
                dest.push(DataPoint {
                    attributes: attrs.clone(),
                    start_time: Some(prev_start),
                    time: Some(t),
                    value: tracker.get_value(),
                    exemplars: vec![],
                });
            }
        }
    }
}
