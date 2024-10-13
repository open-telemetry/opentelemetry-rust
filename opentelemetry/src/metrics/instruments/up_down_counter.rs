use crate::KeyValue;
use core::fmt;
use std::sync::Arc;

use super::AsyncInstrument;

/// An SDK implemented instrument that records increasing or decreasing values.
pub trait SyncUpDownCounter<T> {
    /// Records an increment or decrement to the counter.
    fn add(&self, value: T, attributes: &[KeyValue]);
}

/// An instrument that records increasing or decreasing values.
#[derive(Clone)]
#[non_exhaustive]
pub struct UpDownCounter<T>(Arc<dyn SyncUpDownCounter<T> + Send + Sync>);

impl<T> fmt::Debug for UpDownCounter<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "UpDownCounter<{}>",
            std::any::type_name::<T>()
        ))
    }
}

impl<T> UpDownCounter<T> {
    /// Create a new up down counter.
    pub fn new(inner: Arc<dyn SyncUpDownCounter<T> + Send + Sync>) -> Self {
        UpDownCounter(inner)
    }

    /// Records an increment or decrement to the counter.
    pub fn add(&self, value: T, attributes: &[KeyValue]) {
        self.0.add(value, attributes)
    }
}

/// An async instrument that records increasing or decreasing values.
#[derive(Clone)]
#[non_exhaustive]
pub struct ObservableUpDownCounter<T>(Arc<dyn AsyncInstrument<T>>);

impl<T> fmt::Debug for ObservableUpDownCounter<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "ObservableUpDownCounter<{}>",
            std::any::type_name::<T>()
        ))
    }
}

impl<T> ObservableUpDownCounter<T> {
    /// Create a new observable up down counter.
    pub fn new(inner: Arc<dyn AsyncInstrument<T>>) -> Self {
        ObservableUpDownCounter(inner)
    }
}

impl<T> AsyncInstrument<T> for ObservableUpDownCounter<T> {
    fn observe(&self, measurement: T, attributes: &[KeyValue]) {
        self.0.observe(measurement, attributes)
    }
}
