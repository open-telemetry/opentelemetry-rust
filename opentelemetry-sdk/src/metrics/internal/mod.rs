mod aggregate;
mod exponential_histogram;
mod histogram;
mod last_value;
mod precomputed_sum;
mod sum;

use core::fmt;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::mem::swap;
use std::ops::{Add, AddAssign, DerefMut, Sub};
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use aggregate::is_under_cardinality_limit;
pub(crate) use aggregate::{AggregateBuilder, ComputeAggregation, Measure};
pub(crate) use exponential_histogram::{EXPO_MAX_SCALE, EXPO_MIN_SCALE};
use once_cell::sync::Lazy;
use opentelemetry::{otel_warn, KeyValue};

pub(crate) static STREAM_OVERFLOW_ATTRIBUTES: Lazy<Vec<KeyValue>> =
    Lazy::new(|| vec![KeyValue::new("otel.metric.overflow", "true")]);

pub(crate) trait Aggregator {
    /// A static configuration that is needed in order to initialize aggregator.
    /// E.g. bucket_size at creation time .
    type InitConfig;

    /// Some aggregators can do some computations before updating aggregator.
    /// This helps to reduce contention for aggregators because it makes
    /// [`Aggregator::update`] as short as possible.
    type PreComputedValue;

    /// Called everytime a new attribute-set is stored.
    fn create(init: &Self::InitConfig) -> Self;

    /// Called for each measurement.
    fn update(&self, value: Self::PreComputedValue);

    /// Return current value and reset this instance
    fn clone_and_reset(&self, init: &Self::InitConfig) -> Self;
}

struct NoAttribs<A> {
    tracker: A,
    is_set: AtomicBool,
}

/// The storage for sums.
///
/// This structure is parametrized by an `Operation` that indicates how
/// updates to the underlying value trackers should be performed.
pub(crate) struct ValueMap<A>
where
    A: Aggregator,
{
    // for performance reasons, no_attribs tracker
    no_attribs: NoAttribs<A>,
    // for performance reasons, to handle attributes in the provided order
    all_attribs: RwLock<HashMap<Vec<KeyValue>, Arc<A>>>,
    // different order of attribute keys should still map to same tracker instance
    // this helps to achieve that and also enables implementing collection efficiently
    sorted_attribs: Mutex<HashMap<Vec<KeyValue>, Arc<A>>>,
    /// Configuration for an Aggregator
    config: A::InitConfig,
    /// Swap with `sorted_attribs` on every `collect_and_reset`.
    for_collect_after_reset: Mutex<HashMap<Vec<KeyValue>, Arc<A>>>,
}

impl<A> ValueMap<A>
where
    A: Aggregator,
{
    fn new(config: A::InitConfig) -> Self {
        ValueMap {
            no_attribs: NoAttribs {
                tracker: A::create(&config),
                is_set: AtomicBool::new(false),
            },
            all_attribs: RwLock::new(Default::default()),
            sorted_attribs: Mutex::new(Default::default()),
            config,
            for_collect_after_reset: Mutex::new(Default::default()),
        }
    }

    fn measure(&self, value: A::PreComputedValue, attributes: &[KeyValue]) {
        if attributes.is_empty() {
            self.no_attribs.tracker.update(value);
            self.no_attribs.is_set.store(true, Ordering::Release);
            return;
        }

        // Try to retrieve and update the tracker with the attributes in the provided order first
        match self.all_attribs.read() {
            Ok(trackers) => {
                if let Some(tracker) = trackers.get(attributes) {
                    tracker.update(value);
                    return;
                }
            }
            Err(_) => return,
        };

        // Get or create a tracker
        let sorted_attrs = sort_and_dedup(attributes);
        let Ok(mut sorted_trackers) = self.sorted_attribs.lock() else {
            return;
        };

        let sorted_count = sorted_trackers.len();
        let new_tracker = match sorted_trackers.entry(sorted_attrs) {
            Entry::Occupied(occupied_entry) => occupied_entry.get().clone(),
            Entry::Vacant(vacant_entry) => {
                if !is_under_cardinality_limit(sorted_count) {
                    sorted_trackers.entry(STREAM_OVERFLOW_ATTRIBUTES.clone())
                        .or_insert_with(|| {
                            otel_warn!( name: "ValueMap.measure",
                                message = "Maximum data points for metric stream exceeded. Entry added to overflow. Subsequent overflows to same metric until next collect will not be logged."
                            );
                            Arc::new(A::create(&self.config))
                        })
                        .update(value);
                    return;
                }
                let new_tracker = Arc::new(A::create(&self.config));
                vacant_entry.insert(new_tracker).clone()
            }
        };
        drop(sorted_trackers);

        new_tracker.update(value);

        // Insert new tracker, so we could find it next time
        let Ok(mut all_trackers) = self.all_attribs.write() else {
            return;
        };
        all_trackers.insert(attributes.to_vec(), new_tracker);
    }

    /// Iterate through all attribute sets and populate `DataPoints` in readonly mode.
    /// This is used in Cumulative temporality mode, where [`ValueMap`] is not cleared.
    pub(crate) fn collect_readonly<Res, MapFn>(&self, dest: &mut Vec<Res>, mut map_fn: MapFn)
    where
        MapFn: FnMut(Vec<KeyValue>, &A) -> Res,
    {
        let trackers = match self.sorted_attribs.lock() {
            Ok(trackers) => {
                // it's important to release lock as fast as possible,
                // so we don't block insertion of new attribute sets
                trackers.clone()
            }
            Err(_) => return,
        };

        prepare_data(dest, trackers.len());

        if self.no_attribs.is_set.load(Ordering::Acquire) {
            dest.push(map_fn(vec![], &self.no_attribs.tracker));
        }

        for (attrs, tracker) in trackers.into_iter() {
            dest.push(map_fn(attrs, &tracker));
        }
    }

    /// Iterate through all attribute sets, populate `DataPoints` and reset.
    /// This is used in Delta temporality mode, where [`ValueMap`] is reset after collection.
    pub(crate) fn collect_and_reset<Res, MapFn>(&self, dest: &mut Vec<Res>, mut map_fn: MapFn)
    where
        MapFn: FnMut(Vec<KeyValue>, A) -> Res,
    {
        let mut to_collect = self
            .for_collect_after_reset
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        // reset sorted trackers so new attributes set will be written into new hashmap
        match self.sorted_attribs.lock() {
            Ok(mut trackers) => {
                swap(trackers.deref_mut(), to_collect.deref_mut());
            }
            Err(_) => return,
        };
        // reset all trackers, so all attribute sets will start using new hashmap
        match self.all_attribs.write() {
            Ok(mut all_trackers) => all_trackers.clear(),
            Err(_) => return,
        };

        prepare_data(dest, to_collect.len());

        if self.no_attribs.is_set.swap(false, Ordering::AcqRel) {
            dest.push(map_fn(
                vec![],
                self.no_attribs.tracker.clone_and_reset(&self.config),
            ));
        }

        for (attrs, tracker) in to_collect.drain() {
            let tracker = Arc::into_inner(tracker).expect("the only instance");
            dest.push(map_fn(attrs, tracker));
        }
    }
}

/// Clear and allocate exactly required amount of space for all attribute-sets
fn prepare_data<T>(data: &mut Vec<T>, list_len: usize) {
    data.clear();
    let total_len = list_len + 1; // to account for no_attributes case
    if total_len > data.capacity() {
        data.reserve_exact(total_len - data.capacity());
    }
}

fn sort_and_dedup(attributes: &[KeyValue]) -> Vec<KeyValue> {
    // Use newly allocated vec here as incoming attributes are immutable so
    // cannot sort/de-dup in-place. TODO: This allocation can be avoided by
    // leveraging a ThreadLocal vec.
    let mut sorted = attributes.to_vec();
    sorted.sort_unstable_by(|a, b| a.key.cmp(&b.key));
    sorted.dedup_by(|a, b| a.key == b.key);
    sorted
}

/// Marks a type that can have a value added and retrieved atomically. Required since
/// different types have different backing atomic mechanisms
pub(crate) trait AtomicTracker<T>: Sync + Send + 'static {
    fn store(&self, _value: T);
    fn add(&self, _value: T);
    fn get_value(&self) -> T;
    fn get_and_reset_value(&self) -> T;
}

/// Marks a type that can have an atomic tracker generated for it
pub(crate) trait AtomicallyUpdate<T> {
    type AtomicTracker: AtomicTracker<T>;
    fn new_atomic_tracker(init: T) -> Self::AtomicTracker;
}

pub(crate) trait Number:
    Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + PartialOrd
    + fmt::Debug
    + Clone
    + Copy
    + PartialEq
    + Default
    + Send
    + Sync
    + 'static
    + AtomicallyUpdate<Self>
{
    fn min() -> Self;
    fn max() -> Self;

    fn into_float(self) -> f64;
}

impl Number for i64 {
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
impl Number for u64 {
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
impl Number for f64 {
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

    fn new_atomic_tracker(init: u64) -> Self::AtomicTracker {
        AtomicU64::new(init)
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

    fn new_atomic_tracker(init: i64) -> Self::AtomicTracker {
        AtomicI64::new(init)
    }
}

pub(crate) struct F64AtomicTracker {
    inner: AtomicU64, // Floating points don't have true atomics, so we need to use the their binary representation to perform atomic operations
}

impl F64AtomicTracker {
    fn new(init: f64) -> Self {
        let value_as_u64 = init.to_bits();
        F64AtomicTracker {
            inner: AtomicU64::new(value_as_u64),
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

    fn new_atomic_tracker(init: f64) -> Self::AtomicTracker {
        F64AtomicTracker::new(init)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_store_u64_atomic_value() {
        let atomic = u64::new_atomic_tracker(0);
        let atomic_tracker = &atomic as &dyn AtomicTracker<u64>;

        let value = atomic.get_value();
        assert_eq!(value, 0);

        atomic_tracker.store(25);
        let value = atomic.get_value();
        assert_eq!(value, 25);
    }

    #[test]
    fn can_add_and_get_u64_atomic_value() {
        let atomic = u64::new_atomic_tracker(0);
        atomic.add(15);
        atomic.add(10);

        let value = atomic.get_value();
        assert_eq!(value, 25);
    }

    #[test]
    fn can_reset_u64_atomic_value() {
        let atomic = u64::new_atomic_tracker(0);
        atomic.add(15);

        let value = atomic.get_and_reset_value();
        let value2 = atomic.get_value();

        assert_eq!(value, 15, "Incorrect first value");
        assert_eq!(value2, 0, "Incorrect second value");
    }

    #[test]
    fn can_store_i64_atomic_value() {
        let atomic = i64::new_atomic_tracker(0);
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
        let atomic = i64::new_atomic_tracker(0);
        atomic.add(15);
        atomic.add(-10);

        let value = atomic.get_value();
        assert_eq!(value, 5);
    }

    #[test]
    fn can_reset_i64_atomic_value() {
        let atomic = i64::new_atomic_tracker(0);
        atomic.add(15);

        let value = atomic.get_and_reset_value();
        let value2 = atomic.get_value();

        assert_eq!(value, 15, "Incorrect first value");
        assert_eq!(value2, 0, "Incorrect second value");
    }

    #[test]
    fn can_store_f64_atomic_value() {
        let atomic = f64::new_atomic_tracker(0.0);
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
        let atomic = f64::new_atomic_tracker(0.0);
        atomic.add(15.3);
        atomic.add(10.4);

        let value = atomic.get_value();

        assert!(f64::abs(25.7 - value) < 0.0001);
    }

    #[test]
    fn can_reset_f64_atomic_value() {
        let atomic = f64::new_atomic_tracker(0.0);
        atomic.add(15.5);

        let value = atomic.get_and_reset_value();
        let value2 = atomic.get_value();

        assert!(f64::abs(15.5 - value) < 0.0001, "Incorrect first value");
        assert!(f64::abs(0.0 - value2) < 0.0001, "Incorrect second value");
    }
}
