use crate::{metrics::AsyncInstrument, KeyValue};
use core::fmt;
use std::sync::Arc;

/// An SDK implemented instrument that records increasing values.
pub trait SyncCounter<T> {
    /// Records an increment to the counter.
    fn add(&self, value: T, attributes: &[KeyValue]);
}

/// An instrument that records increasing values.
#[derive(Clone)]
pub struct Counter<T>(Arc<dyn SyncCounter<T> + Send + Sync>);

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
    pub fn new(inner: Arc<dyn SyncCounter<T> + Send + Sync>) -> Self {
        Counter(inner)
    }

    /// Records an increment to the counter.
    pub fn add(&self, value: T, attributes: &[KeyValue]) {
        self.0.add(value, attributes)
    }
}

/// An async instrument that records increasing values.
#[derive(Clone)]
pub struct ObservableCounter<T>(Arc<dyn AsyncInstrument<T>>);

impl<T> ObservableCounter<T> {
    /// Create a new observable counter.
    pub fn new(inner: Arc<dyn AsyncInstrument<T>>) -> Self {
        ObservableCounter(inner)
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

impl<T> ObservableCounter<T> {
    /// Records an increment to the counter.
    ///
    /// It is only valid to call this within a callback. If called outside of the
    /// registered callback it should have no effect on the instrument, and an
    /// error will be reported via the error handler.
    pub fn observe(&self, value: T, attributes: &[KeyValue]) {
        self.0.observe(value, attributes)
    }
}

impl<T> AsyncInstrument<T> for ObservableCounter<T> {
    fn observe(&self, measurement: T, attributes: &[KeyValue]) {
        self.0.observe(measurement, attributes)
    }
}
