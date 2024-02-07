mod aggregate;
mod exponential_histogram;
mod histogram;
mod last_value;
mod sum;

use core::fmt;
use std::ops::{Add, AddAssign, Sub};
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::Mutex;

pub(crate) use aggregate::{AggregateBuilder, ComputeAggregation, Measure};
pub(crate) use exponential_histogram::{EXPO_MAX_SCALE, EXPO_MIN_SCALE};

/// Marks a type that can have a value added and retrieved atomically. Required since
/// different types have different backing atomic mechanisms
pub(crate) trait AtomicTracker<T>: Sync + Send + 'static {
    fn add(&self, value: T);
    fn get_value(&self) -> T;
    fn get_and_reset_value(&self) -> T;
}

/// Marks a type that can have an atomic tracker generated for it
pub(crate) trait AtomicallyUpdate<T> {
    type Tracker: AtomicTracker<T>;
    fn new_atomic_tracker() -> Self::Tracker;
}

pub(crate) trait Number<T>:
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
    type Tracker = AtomicU64;

    fn new_atomic_tracker() -> Self::Tracker {
        AtomicU64::new(0)
    }
}

impl AtomicTracker<i64> for AtomicI64 {
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
    type Tracker = AtomicI64;

    fn new_atomic_tracker() -> Self::Tracker {
        AtomicI64::new(0)
    }
}

pub(crate) struct F64AtomicTracker {
    inner: Mutex<f64>, // Floating points don't have true atomics, so we need to use mutex for them
}

impl F64AtomicTracker {
    fn new() -> Self {
        F64AtomicTracker {
            inner: Mutex::new(0.0),
        }
    }
}

impl AtomicTracker<f64> for F64AtomicTracker {
    fn add(&self, value: f64) {
        let mut guard = self.inner.lock().expect("F64 mutex was poisoned");
        *guard += value;
    }

    fn get_value(&self) -> f64 {
        let guard = self.inner.lock().expect("F64 mutex was poisoned");
        *guard
    }

    fn get_and_reset_value(&self) -> f64 {
        let mut guard = self.inner.lock().expect("F64 mutex was poisoned");
        let value = *guard;
        *guard = 0.0;

        value
    }
}

impl AtomicallyUpdate<f64> for f64 {
    type Tracker = F64AtomicTracker;

    fn new_atomic_tracker() -> Self::Tracker {
        F64AtomicTracker::new()
    }
}
