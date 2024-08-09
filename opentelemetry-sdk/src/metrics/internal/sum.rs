use std::collections::HashSet;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::vec;
use std::{
    collections::HashMap,
    sync::{Mutex, RwLock},
    time::SystemTime,
};

use crate::metrics::data::{self, Aggregation, DataPoint, Temporality};
use crate::metrics::AttributeSet;
use once_cell::sync::Lazy;
use opentelemetry::KeyValue;
use opentelemetry::{global, metrics::MetricsError};

use super::{aggregate::is_under_cardinality_limit, AtomicTracker, Number};

pub(crate) static STREAM_OVERFLOW_ATTRIBUTES: Lazy<Vec<KeyValue>> =
    Lazy::new(|| vec![KeyValue::new("otel.metric.overflow", "true")]);

/// Abstracts the update operation for a measurement.
trait Operation {
    fn update_tracker<T: 'static, AT: AtomicTracker<T>>(tracker: &AT, value: T);
}

struct Increment;

impl Operation for Increment {
    fn update_tracker<T: 'static, AT: AtomicTracker<T>>(tracker: &AT, value: T) {
        tracker.add(value);
    }
}

struct Assign;

impl Operation for Assign {
    fn update_tracker<T: 'static, AT: AtomicTracker<T>>(tracker: &AT, value: T) {
        tracker.store(value);
    }
}

/// The storage for sums.
///
/// This structure is parametrized by an `Operation` that indicates how
/// updates to the underlying value trackers should be performed.
struct ValueMap<T: Number<T>, O> {
    /// Trackers store the values associated with different attribute sets.
    trackers: RwLock<HashMap<Vec<KeyValue>, Arc<T::AtomicTracker>>>,
    /// Number of different attribute set stored in the `trackers` map.
    count: AtomicUsize,
    /// Indicates whether a value with no attributes has been stored.
    has_no_attribute_value: AtomicBool,
    /// Tracker for values with no attributes attached.
    no_attribute_tracker: T::AtomicTracker,
    phantom: PhantomData<O>,
}

impl<T: Number<T>, O> Default for ValueMap<T, O> {
    fn default() -> Self {
        ValueMap::new()
    }
}

impl<T: Number<T>, O> ValueMap<T, O> {
    fn new() -> Self {
        ValueMap {
            trackers: RwLock::new(HashMap::new()),
            has_no_attribute_value: AtomicBool::new(false),
            no_attribute_tracker: T::new_atomic_tracker(),
            count: AtomicUsize::new(0),
            phantom: PhantomData,
        }
    }
}

impl<T: Number<T>, O: Operation> ValueMap<T, O> {
    fn measure(&self, measurement: T, attributes: &[KeyValue]) {
        if attributes.is_empty() {
            O::update_tracker(&self.no_attribute_tracker, measurement);
            self.has_no_attribute_value.store(true, Ordering::Release);
            return;
        }

        let Ok(trackers) = self.trackers.read() else {
            return;
        };

        // Try to retrieve and update the tracker with the attributes in the provided order first
        if let Some(tracker) = trackers.get(attributes) {
            O::update_tracker(&**tracker, measurement);
            return;
        }

        // Try to retrieve and update the tracker with the attributes sorted.
        let sorted_attrs = AttributeSet::from(attributes).into_vec();
        if let Some(tracker) = trackers.get(sorted_attrs.as_slice()) {
            O::update_tracker(&**tracker, measurement);
            return;
        }

        // Give up the read lock before acquiring the write lock.
        drop(trackers);

        let Ok(mut trackers) = self.trackers.write() else {
            return;
        };

        // Recheck both the provided and sorted orders after acquiring the write lock
        // in case another thread has pushed an update in the meantime.
        if let Some(tracker) = trackers.get(attributes) {
            O::update_tracker(&**tracker, measurement);
        } else if let Some(tracker) = trackers.get(sorted_attrs.as_slice()) {
            O::update_tracker(&**tracker, measurement);
        } else if is_under_cardinality_limit(self.count.load(Ordering::SeqCst)) {
            let new_tracker = Arc::new(T::new_atomic_tracker());
            O::update_tracker(&*new_tracker, measurement);

            // Insert tracker with the attributes in the provided and sorted orders
            trackers.insert(attributes.to_vec(), new_tracker.clone());
            trackers.insert(sorted_attrs, new_tracker);

            self.count.fetch_add(1, Ordering::SeqCst);
        } else if let Some(overflow_value) = trackers.get(STREAM_OVERFLOW_ATTRIBUTES.as_slice()) {
            O::update_tracker(&**overflow_value, measurement);
        } else {
            let new_tracker = T::new_atomic_tracker();
            O::update_tracker(&new_tracker, measurement);
            trackers.insert(STREAM_OVERFLOW_ATTRIBUTES.clone(), Arc::new(new_tracker));
            global::handle_error(MetricsError::Other("Warning: Maximum data points for metric stream exceeded. Entry added to overflow. Subsequent overflows to same metric until next collect will not be logged.".into()));
        }
    }
}

/// Summarizes a set of measurements made as their arithmetic sum.
pub(crate) struct Sum<T: Number<T>> {
    value_map: ValueMap<T, Increment>,
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
        self.value_map.measure(measurement, attrs)
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
        for (attrs, tracker) in trackers.iter() {
            s_data.data_points.push(DataPoint {
                attributes: attrs.clone(),
                start_time: Some(prev_start),
                time: Some(t),
                value: tracker.get_value(),
                exemplars: vec![],
            });
        }

        (
            s_data.data_points.len(),
            new_agg.map(|a| Box::new(a) as Box<_>),
        )
    }
}

/// Summarizes a set of pre-computed sums as their arithmetic sum.
pub(crate) struct PrecomputedSum<T: Number<T>> {
    value_map: ValueMap<T, Assign>,
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
        self.value_map.measure(measurement, attrs)
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

        for (attrs, tracker) in trackers.drain() {
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
        for (attrs, tracker) in trackers.iter() {
            s_data.data_points.push(DataPoint {
                attributes: attrs.clone(),
                start_time: Some(prev_start),
                time: Some(t),
                value: tracker.get_value(),
                exemplars: vec![],
            });
        }

        (
            s_data.data_points.len(),
            new_agg.map(|a| Box::new(a) as Box<_>),
        )
    }
}
