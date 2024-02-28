use std::sync::atomic::{AtomicBool, Ordering};
use std::{
    sync::{Arc, Mutex},
    time::SystemTime,
};

use crate::attributes::AttributeSet;
use crate::metrics::data::{self, Aggregation, DataPoint, Temporality};
use opentelemetry::{global, metrics::MetricsError};
use std::hash::{Hash, Hasher};

#[cfg(feature = "use_hashbrown")]
use ahash::AHasher;
#[cfg(feature = "use_hashbrown")]
use hashbrown::{hash_map::Entry, HashMap};

#[cfg(not(feature = "use_hashbrown"))]
use std::collections::{
    hash_map::{DefaultHasher, Entry},
    HashMap,
};

use super::{
    aggregate::{is_under_cardinality_limit, STREAM_OVERFLOW_ATTRIBUTE_SET},
    AtomicTracker, Number,
};

const BUCKET_COUNT: usize = 256;
type BucketValue<T> = Mutex<Option<HashMap<AttributeSet, T>>>;
type Buckets<T> = Arc<[BucketValue<T>; BUCKET_COUNT]>;
/// The storage for sums.
struct ValueMap<T: Number<T>> {
    buckets: Buckets<T>,
    has_no_value_attribute_value: AtomicBool,
    no_attribute_value: T::AtomicTracker,
}

impl<T: Number<T>> Default for ValueMap<T> {
    fn default() -> Self {
        ValueMap::new()
    }
}

impl<T: Number<T>> ValueMap<T> {
    fn new() -> Self {
        let buckets = std::iter::repeat_with(|| Mutex::new(None))
            .take(BUCKET_COUNT)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        ValueMap {
            buckets: Arc::new(buckets),
            has_no_value_attribute_value: AtomicBool::new(false),
            no_attribute_value: T::new_atomic_tracker(),
        }
    }

    // Hash function to determine the bucket
    fn hash_to_bucket(key: &AttributeSet) -> u8 {
        #[cfg(not(feature = "use_hashbrown"))]
        let mut hasher = DefaultHasher::new();
        #[cfg(feature = "use_hashbrown")]
        let mut hasher = AHasher::default();

        key.hash(&mut hasher);
        // Use the 8 least significant bits directly, avoiding the modulus operation.
        hasher.finish() as u8
    }

    // Calculate the total length of data points across all buckets.
    fn total_data_points_count(&self) -> usize {
        self.buckets
            .iter()
            .map(|bucket_mutex| {
                let locked_bucket = bucket_mutex.lock().unwrap();
                locked_bucket.as_ref().map_or(0, |bucket| bucket.len())
            })
            .sum::<usize>()
    }
}

impl<T: Number<T>> ValueMap<T> {
    fn measure(&self, measurement: T, attrs: AttributeSet) {
        if attrs.is_empty() {
            self.no_attribute_value.add(measurement);
            self.has_no_value_attribute_value
                .store(true, Ordering::Release);
        } else {
            let bucket_index = Self::hash_to_bucket(&attrs) as usize; // Ensure index is usize for array indexing
            let bucket_mutex = &self.buckets[bucket_index];
            let mut bucket_guard = bucket_mutex.lock().unwrap();

            if bucket_guard.is_none() {
                *bucket_guard = Some(HashMap::new()); // Initialize the bucket if it's None
            }

            if let Some(ref mut values) = *bucket_guard {
                let size = values.len();
                match values.entry(attrs) {
                    Entry::Occupied(mut occupied_entry) => {
                        let sum = occupied_entry.get_mut();
                        *sum += measurement;
                    }
                    Entry::Vacant(vacant_entry) => {
                        if is_under_cardinality_limit(size) {
                            vacant_entry.insert(measurement);
                        } else {
                            // TBD - Update total_count ??
                            values
                                .entry(STREAM_OVERFLOW_ATTRIBUTE_SET.clone())
                                .and_modify(|val| *val += measurement)
                                .or_insert(measurement);
                            global::handle_error(MetricsError::Other("Warning: Maximum data points for metric stream exceeded. Entry added to overflow.".into()));
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
            value_map: ValueMap::new(),
            monotonic,
            start: Mutex::new(SystemTime::now()),
        }
    }

    pub(crate) fn measure(&self, measurement: T, attrs: AttributeSet) {
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

        let total_len: usize = self.value_map.total_data_points_count() + 1;
        if total_len > s_data.data_points.capacity() {
            let additional_space_needed = total_len - s_data.data_points.capacity();
            s_data.data_points.reserve_exact(additional_space_needed);
        }

        let prev_start = self.start.lock().map(|start| *start).unwrap_or(t);
        if self
            .value_map
            .has_no_value_attribute_value
            .swap(false, Ordering::AcqRel)
        {
            s_data.data_points.push(DataPoint {
                attributes: AttributeSet::default(),
                start_time: Some(prev_start),
                time: Some(t),
                value: self.value_map.no_attribute_value.get_and_reset_value(),
                exemplars: vec![],
            });
        }

        for bucket_mutex in self.value_map.buckets.iter() {
            if let Some(ref mut locked_bucket) = *bucket_mutex.lock().unwrap() {
                for (attrs, value) in locked_bucket.drain() {
                    s_data.data_points.push(DataPoint {
                        attributes: attrs,
                        start_time: Some(*self.start.lock().unwrap()),
                        time: Some(t),
                        value,
                        exemplars: vec![],
                    });
                }
                // The bucket is automatically cleared by the .drain() method
            }
        }

        // The delta collection cycle resets.
        if let Ok(mut start) = self.start.lock() {
            *start = t;
        }

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

        let total_len: usize = self.value_map.total_data_points_count() + 1;
        if total_len > s_data.data_points.capacity() {
            let additional_space_needed = total_len - s_data.data_points.capacity();
            s_data.data_points.reserve_exact(additional_space_needed);
        }

        let prev_start = self.start.lock().map(|start| *start).unwrap_or(t);

        if self
            .value_map
            .has_no_value_attribute_value
            .load(Ordering::Acquire)
        {
            s_data.data_points.push(DataPoint {
                attributes: AttributeSet::default(),
                start_time: Some(prev_start),
                time: Some(t),
                value: self.value_map.no_attribute_value.get_value(),
                exemplars: vec![],
            });
        }

        // TODO: This will use an unbounded amount of memory if there
        // are unbounded number of attribute sets being aggregated. Attribute
        // sets that become "stale" need to be forgotten so this will not
        // overload the system.
        for bucket_mutex in self.value_map.buckets.iter() {
            if let Some(ref locked_bucket) = *bucket_mutex.lock().unwrap() {
                for (attrs, value) in locked_bucket.iter() {
                    s_data.data_points.push(DataPoint {
                        attributes: attrs.clone(),
                        start_time: Some(*self.start.lock().unwrap()), // Consider last reset time
                        time: Some(t),
                        value: *value,
                        exemplars: vec![],
                    });
                }
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
    value_map: ValueMap<T>,
    monotonic: bool,
    start: Mutex<SystemTime>,
    reported: Mutex<HashMap<AttributeSet, T>>,
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

    pub(crate) fn measure(&self, measurement: T, attrs: AttributeSet) {
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

        let total_len: usize = self.value_map.total_data_points_count() + 1;
        if total_len > s_data.data_points.capacity() {
            let additional_space_needed = total_len - s_data.data_points.capacity();
            s_data.data_points.reserve_exact(additional_space_needed);
        }

        let mut new_reported = HashMap::with_capacity(total_len);
        let mut reported = match self.reported.lock() {
            Ok(r) => r,
            Err(_) => return (0, None),
        };

        if self
            .value_map
            .has_no_value_attribute_value
            .swap(false, Ordering::AcqRel)
        {
            s_data.data_points.push(DataPoint {
                attributes: AttributeSet::default(),
                start_time: Some(prev_start),
                time: Some(t),
                value: self.value_map.no_attribute_value.get_and_reset_value(),
                exemplars: vec![],
            });
        }

        for bucket_mutex in self.value_map.buckets.iter() {
            if let Some(ref mut locked_bucket) = *bucket_mutex.lock().unwrap() {
                let default = T::default();
                for (attrs, value) in locked_bucket.drain() {
                    let delta = value - *reported.get(&attrs).unwrap_or(&default);
                    if delta != default {
                        new_reported.insert(attrs.clone(), value);
                    }
                    s_data.data_points.push(DataPoint {
                        attributes: attrs.clone(),
                        start_time: Some(prev_start),
                        time: Some(t),
                        value: delta,
                        exemplars: vec![],
                    });
                }
            }
        }

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

        let total_len: usize = self.value_map.total_data_points_count() + 1;
        if total_len > s_data.data_points.capacity() {
            let additional_space_needed = total_len - s_data.data_points.capacity();
            s_data.data_points.reserve_exact(additional_space_needed);
        }

        let mut new_reported = HashMap::with_capacity(total_len);
        let mut reported = match self.reported.lock() {
            Ok(r) => r,
            Err(_) => return (0, None),
        };

        if self
            .value_map
            .has_no_value_attribute_value
            .load(Ordering::Acquire)
        {
            s_data.data_points.push(DataPoint {
                attributes: AttributeSet::default(),
                start_time: Some(prev_start),
                time: Some(t),
                value: self.value_map.no_attribute_value.get_value(),
                exemplars: vec![],
            });
        }

        let default = T::default();
        for bucket_mutex in self.value_map.buckets.iter() {
            if let Some(ref locked_bucket) = *bucket_mutex.lock().unwrap() {
                for (attrs, value) in locked_bucket.iter() {
                    let delta = *value - *reported.get(attrs).unwrap_or(&default);
                    if delta != default {
                        new_reported.insert(attrs.clone(), *value);
                    }
                    s_data.data_points.push(DataPoint {
                        attributes: attrs.clone(),
                        start_time: Some(prev_start),
                        time: Some(t),
                        value: *value, // For cumulative, we use the value directly without calculating delta
                        exemplars: vec![],
                    });
                }
            }
        }

        *reported = new_reported;
        drop(reported); // drop before values guard is dropped

        (
            s_data.data_points.len(),
            new_agg.map(|a| Box::new(a) as Box<_>),
        )
    }
}
