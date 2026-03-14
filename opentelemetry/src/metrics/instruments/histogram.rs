use crate::KeyValue;
use core::fmt;
use std::sync::Arc;

use super::SyncInstrument;
#[cfg(feature = "experimental_metrics_bound_instruments")]
use super::BoundSyncInstrument;

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

    /// Binds this histogram to a fixed set of attributes.
    #[cfg(feature = "experimental_metrics_bound_instruments")]
    pub fn bind(&self, attributes: &[KeyValue]) -> BoundHistogram<T> {
        BoundHistogram(self.0.bind(attributes))
    }
}

/// A histogram bound to a fixed set of attributes.
///
/// Created by calling [`Histogram::bind`] with an attribute set. All subsequent
/// [`record`](BoundHistogram::record) calls use the pre-resolved attributes, bypassing
/// per-call attribute lookup for significantly better performance.
#[cfg(feature = "experimental_metrics_bound_instruments")]
#[must_use = "dropping a BoundHistogram immediately is a no-op; store it to benefit from pre-bound attributes"]
pub struct BoundHistogram<T>(Box<dyn BoundSyncInstrument<T> + Send + Sync>);

#[cfg(feature = "experimental_metrics_bound_instruments")]
impl<T> fmt::Debug for BoundHistogram<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "BoundHistogram<{}>",
            std::any::type_name::<T>()
        ))
    }
}

#[cfg(feature = "experimental_metrics_bound_instruments")]
impl<T> BoundHistogram<T> {
    /// Records a value in the histogram using the pre-bound attributes.
    pub fn record(&self, value: T) {
        self.0.measure(value)
    }
}
