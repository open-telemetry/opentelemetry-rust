use std::collections::HashSet;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::vec;
use std::{collections::HashMap, sync::Mutex, time::SystemTime};

use crate::metrics::data::{self, Aggregation, DataPoint, Temporality};
use opentelemetry::KeyValue;

use super::{Assign, Increment, ValueMap};
use super::{AtomicTracker, Number};

/// Summarizes a set of measurements made as their arithmetic sum.
pub(crate) struct Sum<T: Number<T>> {
    value_map: ValueMap<T, T, Increment>,
    monotonic: bool,
    start: Mutex<SystemTime>,
}

impl<T: Number<T>> Sum<T> {
    /// Returns an aggregator that summarizes a set of measurements as their
    /// arithmetic sum.
    ///
    /// Each sum is scoped by attributes and the aggregation cycle the measurements
    /// were made in.
    pub(crate) fn new(monotonic: bool) -> Self {
        Sum {
            value_map: ValueMap::new(),
            monotonic,
            start: Mutex::new(SystemTime::now()),
        }
    }

    pub(crate) fn measure(&self, measurement: T, attrs: &[KeyValue]) {
        // The argument index is not applicable to Sum.
        self.value_map.measure(measurement, attrs, 0);
    }

    pub(crate) fn delta(
        &self,
        dest: Option<&mut dyn Aggregation>,
    ) -> (usize, Option<Box<dyn Aggregation>>) {
        let t = SystemTime::now();

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
        s_data.temporality = Temporality::Delta;
        s_data.is_monotonic = self.monotonic;
        s_data.data_points.clear();

        // Max number of data points need to account for the special casing
        // of the no attribute value + overflow attribute.
        let n = self.value_map.count.load(Ordering::SeqCst) + 2;
        if n > s_data.data_points.capacity() {
            s_data
                .data_points
                .reserve_exact(n - s_data.data_points.capacity());
        }

        let prev_start = self.start.lock().map(|start| *start).unwrap_or(t);
        if self
            .value_map
            .has_no_attribute_value
            .swap(false, Ordering::AcqRel)
        {
            s_data.data_points.push(DataPoint {
                attributes: vec![],
                start_time: Some(prev_start),
                time: Some(t),
                value: self.value_map.no_attribute_tracker.get_and_reset_value(),
                exemplars: vec![],
            });
        }

        let mut trackers = match self.value_map.trackers.write() {
            Ok(v) => v,
            Err(_) => return (0, None),
        };

        let mut seen = HashSet::new();
        for (attrs, tracker) in trackers.drain() {
            if seen.insert(Arc::as_ptr(&tracker)) {
                s_data.data_points.push(DataPoint {
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
        s_data.temporality = Temporality::Cumulative;
        s_data.is_monotonic = self.monotonic;
        s_data.data_points.clear();

        // Max number of data points need to account for the special casing
        // of the no attribute value + overflow attribute.
        let n = self.value_map.count.load(Ordering::SeqCst) + 2;
        if n > s_data.data_points.capacity() {
            s_data
                .data_points
                .reserve_exact(n - s_data.data_points.capacity());
        }

        let prev_start = self.start.lock().map(|start| *start).unwrap_or(t);

        if self
            .value_map
            .has_no_attribute_value
            .load(Ordering::Acquire)
        {
            s_data.data_points.push(DataPoint {
                attributes: vec![],
                start_time: Some(prev_start),
                time: Some(t),
                value: self.value_map.no_attribute_tracker.get_value(),
                exemplars: vec![],
            });
        }

        let trackers = match self.value_map.trackers.write() {
            Ok(v) => v,
            Err(_) => return (0, None),
        };

        // TODO: This will use an unbounded amount of memory if there
        // are unbounded number of attribute sets being aggregated. Attribute
        // sets that become "stale" need to be forgotten so this will not
        // overload the system.
        let mut seen = HashSet::new();
        for (attrs, tracker) in trackers.iter() {
            if seen.insert(Arc::as_ptr(tracker)) {
                s_data.data_points.push(DataPoint {
                    attributes: attrs.clone(),
                    start_time: Some(prev_start),
                    time: Some(t),
                    value: tracker.get_value(),
                    exemplars: vec![],
                });
            }
        }

        (
            s_data.data_points.len(),
            new_agg.map(|a| Box::new(a) as Box<_>),
        )
    }
}

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

        // Max number of data points need to account for the special casing
        // of the no attribute value + overflow attribute.
        let n = self.value_map.count.load(Ordering::SeqCst) + 2;
        if n > s_data.data_points.capacity() {
            s_data
                .data_points
                .reserve_exact(n - s_data.data_points.capacity());
        }
        let mut new_reported = HashMap::with_capacity(n);
        let mut reported = match self.reported.lock() {
            Ok(r) => r,
            Err(_) => return (0, None),
        };

        if self
            .value_map
            .has_no_attribute_value
            .swap(false, Ordering::AcqRel)
        {
            let value = self.value_map.no_attribute_tracker.get_value();
            let delta = value - *reported.get(&vec![]).unwrap_or(&T::default());
            new_reported.insert(vec![], value);

            s_data.data_points.push(DataPoint {
                attributes: vec![],
                start_time: Some(prev_start),
                time: Some(t),
                value: delta,
                exemplars: vec![],
            });
        }

        let mut trackers = match self.value_map.trackers.write() {
            Ok(v) => v,
            Err(_) => return (0, None),
        };

        let mut seen = HashSet::new();
        for (attrs, tracker) in trackers.drain() {
            if seen.insert(Arc::as_ptr(&tracker)) {
                let value = tracker.get_value();
                let delta = value - *reported.get(&attrs).unwrap_or(&T::default());
                new_reported.insert(attrs.clone(), value);
                s_data.data_points.push(DataPoint {
                    attributes: attrs.clone(),
                    start_time: Some(prev_start),
                    time: Some(t),
                    value: delta,
                    exemplars: vec![],
                });
            }
        }

        // The delta collection cycle resets.
        if let Ok(mut start) = self.start.lock() {
            *start = t;
        }
        self.value_map.count.store(0, Ordering::SeqCst);

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

        // Max number of data points need to account for the special casing
        // of the no attribute value + overflow attribute.
        let n = self.value_map.count.load(Ordering::SeqCst) + 2;
        if n > s_data.data_points.capacity() {
            s_data
                .data_points
                .reserve_exact(n - s_data.data_points.capacity());
        }

        if self
            .value_map
            .has_no_attribute_value
            .load(Ordering::Acquire)
        {
            s_data.data_points.push(DataPoint {
                attributes: vec![],
                start_time: Some(prev_start),
                time: Some(t),
                value: self.value_map.no_attribute_tracker.get_value(),
                exemplars: vec![],
            });
        }

        let trackers = match self.value_map.trackers.write() {
            Ok(v) => v,
            Err(_) => return (0, None),
        };

        let mut seen = HashSet::new();
        for (attrs, tracker) in trackers.iter() {
            if seen.insert(Arc::as_ptr(tracker)) {
                s_data.data_points.push(DataPoint {
                    attributes: attrs.clone(),
                    start_time: Some(prev_start),
                    time: Some(t),
                    value: tracker.get_value(),
                    exemplars: vec![],
                });
            }
        }

        (
            s_data.data_points.len(),
            new_agg.map(|a| Box::new(a) as Box<_>),
        )
    }
}
