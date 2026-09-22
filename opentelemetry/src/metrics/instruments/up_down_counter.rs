use crate::KeyValue;
use core::fmt;
use std::sync::Arc;

#[cfg(feature = "experimental_metrics_bound_instruments")]
use super::BoundSyncInstrument;
use super::SyncInstrument;

/// An instrument that records increasing or decreasing values.
///
/// [`UpDownCounter`] can be cloned to create multiple handles to the same instrument. If a [`UpDownCounter`] needs to be shared,
/// users are recommended to clone the [`UpDownCounter`] instead of creating duplicate [`UpDownCounter`]s for the same metric. Creating
/// duplicate [`UpDownCounter`]s for the same metric could lower SDK performance.
#[derive(Clone)]
#[non_exhaustive]
pub struct UpDownCounter<T>(Arc<dyn SyncInstrument<T> + Send + Sync>);

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
    pub fn new(inner: Arc<dyn SyncInstrument<T> + Send + Sync>) -> Self {
        UpDownCounter(inner)
    }

    /// Records an increment or decrement to the counter.
    pub fn add(&self, value: T, attributes: &[KeyValue]) {
        self.0.measure(value, attributes)
    }

    /// Binds this up-down counter to a fixed set of attributes.
    ///
    /// Corresponds to the `Bind` capability in the OpenTelemetry spec (status:
    /// Development as of spec 1.57.0).
    #[cfg(feature = "experimental_metrics_bound_instruments")]
    pub fn bind(&self, attributes: &[KeyValue]) -> BoundUpDownCounter<T> {
        BoundUpDownCounter(Arc::from(self.0.bind(attributes)))
    }
}

/// An up-down counter bound to a fixed set of attributes.
///
/// Created by calling [`UpDownCounter::bind`] with an attribute set. All subsequent
/// [`add`](BoundUpDownCounter::add) calls use the pre-resolved attributes, bypassing
/// per-call attribute lookup for significantly better performance.
///
/// `BoundUpDownCounter` can be cloned cheaply to share a single bound state across
/// threads or modules without re-binding. The underlying tracker is reclaimed
/// when the last clone is dropped.
#[cfg(feature = "experimental_metrics_bound_instruments")]
#[derive(Clone)]
#[must_use = "dropping a BoundUpDownCounter immediately is a no-op; store it to benefit from pre-bound attributes"]
pub struct BoundUpDownCounter<T>(Arc<dyn BoundSyncInstrument<T> + Send + Sync>);

#[cfg(feature = "experimental_metrics_bound_instruments")]
impl<T> fmt::Debug for BoundUpDownCounter<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "BoundUpDownCounter<{}>",
            std::any::type_name::<T>()
        ))
    }
}

#[cfg(feature = "experimental_metrics_bound_instruments")]
impl<T> BoundUpDownCounter<T> {
    /// Records an increment or decrement using the pre-bound attributes.
    pub fn add(&self, value: T) {
        self.0.measure(value)
    }
}

/// An async instrument that records increasing or decreasing values.
#[derive(Clone)]
#[non_exhaustive]
pub struct ObservableUpDownCounter<T> {
    _marker: std::marker::PhantomData<T>,
}

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
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        ObservableUpDownCounter {
            _marker: std::marker::PhantomData,
        }
    }
}
