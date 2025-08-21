use crate::KeyValue;
use core::fmt;
use std::sync::Arc;

use super::SyncInstrument;

/// An instrument that records a distribution of values.
///
/// [`Histogram`] can be cloned to create multiple handles to the same instrument. If a [`Histogram`] needs to be shared,
/// users are recommended to clone the [`Histogram`] instead of creating duplicate [`Histogram`]s for the same metric. Creating
/// duplicate [`Histogram`]s for the same metric could lower SDK performance.
#[derive(Clone)]
#[non_exhaustive]
pub struct Histogram<T>(Arc<dyn SyncInstrument<T> + Send + Sync>);

impl<T> fmt::Debug for Histogram<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("Histogram<{}>", std::any::type_name::<T>()))
    }
}

impl<T> Histogram<T> {
    /// Create a new histogram.
    pub fn new(inner: Arc<dyn SyncInstrument<T> + Send + Sync>) -> Self {
        Histogram(inner)
    }

    /// Adds an additional value to the distribution.
    pub fn record(&self, value: T, attributes: &[KeyValue]) {
        self.0.measure(value, attributes)
    }
}

/// An async instrument that records a distribution of values.
#[derive(Clone)]
#[non_exhaustive]
pub struct ObservableHistogram<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T> fmt::Debug for ObservableHistogram<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "ObservableHistogram<{}>",
            std::any::type_name::<T>()
        ))
    }
}

impl<T> ObservableHistogram<T> {
    /// Create a new gauge
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        ObservableHistogram {
            _marker: std::marker::PhantomData,
        }
    }
}
