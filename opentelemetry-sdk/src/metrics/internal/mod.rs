mod aggregate;
mod exponential_histogram;
mod histogram;
mod last_value;
mod sum;

use core::fmt;
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Sub};
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};
use std::sync::Mutex;

pub(crate) use aggregate::{AggregateBuilder, ComputeAggregation, Measure};
pub(crate) use exponential_histogram::{EXPO_MAX_SCALE, EXPO_MIN_SCALE};

/// Marks a type that can have a value added and retrieved atomically. Required since
/// different types have different backing atomic mechanisms
pub(crate) trait AtomicValue<T>: Sync + Send + 'static {
    fn add(&self, value: T);
    fn get_value(&self, reset: bool) -> T;
}

/// Keeps track if an atomic value has had a value set since the last reset
pub(crate) struct AtomicTracker<N, T: AtomicValue<N>> {
    value: T,
    has_value: AtomicBool,
    _number: PhantomData<N>, // Required for the N generic to be considered used
}

/// Marks a type that can have an atomic tracker generated for it
pub(crate) trait AtomicallyUpdate<T> {
    type AtomicValue: AtomicValue<T>;
    fn new_atomic_tracker() -> AtomicTracker<T, Self::AtomicValue>;
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

impl AtomicValue<u64> for AtomicU64 {
    fn add(&self, value: u64) {
        self.fetch_add(value, Ordering::Relaxed);
    }

    fn get_value(&self, reset: bool) -> u64 {
        if reset {
            self.swap(0, Ordering::Relaxed)
        } else {
            self.load(Ordering::Relaxed)
        }
    }
}

impl AtomicallyUpdate<u64> for u64 {
    type AtomicValue = AtomicU64;

    fn new_atomic_tracker() -> AtomicTracker<u64, Self::AtomicValue> {
        AtomicTracker::new(AtomicU64::new(0))
    }
}

impl AtomicValue<i64> for AtomicI64 {
    fn add(&self, value: i64) {
        self.fetch_add(value, Ordering::Relaxed);
    }

    fn get_value(&self, reset: bool) -> i64 {
        if reset {
            self.swap(0, Ordering::Relaxed)
        } else {
            self.load(Ordering::Relaxed)
        }
    }
}

impl AtomicallyUpdate<i64> for i64 {
    type AtomicValue = AtomicI64;

    fn new_atomic_tracker() -> AtomicTracker<i64, Self::AtomicValue> {
        AtomicTracker::new(AtomicI64::new(0))
    }
}

pub(crate) struct F64AtomicTracker {
    inner: Mutex<f64>, // Floating points don't have true atomics, so we need to use mutex for them
}

impl F64AtomicTracker {
    pub(crate) fn new() -> Self {
        F64AtomicTracker {
            inner: Mutex::new(0.0),
        }
    }
}

impl AtomicValue<f64> for F64AtomicTracker {
    fn add(&self, value: f64) {
        let mut guard = self.inner.lock().expect("F64 mutex was poisoned");
        *guard += value;
    }

    fn get_value(&self, reset: bool) -> f64 {
        let mut guard = self.inner.lock().expect("F64 mutex was poisoned");
        if reset {
            let value = *guard;
            *guard = 0.0;
            value
        } else {
            *guard
        }
    }
}

impl AtomicallyUpdate<f64> for f64 {
    type AtomicValue = F64AtomicTracker;

    fn new_atomic_tracker() -> AtomicTracker<f64, Self::AtomicValue> {
        AtomicTracker::new(F64AtomicTracker::new())
    }
}

impl<N, T: AtomicValue<N>> AtomicTracker<N, T> {
    fn new(value: T) -> Self {
        AtomicTracker {
            value,
            has_value: AtomicBool::new(false),
            _number: PhantomData::default(),
        }
    }

    pub(crate) fn add(&self, value: N) {
        self.value.add(value);
        self.has_value.store(true, Ordering::Release);
    }

    pub(crate) fn get_value(&self, reset: bool) -> Option<N> {
        let has_value = if reset {
            self.has_value.swap(false, Ordering::AcqRel)
        } else {
            self.has_value.load(Ordering::Acquire)
        };

        if has_value {
            Some(self.value.get_value(reset))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_add_and_get_u64_atomic_value() {
        let atomic = u64::new_atomic_tracker();
        atomic.add(15);
        atomic.add(10);

        let value = atomic.get_value(false).unwrap();
        assert_eq!(value, 25);
    }

    #[test]
    fn can_reset_u64_atomic_value() {
        let atomic = u64::new_atomic_tracker();
        atomic.add(15);

        let value = atomic.get_value(true).unwrap();
        let value2 = atomic.get_value(false).unwrap();

        assert_eq!(value, 15, "Incorrect first value");
        assert_eq!(value2, 0, "Incorrect second value");
    }

    #[test]
    fn can_add_and_get_i64_atomic_value() {
        let atomic = i64::new_atomic_tracker();
        atomic.add(15);
        atomic.add(-10);

        let value = atomic.get_value(false).unwrap();
        assert_eq!(value, 5);
    }

    #[test]
    fn can_reset_i64_atomic_value() {
        let atomic = i64::new_atomic_tracker();
        atomic.add(15);

        let value = atomic.get_value(true).unwrap();
        let value2 = atomic.get_value(false).unwrap();

        assert_eq!(value, 15, "Incorrect first value");
        assert_eq!(value2, 0, "Incorrect second value");
    }

    #[test]
    fn can_add_and_get_f64_atomic_value() {
        let atomic = f64::new_atomic_tracker();
        atomic.add(15.3);
        atomic.add(10.4);

        let value = atomic.get_value(false).unwrap();

        assert!(f64::abs(25.7 - value) < 0.0001);
    }

    #[test]
    fn can_reset_f64_atomic_value() {
        let atomic = f64::new_atomic_tracker();
        atomic.add(15.5);

        let value = atomic.get_value(true).unwrap();
        let value2 = atomic.get_value(false).unwrap();

        assert!(f64::abs(15.5 - value) < 0.0001, "Incorrect first value");
        assert!(f64::abs(0.0 - value2) < 0.0001, "Incorrect second value");
    }
}
