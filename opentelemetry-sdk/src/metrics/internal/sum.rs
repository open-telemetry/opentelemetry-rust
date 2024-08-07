use std::collections::HashSet;
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

/// The storage for sums.
struct ValueMap<T: Number<T>> {
    values: RwLock<HashMap<Vec<KeyValue>, Arc<T::AtomicTracker>>>,
    has_no_value_attribute_value: AtomicBool,
    no_attribute_value: T::AtomicTracker,
    count: AtomicUsize,
    assign_only: bool, // if true, only assign incoming value, instead of adding to existing value
}

impl<T: Number<T>> Default for ValueMap<T> {
    fn default() -> Self {
        ValueMap::new(false)
    }
}

impl<T: Number<T>> ValueMap<T> {
    fn new(assign_only: bool) -> Self {
        ValueMap {
            values: RwLock::new(HashMap::new()),
            has_no_value_attribute_value: AtomicBool::new(false),
            no_attribute_value: T::new_atomic_tracker(),
            count: AtomicUsize::new(0),
            assign_only: assign_only,
        }
    }
}

impl<T: Number<T>> ValueMap<T> {
    fn measure(&self, measurement: T, attrs: &[KeyValue]) {
        if attrs.is_empty() {
            if self.assign_only {
                self.no_attribute_value.store(measurement);
            } else {
                self.no_attribute_value.add(measurement);
            }
            self.has_no_value_attribute_value
                .store(true, Ordering::Release);
        } else if let Ok(values) = self.values.read() {
            // Try incoming order first
            if let Some(value_to_update) = values.get(attrs) {
                if self.assign_only {
                    value_to_update.store(measurement);
                } else {
                    value_to_update.add(measurement);
                }
            } else {
                // Then try sorted order.
                let sorted_attrs = AttributeSet::from(attrs).into_vec();
                if let Some(value_to_update) = values.get(sorted_attrs.as_slice()) {
                    if self.assign_only {
                        value_to_update.store(measurement);
                    } else {
                        value_to_update.add(measurement);
                    }
                } else {
                    // Give up the lock, before acquiring write lock.
                    drop(values);
                    if let Ok(mut values) = self.values.write() {
                        // Recheck both incoming and sorted after acquiring
                        // write lock, in case another thread has added the
                        // value.
                        if let Some(value_to_update) = values.get(attrs) {
                            if self.assign_only {
                                value_to_update.store(measurement);
                            } else {
                                value_to_update.add(measurement);
                            }
                        } else if let Some(value_to_update) = values.get(sorted_attrs.as_slice()) {
                            if self.assign_only {
                                value_to_update.store(measurement);
                            } else {
                                value_to_update.add(measurement);
                            }
                        } else if is_under_cardinality_limit(self.count.load(Ordering::SeqCst)) {
                            let new_value = T::new_atomic_tracker();
                            if self.assign_only {
                                new_value.store(measurement);
                            } else {
                                new_value.add(measurement);
                            }
                            let new_value = Arc::new(new_value);

                            // Insert original order
                            values.insert(attrs.to_vec(), new_value.clone());

                            // Insert sorted order
                            values.insert(sorted_attrs, new_value);

                            self.count.fetch_add(1, Ordering::SeqCst);
                        } else if let Some(overflow_value) =
                            values.get(STREAM_OVERFLOW_ATTRIBUTES.as_slice())
                        {
                            if self.assign_only {
                                overflow_value.store(measurement);
                            } else {
                                overflow_value.add(measurement);
                            }
                        } else {
                            let new_value = T::new_atomic_tracker();
                            if self.assign_only {
                                new_value.store(measurement);
                            } else {
                                new_value.add(measurement);
                            }
                            values.insert(STREAM_OVERFLOW_ATTRIBUTES.clone(), Arc::new(new_value));
                            global::handle_error(MetricsError::Other("Warning: Maximum data points for metric stream exceeded. Entry added to overflow. Subsequent overflows to same metric until next collect will not be logged.".into()));
                        }
                    }
                }
            }
        }
    }
}

/// Summarizes a set of measurements made as their arithmetic sum.
pub(crate) struct Sum<T: Number<T>> {
    value_map: ValueMap<T>,
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
            value_map: ValueMap::new(false),
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
            .has_no_value_attribute_value
            .swap(false, Ordering::AcqRel)
        {
            s_data.data_points.push(DataPoint {
                attributes: vec![],
                start_time: Some(prev_start),
                time: Some(t),
                value: self.value_map.no_attribute_value.get_and_reset_value(),
                exemplars: vec![],
            });
        }

        let mut values = match self.value_map.values.write() {
            Ok(v) => v,
            Err(_) => return (0, None),
        };

        let mut seen = HashSet::new();
        for (attrs, value) in values.drain() {
            if seen.insert(Arc::as_ptr(&value)) {
                s_data.data_points.push(DataPoint {
                    attributes: attrs.clone(),
                    start_time: Some(prev_start),
                    time: Some(t),
                    value: value.get_value(),
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
            .has_no_value_attribute_value
            .load(Ordering::Acquire)
        {
            s_data.data_points.push(DataPoint {
                attributes: vec![],
                start_time: Some(prev_start),
                time: Some(t),
                value: self.value_map.no_attribute_value.get_value(),
                exemplars: vec![],
            });
        }

        let values = match self.value_map.values.write() {
            Ok(v) => v,
            Err(_) => return (0, None),
        };

        // TODO: This will use an unbounded amount of memory if there
        // are unbounded number of attribute sets being aggregated. Attribute
        // sets that become "stale" need to be forgotten so this will not
        // overload the system.
        for (attrs, value) in values.iter() {
            s_data.data_points.push(DataPoint {
                attributes: attrs.clone(),
                start_time: Some(prev_start),
                time: Some(t),
                value: value.get_value(),
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
    value_map: ValueMap<T>,
    monotonic: bool,
    start: Mutex<SystemTime>,
    reported: Mutex<HashMap<Vec<KeyValue>, T>>,
}

impl<T: Number<T>> PrecomputedSum<T> {
    pub(crate) fn new(monotonic: bool) -> Self {
        PrecomputedSum {
            value_map: ValueMap::new(true),
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
            .has_no_value_attribute_value
            .swap(false, Ordering::AcqRel)
        {
            let default = T::default();
            let delta = self.value_map.no_attribute_value.get_value()
                - *reported.get(&vec![]).unwrap_or(&default);
            new_reported.insert(vec![], self.value_map.no_attribute_value.get_value());

            s_data.data_points.push(DataPoint {
                attributes: vec![],
                start_time: Some(prev_start),
                time: Some(t),
                value: delta,
                exemplars: vec![],
            });
        }

        let mut values = match self.value_map.values.write() {
            Ok(v) => v,
            Err(_) => return (0, None),
        };

        let default = T::default();
        for (attrs, value) in values.drain() {
            let delta = value.get_value() - *reported.get(&attrs).unwrap_or(&default);
            new_reported.insert(attrs.clone(), value.get_value());
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
            .has_no_value_attribute_value
            .load(Ordering::Acquire)
        {
            s_data.data_points.push(DataPoint {
                attributes: vec![],
                start_time: Some(prev_start),
                time: Some(t),
                value: self.value_map.no_attribute_value.get_value(),
                exemplars: vec![],
            });
        }

        let values = match self.value_map.values.write() {
            Ok(v) => v,
            Err(_) => return (0, None),
        };
        for (attrs, value) in values.iter() {
            s_data.data_points.push(DataPoint {
                attributes: attrs.clone(),
                start_time: Some(prev_start),
                time: Some(t),
                value: value.get_value(),
                exemplars: vec![],
            });
        }

        (
            s_data.data_points.len(),
            new_agg.map(|a| Box::new(a) as Box<_>),
        )
    }
}
