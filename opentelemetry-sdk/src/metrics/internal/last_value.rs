use std::{
    collections::HashSet,
    sync::{atomic::Ordering, Arc},
    time::SystemTime,
};

use crate::metrics::data::DataPoint;
use opentelemetry::KeyValue;

use super::{Assign, AtomicTracker, Number, ValueMap};

/// Summarizes a set of measurements as the last one made.
pub(crate) struct LastValue<T: Number<T>> {
    value_map: ValueMap<T, Assign>,
}

impl<T: Number<T>> LastValue<T> {
    pub(crate) fn new() -> Self {
        LastValue {
            value_map: ValueMap::new(),
        }
    }

    pub(crate) fn measure(&self, measurement: T, attrs: &[KeyValue]) {
        self.value_map.measure(measurement, attrs);
    }

    pub(crate) fn compute_aggregation(&self, dest: &mut Vec<DataPoint<T>>) {
        let t = SystemTime::now();
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
                start_time: None,
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
                    start_time: None,
                    time: Some(t),
                    value: tracker.get_value(),
                    exemplars: vec![],
                });
            }
        }
    }
}
