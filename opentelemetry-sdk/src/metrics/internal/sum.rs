use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
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
use hashbrown::HashMap;

#[cfg(not(feature = "use_hashbrown"))]
use std::collections::{hash_map::DefaultHasher, HashMap};

use super::{
    aggregate::{is_under_cardinality_limit, STREAM_OVERFLOW_ATTRIBUTE_SET},
    AtomicTracker, Number,
};

const BUCKET_COUNT: usize = 256;
const OVERFLOW_BUCKET_INDEX: usize = BUCKET_COUNT - 1; // Use the last bucket as overflow bucket
type BucketValue<T> = Mutex<Option<HashMap<AttributeSet, T>>>;
type Buckets<T> = Arc<[BucketValue<T>; BUCKET_COUNT]>;
/// The storage for sums.
struct ValueMap<T: Number<T>> {
    buckets: Buckets<T>,
    has_no_value_attribute_value: AtomicBool,
    no_attribute_value: T::AtomicTracker,
    total_unique_entries: AtomicUsize,
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
            .unwrap(); // this will never fail as Vec length is fixed

        ValueMap {
            buckets: Arc::new(buckets),
            has_no_value_attribute_value: AtomicBool::new(false),
            no_attribute_value: T::new_atomic_tracker(),
            total_unique_entries: AtomicUsize::new(0),
        }
    }

    // Hash function to determine the bucket
    fn hash_to_bucket(key: &AttributeSet) -> usize {
        #[cfg(not(feature = "use_hashbrown"))]
        let mut hasher = DefaultHasher::new();
        #[cfg(feature = "use_hashbrown")]
        let mut hasher = AHasher::default();

        key.hash(&mut hasher);
        // Use the 8 least significant bits directly, avoiding the modulus operation.
        hasher.finish() as u8 as usize
    }

    fn try_increment(&self) -> bool {
        loop {
            let current = self.total_unique_entries.load(Ordering::Acquire);
            if is_under_cardinality_limit(current) {
                // Attempt to increment atomically if old value is still current, else retry
                match self.total_unique_entries.compare_exchange(
                    current,
                    current + 1,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(_) => return true, // Increment successful
                    Err(_) => continue, // Failed to increment due to concurrent modification, retry
                }
            } else {
                return false; // Limit reached, do not increment
            }
        }
    }
}

impl<T: Number<T>> ValueMap<T> {
    fn measure(&self, measurement: T, attrs: AttributeSet) {
        if attrs.is_empty() {
            self.no_attribute_value.add(measurement);
            self.has_no_value_attribute_value
                .store(true, Ordering::Release);
            return;
        }

        // Determine the bucket index for the attributes
        let bucket_index = Self::hash_to_bucket(&attrs);
        let mut bucket_guard = self.buckets[bucket_index].lock().unwrap();
        if let Some(bucket) = &mut *bucket_guard {
            // if the attribute is present in bucket, lets update the value and return
            // (we need to be fast here, as this is most common use-case)
            if let Some(value) = bucket.get_mut(&attrs) {
                *value += measurement;
                return;
            }
        }

        // if the attribute is not present in bucket, lets unlock the bucket
        drop(bucket_guard);
        // Attempt to first increment the total unique entries if under limit.
        let under_limit = self.try_increment();
        // Determine the bucket index for the attributes
        let (bucket_index, attrs) = if under_limit {
            (bucket_index, attrs) // the index remains same
        } else {
            // TBD - Should we log, as this can flood the logs ?
            (OVERFLOW_BUCKET_INDEX, STREAM_OVERFLOW_ATTRIBUTE_SET.clone())
        };

        // Lock and update the relevant bucket
        let mut final_bucket_guard = self.buckets[bucket_index].lock().unwrap();
        let bucket = final_bucket_guard.get_or_insert_with(HashMap::default);

        bucket
            .entry(attrs)
            .and_modify(|e| *e += measurement)
            .or_insert(measurement);
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

        let total_len: usize = self.value_map.total_unique_entries.load(Ordering::Relaxed) + 1;
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
            match bucket_mutex.lock() {
                Ok(mut locked_bucket) => {
                    if let Some(ref mut bucket) = *locked_bucket {
                        for (attrs, value) in bucket.drain() {
                            // Correctly handle lock acquisition on self.start
                            let start_time = self.start.lock().map_or_else(
                                |_| SystemTime::now(), // In case of an error, use SystemTime::now()
                                |guard| *guard, // In case of success, dereference the guard to get the SystemTime
                            );

                            s_data.data_points.push(DataPoint {
                                attributes: attrs,
                                start_time: Some(start_time),
                                time: Some(t),
                                value,
                                exemplars: vec![],
                            });
                            self.value_map
                                .total_unique_entries
                                .fetch_sub(1, Ordering::Relaxed);
                        }
                    }
                }
                Err(e) => {
                    global::handle_error(MetricsError::Other(format!(
                        "Failed to acquire lock on bucket due to: {:?}",
                        e
                    )));
                }
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

        let total_len: usize = self.value_map.total_unique_entries.load(Ordering::Relaxed) + 1;
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
            // Handle potential lock failure gracefully
            if let Ok(locked_bucket) = bucket_mutex.lock() {
                if let Some(locked_bucket) = &*locked_bucket {
                    for (attrs, value) in locked_bucket.iter() {
                        // Handle potential lock failure on self.start and use current time as fallback
                        let start_time = self.start.lock().map_or_else(
                            |_| SystemTime::now(), // Use SystemTime::now() as fallback on error
                            |guard| *guard, // Dereference the guard to get the SystemTime on success
                        );

                        s_data.data_points.push(DataPoint {
                            attributes: attrs.clone(),
                            start_time: Some(start_time),
                            time: Some(t),
                            value: *value,
                            exemplars: vec![],
                        });
                    }
                }
            } else {
                global::handle_error(MetricsError::Other(
                    "Failed to acquire lock on a bucket".into(),
                ));
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

        let total_len: usize = self.value_map.total_unique_entries.load(Ordering::Relaxed) + 1;
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
            match bucket_mutex.lock() {
                Ok(mut locked_bucket) => {
                    if let Some(locked_bucket) = &mut *locked_bucket {
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
                            self.value_map
                                .total_unique_entries
                                .fetch_sub(1, Ordering::Relaxed);
                        }
                    }
                }
                Err(e) => {
                    // Log or handle the lock acquisition error if necessary
                    global::handle_error(MetricsError::Other(format!(
                        "Failed to acquire lock on bucket due to: {:?}",
                        e
                    )));
                    // Continue to the next bucket if the lock cannot be acquired
                    continue;
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

        let total_len: usize = self.value_map.total_unique_entries.load(Ordering::Relaxed) + 1;
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
            // Safely attempt to acquire the lock, handling any potential error.
            let locked_bucket = match bucket_mutex.lock() {
                Ok(bucket) => bucket,
                Err(e) => {
                    // Log the error or handle it as needed.
                    global::handle_error(MetricsError::Other(format!(
                        "Failed to acquire lock on bucket due to: {:?}",
                        e
                    )));
                    continue; // Skip to the next bucket if the lock cannot be acquired.
                }
            };

            // Proceed only if the bucket is not empty.
            if let Some(locked_bucket) = &*locked_bucket {
                for (attrs, value) in locked_bucket.iter() {
                    let delta = *value - *reported.get(attrs).unwrap_or(&default);
                    if delta != default {
                        new_reported.insert(attrs.clone(), *value);
                    }

                    s_data.data_points.push(DataPoint {
                        attributes: attrs.clone(),
                        start_time: Some(prev_start),
                        time: Some(t),
                        value: *value, // For cumulative, directly use the value without calculating the delta.
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
