use crate::{
    metrics::{InstrumentBuilder, MetricsError},
    KeyValue,
};
use core::fmt;
use std::sync::Arc;

/// An SDK implemented instrument that records a distribution of values.
pub trait SyncHistogram<T> {
    /// Adds an additional value to the distribution.
    fn record(&self, value: T, attributes: &[KeyValue]);
}

/// An instrument that records a distribution of values.
#[derive(Clone)]
pub struct Histogram<T>(Arc<dyn SyncHistogram<T> + Send + Sync>);

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
    pub fn new(inner: Arc<dyn SyncHistogram<T> + Send + Sync>) -> Self {
        Histogram(inner)
    }

    /// Adds an additional value to the distribution.
    pub fn record(&self, value: T, attributes: &[KeyValue]) {
        self.0.record(value, attributes)
    }
}

impl TryFrom<InstrumentBuilder<Histogram<f64>>> for Histogram<f64> {
    type Error = MetricsError;

    fn try_from(builder: InstrumentBuilder<Histogram<f64>>) -> Result<Self, Self::Error> {
        builder
            .instrument_provider
            .f64_histogram(builder.name, builder.description, builder.unit)
    }
}

impl TryFrom<InstrumentBuilder<Histogram<u64>>> for Histogram<u64> {
    type Error = MetricsError;

    fn try_from(builder: InstrumentBuilder<Histogram<u64>>) -> Result<Self, Self::Error> {
        builder
            .instrument_provider
            .u64_histogram(builder.name, builder.description, builder.unit)
    }
}
