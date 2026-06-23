use crate::KeyValue;
use core::fmt;
use std::sync::Arc;

#[cfg(feature = "experimental_metrics_bound_instruments")]
use super::BoundSyncInstrument;
use super::SyncInstrument;

/// An instrument that records independent values
///
/// [`Gauge`] can be cloned to create multiple handles to the same instrument. If a [`Gauge`] needs to be shared,
/// users are recommended to clone the [`Gauge`] instead of creating duplicate [`Gauge`]s for the same metric. Creating
/// duplicate [`Gauge`]s for the same metric could lower SDK performance.
#[derive(Clone)]
#[non_exhaustive]
pub struct Gauge<T>(Arc<dyn SyncInstrument<T> + Send + Sync>);

impl<T> fmt::Debug for Gauge<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("Gauge<{}>", std::any::type_name::<T>()))
    }
}

impl<T> Gauge<T> {
    /// Create a new gauge.
    pub fn new(inner: Arc<dyn SyncInstrument<T> + Send + Sync>) -> Self {
        Gauge(inner)
    }

    /// Records an independent value.
    pub fn record(&self, value: T, attributes: &[KeyValue]) {
        self.0.measure(value, attributes)
    }

    /// Binds this gauge to a fixed set of attributes.
    #[cfg(feature = "experimental_metrics_bound_instruments")]
    pub fn bind(&self, attributes: &[KeyValue]) -> BoundGauge<T> {
        BoundGauge(Arc::from(self.0.bind(attributes)))
    }
}

/// A gauge bound to a fixed set of attributes.
///
/// Created by calling [`Gauge::bind`] with an attribute set. All subsequent
/// [`record`](BoundGauge::record) calls use the pre-resolved attributes, bypassing
/// per-call attribute lookup for significantly better performance.
///
/// `BoundGauge` can be cloned cheaply to share a single bound state across
/// threads or modules without re-binding. The underlying tracker is reclaimed
/// when the last clone is dropped.
#[cfg(feature = "experimental_metrics_bound_instruments")]
#[derive(Clone)]
#[must_use = "dropping a BoundGauge immediately is a no-op; store it to benefit from pre-bound attributes"]
pub struct BoundGauge<T>(Arc<dyn BoundSyncInstrument<T> + Send + Sync>);

#[cfg(feature = "experimental_metrics_bound_instruments")]
impl<T> fmt::Debug for BoundGauge<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("BoundGauge<{}>", std::any::type_name::<T>()))
    }
}

#[cfg(feature = "experimental_metrics_bound_instruments")]
impl<T> BoundGauge<T> {
    /// Records an independent value using the pre-bound attributes.
    pub fn record(&self, value: T) {
        self.0.measure(value)
    }
}

/// An async instrument that records independent readings.
#[derive(Clone)]
#[non_exhaustive]
pub struct ObservableGauge<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T> fmt::Debug for ObservableGauge<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "ObservableGauge<{}>",
            std::any::type_name::<T>()
        ))
    }
}

impl<T> ObservableGauge<T> {
    /// Create a new gauge
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        ObservableGauge {
            _marker: std::marker::PhantomData,
        }
    }
}
