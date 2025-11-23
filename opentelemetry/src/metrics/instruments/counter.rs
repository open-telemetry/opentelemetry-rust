use crate::KeyValue;
use core::fmt;
use std::sync::Arc;

use super::SyncInstrument;

/// A monotonic instrument that records increasing values.
///
/// Counters are used to measure values that only increase over time, such as the number
/// of requests received, bytes sent, or errors encountered. Only non-negative values
/// should be recorded. Negative values violate the monotonic property and will be
/// dropped by the SDK with a warning.
///
/// # Cloning
///
/// [`Counter`] can be cloned to create multiple handles to the same instrument. If a [`Counter`] needs to be shared,
/// users are recommended to clone the [`Counter`] instead of creating duplicate [`Counter`]s for the same metric. Creating
/// duplicate [`Counter`]s for the same metric could lower SDK performance.
#[derive(Clone)]
#[non_exhaustive]
pub struct Counter<T>(Arc<dyn SyncInstrument<T> + Send + Sync>);

impl<T> fmt::Debug for Counter<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("Counter<{}>", std::any::type_name::<T>()))
    }
}

impl<T> Counter<T> {
    /// Create a new counter.
    pub fn new(inner: Arc<dyn SyncInstrument<T> + Send + Sync>) -> Self {
        Counter(inner)
    }

    /// Records an increment to the counter.
    ///
    /// # Arguments
    ///
    /// * `value` - A non-negative value to add to the counter. According to the
    ///   OpenTelemetry specification, counters are monotonic instruments that record
    ///   increasing values. Passing a negative value violates this contract.
    ///
    /// * `attributes` - A set of key-value pairs that describe the measurement context.
    ///
    /// # Behavior with negative values
    ///
    /// The API does not validate the value, but the SDK implementation will log a warning
    /// and drop negative values to maintain the monotonic property of counters. Applications
    /// should ensure only non-negative values are passed to this method.
    pub fn add(&self, value: T, attributes: &[KeyValue]) {
        self.0.measure(value, attributes)
    }
}

/// A monotonic asynchronous instrument that records increasing values.
///
/// Observable counters are used to measure values that only increase over time and are
/// observed via callbacks, such as process CPU time or total memory usage. Only non-negative
/// values should be recorded. Negative values violate the monotonic property and will be
/// dropped by the SDK with a warning.
#[derive(Clone)]
#[non_exhaustive]
pub struct ObservableCounter<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T> ObservableCounter<T> {
    /// Create a new observable counter.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        ObservableCounter {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T> fmt::Debug for ObservableCounter<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "ObservableCounter<{}>",
            std::any::type_name::<T>()
        ))
    }
}
