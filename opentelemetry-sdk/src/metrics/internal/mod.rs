mod aggregate;
mod exponential_histogram;
mod histogram;
mod last_value;
mod sum;

use core::fmt;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Sub};
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

use aggregate::is_under_cardinality_limit;
pub(crate) use aggregate::{AggregateBuilder, ComputeAggregation, Measure};
pub(crate) use exponential_histogram::{EXPO_MAX_SCALE, EXPO_MIN_SCALE};
use once_cell::sync::Lazy;
use opentelemetry::metrics::MetricsError;
use opentelemetry::{global, KeyValue};

use crate::metrics::AttributeSet;

pub(crate) static STREAM_OVERFLOW_ATTRIBUTES: Lazy<Vec<KeyValue>> =
    Lazy::new(|| vec![KeyValue::new("otel.metric.overflow", "true")]);

/// Abstracts the update operation for a measurement.
pub(crate) trait Operation {
    fn update_tracker<T: Default, AT: AtomicTracker<T>>(tracker: &AT, value: T, index: usize);
}

struct Increment;

impl Operation for Increment {
    fn update_tracker<T: Default, AT: AtomicTracker<T>>(tracker: &AT, value: T, _: usize) {
        tracker.add(value);
    }
}

struct Assign;

impl Operation for Assign {
    fn update_tracker<T: Default, AT: AtomicTracker<T>>(tracker: &AT, value: T, _: usize) {
        tracker.store(value);
    }
}

/// The storage for sums.
///
/// This structure is parametrized by an `Operation` that indicates how
/// updates to the underlying value trackers should be performed.
pub(crate) struct ValueMap<AU: AtomicallyUpdate<T>, T: Number<T>, O> {
    /// Trackers store the values associated with different attribute sets.
    trackers: RwLock<HashMap<Vec<KeyValue>, Arc<AU::AtomicTracker>>>,
    /// Number of different attribute set stored in the `trackers` map.
    count: AtomicUsize,
    /// Indicates whether a value with no attributes has been stored.
    has_no_attribute_value: AtomicBool,
    /// Tracker for values with no attributes attached.
    no_attribute_tracker: AU::AtomicTracker,
    /// Buckets Count is only used by Histogram.
    buckets_count: Option<usize>,
    phantom: PhantomData<O>,
}

impl<AU: AtomicallyUpdate<T>, T: Number<T>, O> Default for ValueMap<AU, T, O> {
    fn default() -> Self {
        ValueMap::new()
    }
}

impl<AU: AtomicallyUpdate<T>, T: Number<T>, O> ValueMap<AU, T, O> {
    fn new() -> Self {
        ValueMap {
            trackers: RwLock::new(HashMap::new()),
            has_no_attribute_value: AtomicBool::new(false),
            no_attribute_tracker: AU::new_atomic_tracker(None),
            count: AtomicUsize::new(0),
            buckets_count: None,
            phantom: PhantomData,
        }
    }

    fn new_with_buckets_count(buckets_count: usize) -> Self {
        ValueMap {
            trackers: RwLock::new(HashMap::new()),
            has_no_attribute_value: AtomicBool::new(false),
            no_attribute_tracker: AU::new_atomic_tracker(Some(buckets_count)),
            count: AtomicUsize::new(0),
            buckets_count: Some(buckets_count),
            phantom: PhantomData,
        }
    }
}

impl<AU: AtomicallyUpdate<T>, T: Number<T>, O: Operation> ValueMap<AU, T, O> {
    fn measure(&self, measurement: T, attributes: &[KeyValue], index: usize) {
        if attributes.is_empty() {
            O::update_tracker(&self.no_attribute_tracker, measurement, index);
            self.has_no_attribute_value.store(true, Ordering::Release);
            return;
        }

        let Ok(trackers) = self.trackers.read() else {
            return;
        };

        // Try to retrieve and update the tracker with the attributes in the provided order first
        if let Some(tracker) = trackers.get(attributes) {
            O::update_tracker(&**tracker, measurement, index);
            return;
        }

        // Try to retrieve and update the tracker with the attributes sorted.
        let sorted_attrs = AttributeSet::from(attributes).into_vec();
        if let Some(tracker) = trackers.get(sorted_attrs.as_slice()) {
            O::update_tracker(&**tracker, measurement, index);
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
            O::update_tracker(&**tracker, measurement, index);
        } else if let Some(tracker) = trackers.get(sorted_attrs.as_slice()) {
            O::update_tracker(&**tracker, measurement, index);
        } else if is_under_cardinality_limit(self.count.load(Ordering::SeqCst)) {
            let new_tracker = Arc::new(AU::new_atomic_tracker(self.buckets_count));
            O::update_tracker(&*new_tracker, measurement, index);

            // Insert tracker with the attributes in the provided and sorted orders
            trackers.insert(attributes.to_vec(), new_tracker.clone());
            trackers.insert(sorted_attrs, new_tracker);

            self.count.fetch_add(1, Ordering::SeqCst);
        } else if let Some(overflow_value) = trackers.get(STREAM_OVERFLOW_ATTRIBUTES.as_slice()) {
            O::update_tracker(&**overflow_value, measurement, index);
        } else {
            let new_tracker = AU::new_atomic_tracker(self.buckets_count);
            O::update_tracker(&new_tracker, measurement, index);
            trackers.insert(STREAM_OVERFLOW_ATTRIBUTES.clone(), Arc::new(new_tracker));
            global::handle_error(MetricsError::Other("Warning: Maximum data points for metric stream exceeded. Entry added to overflow. Subsequent overflows to same metric until next collect will not be logged.".into()));
        }
    }
}

/// Marks a type that can have a value added and retrieved atomically. Required since
/// different types have different backing atomic mechanisms
pub(crate) trait AtomicTracker<T: Default>: Sync + Send + 'static {
    fn store(&self, _value: T) {}
    fn add(&self, _value: T) {}
    fn get_value(&self) -> T {
        T::default()
    }
    fn get_and_reset_value(&self) -> T {
        T::default()
    }
    fn update_histogram(&self, _index: usize, _value: T) {}
}

/// Marks a type that can have an atomic tracker generated for it
pub(crate) trait AtomicallyUpdate<T: Default> {
    type AtomicTracker: AtomicTracker<T>;
    fn new_atomic_tracker(buckets_count: Option<usize>) -> Self::AtomicTracker;
}

pub(crate) trait Number<T: Default>:
    Add<Output = T>
    + AddAssign
    + Sub<Output = T>
    + PartialOrd
    + fmt::Debug
    + Clone
    + Copy
    + PartialEq
    + Default
    + Send
    + Sync
    + 'static
    + AtomicallyUpdate<T>
{
    fn min() -> Self;
    fn max() -> Self;

    fn into_float(self) -> f64;
}

impl Number<i64> for i64 {
    fn min() -> Self {
        i64::MIN
    }

    fn max() -> Self {
        i64::MAX
    }

    fn into_float(self) -> f64 {
        // May have precision loss at high values
        self as f64
    }
}
impl Number<u64> for u64 {
    fn min() -> Self {
        u64::MIN
    }

    fn max() -> Self {
        u64::MAX
    }

    fn into_float(self) -> f64 {
        // May have precision loss at high values
        self as f64
    }
}
impl Number<f64> for f64 {
    fn min() -> Self {
        f64::MIN
    }

    fn max() -> Self {
        f64::MAX
    }

    fn into_float(self) -> f64 {
        self
    }
}

impl AtomicTracker<u64> for AtomicU64 {
    fn store(&self, value: u64) {
        self.store(value, Ordering::Relaxed);
    }

    fn add(&self, value: u64) {
        self.fetch_add(value, Ordering::Relaxed);
    }

    fn get_value(&self) -> u64 {
        self.load(Ordering::Relaxed)
    }

    fn get_and_reset_value(&self) -> u64 {
        self.swap(0, Ordering::Relaxed)
    }
}

impl AtomicallyUpdate<u64> for u64 {
    type AtomicTracker = AtomicU64;

    fn new_atomic_tracker(_: Option<usize>) -> Self::AtomicTracker {
        AtomicU64::new(0)
    }
}

impl AtomicTracker<i64> for AtomicI64 {
    fn store(&self, value: i64) {
        self.store(value, Ordering::Relaxed);
    }

    fn add(&self, value: i64) {
        self.fetch_add(value, Ordering::Relaxed);
    }

    fn get_value(&self) -> i64 {
        self.load(Ordering::Relaxed)
    }

    fn get_and_reset_value(&self) -> i64 {
        self.swap(0, Ordering::Relaxed)
    }
}

impl AtomicallyUpdate<i64> for i64 {
    type AtomicTracker = AtomicI64;

    fn new_atomic_tracker(_: Option<usize>) -> Self::AtomicTracker {
        AtomicI64::new(0)
    }
}

pub(crate) struct F64AtomicTracker {
    inner: AtomicU64, // Floating points don't have true atomics, so we need to use the their binary representation to perform atomic operations
}

impl F64AtomicTracker {
    fn new() -> Self {
        let zero_as_u64 = 0.0_f64.to_bits();
        F64AtomicTracker {
            inner: AtomicU64::new(zero_as_u64),
        }
    }
}

impl AtomicTracker<f64> for F64AtomicTracker {
    fn store(&self, value: f64) {
        let value_as_u64 = value.to_bits();
        self.inner.store(value_as_u64, Ordering::Relaxed);
    }

    fn add(&self, value: f64) {
        let mut current_value_as_u64 = self.inner.load(Ordering::Relaxed);

        loop {
            let current_value = f64::from_bits(current_value_as_u64);
            let new_value = current_value + value;
            let new_value_as_u64 = new_value.to_bits();
            match self.inner.compare_exchange(
                current_value_as_u64,
                new_value_as_u64,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                // Succeeded in updating the value
                Ok(_) => return,

                // Some other thread changed the value before this thread could update it.
                // Read the latest value again and try to swap it with the recomputed `new_value_as_u64`.
                Err(v) => current_value_as_u64 = v,
            }
        }
    }

    fn get_value(&self) -> f64 {
        let value_as_u64 = self.inner.load(Ordering::Relaxed);
        f64::from_bits(value_as_u64)
    }

    fn get_and_reset_value(&self) -> f64 {
        let zero_as_u64 = 0.0_f64.to_bits();
        let value = self.inner.swap(zero_as_u64, Ordering::Relaxed);
        f64::from_bits(value)
    }
}

impl AtomicallyUpdate<f64> for f64 {
    type AtomicTracker = F64AtomicTracker;

    fn new_atomic_tracker(_: Option<usize>) -> Self::AtomicTracker {
        F64AtomicTracker::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_store_u64_atomic_value() {
        let atomic = u64::new_atomic_tracker(None);
        let atomic_tracker = &atomic as &dyn AtomicTracker<u64>;

        let value = atomic.get_value();
        assert_eq!(value, 0);

        atomic_tracker.store(25);
        let value = atomic.get_value();
        assert_eq!(value, 25);
    }

    #[test]
    fn can_add_and_get_u64_atomic_value() {
        let atomic = u64::new_atomic_tracker(None);
        atomic.add(15);
        atomic.add(10);

        let value = atomic.get_value();
        assert_eq!(value, 25);
    }

    #[test]
    fn can_reset_u64_atomic_value() {
        let atomic = u64::new_atomic_tracker(None);
        atomic.add(15);

        let value = atomic.get_and_reset_value();
        let value2 = atomic.get_value();

        assert_eq!(value, 15, "Incorrect first value");
        assert_eq!(value2, 0, "Incorrect second value");
    }

    #[test]
    fn can_store_i64_atomic_value() {
        let atomic = i64::new_atomic_tracker(None);
        let atomic_tracker = &atomic as &dyn AtomicTracker<i64>;

        let value = atomic.get_value();
        assert_eq!(value, 0);

        atomic_tracker.store(-25);
        let value = atomic.get_value();
        assert_eq!(value, -25);

        atomic_tracker.store(25);
        let value = atomic.get_value();
        assert_eq!(value, 25);
    }

    #[test]
    fn can_add_and_get_i64_atomic_value() {
        let atomic = i64::new_atomic_tracker(None);
        atomic.add(15);
        atomic.add(-10);

        let value = atomic.get_value();
        assert_eq!(value, 5);
    }

    #[test]
    fn can_reset_i64_atomic_value() {
        let atomic = i64::new_atomic_tracker(None);
        atomic.add(15);

        let value = atomic.get_and_reset_value();
        let value2 = atomic.get_value();

        assert_eq!(value, 15, "Incorrect first value");
        assert_eq!(value2, 0, "Incorrect second value");
    }

    #[test]
    fn can_store_f64_atomic_value() {
        let atomic = f64::new_atomic_tracker(None);
        let atomic_tracker = &atomic as &dyn AtomicTracker<f64>;

        let value = atomic.get_value();
        assert_eq!(value, 0.0);

        atomic_tracker.store(-15.5);
        let value = atomic.get_value();
        assert!(f64::abs(-15.5 - value) < 0.0001);

        atomic_tracker.store(25.7);
        let value = atomic.get_value();
        assert!(f64::abs(25.7 - value) < 0.0001);
    }

    #[test]
    fn can_add_and_get_f64_atomic_value() {
        let atomic = f64::new_atomic_tracker(None);
        atomic.add(15.3);
        atomic.add(10.4);

        let value = atomic.get_value();

        assert!(f64::abs(25.7 - value) < 0.0001);
    }

    #[test]
    fn can_reset_f64_atomic_value() {
        let atomic = f64::new_atomic_tracker(None);
        atomic.add(15.5);

        let value = atomic.get_and_reset_value();
        let value2 = atomic.get_value();

        assert!(f64::abs(15.5 - value) < 0.0001, "Incorrect first value");
        assert!(f64::abs(0.0 - value2) < 0.0001, "Incorrect second value");
    }
}
