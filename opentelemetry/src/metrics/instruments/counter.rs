use crate::{
    metrics::{AsyncInstrument, AsyncInstrumentBuilder, InstrumentBuilder, MetricsError},
    KeyValue,
};
use core::fmt;
use std::sync::Arc;
use std::{any::Any, convert::TryFrom};

/// An SDK implemented instrument that records increasing values.
pub trait SyncCounter<T> {
    /// Records an increment to the counter.
    fn add(&self, value: T, attributes: &[KeyValue]);

    /// Creates a child counter that's bound to a specific attribute set
    fn bind(&self, attributes: &[KeyValue]) -> Arc<dyn BoundSyncCounter<T> + Send + Sync>;
}

/// An SDK implemented instrument that records increasing values for a predefined attribute set
pub trait BoundSyncCounter<T> {
    /// Records an increment to the counter
    fn add(&self, value: T);
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

    /// Creates an instance of this counter that's bound to a specific set of attributes
    pub fn bind(&self, attributes: &[KeyValue]) -> BoundCounter<T> {
        BoundCounter(self.0.bind(attributes))
    }
}

impl TryFrom<InstrumentBuilder<'_, Counter<u64>>> for Counter<u64> {
    type Error = MetricsError;

    fn try_from(builder: InstrumentBuilder<'_, Counter<u64>>) -> Result<Self, Self::Error> {
        builder.meter.instrument_provider.u64_counter(
            builder.name,
            builder.description,
            builder.unit,
        )
    }
}

impl TryFrom<InstrumentBuilder<'_, Counter<f64>>> for Counter<f64> {
    type Error = MetricsError;

    fn try_from(builder: InstrumentBuilder<'_, Counter<f64>>) -> Result<Self, Self::Error> {
        builder.meter.instrument_provider.f64_counter(
            builder.name,
            builder.description,
            builder.unit,
        )
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

    /// Used for SDKs to downcast instruments in callbacks.
    pub fn as_any(&self) -> Arc<dyn Any> {
        self.0.as_any()
    }
}

impl<T> AsyncInstrument<T> for ObservableCounter<T> {
    fn observe(&self, measurement: T, attributes: &[KeyValue]) {
        self.0.observe(measurement, attributes)
    }

    fn as_any(&self) -> Arc<dyn Any> {
        self.0.as_any()
    }
}

impl TryFrom<AsyncInstrumentBuilder<'_, ObservableCounter<u64>, u64>> for ObservableCounter<u64> {
    type Error = MetricsError;

    fn try_from(
        builder: AsyncInstrumentBuilder<'_, ObservableCounter<u64>, u64>,
    ) -> Result<Self, Self::Error> {
        builder.meter.instrument_provider.u64_observable_counter(
            builder.name,
            builder.description,
            builder.unit,
            builder.callbacks,
        )
    }
}

impl TryFrom<AsyncInstrumentBuilder<'_, ObservableCounter<f64>, f64>> for ObservableCounter<f64> {
    type Error = MetricsError;

    fn try_from(
        builder: AsyncInstrumentBuilder<'_, ObservableCounter<f64>, f64>,
    ) -> Result<Self, Self::Error> {
        builder.meter.instrument_provider.f64_observable_counter(
            builder.name,
            builder.description,
            builder.unit,
            builder.callbacks,
        )
    }
}

/// An instrument that records increasing values for a predefined attribute set.
#[derive(Clone)]
pub struct BoundCounter<T>(Arc<dyn BoundSyncCounter<T> + Send + Sync>);

impl<T> BoundCounter<T> {
    /// Records an increment to the counter
    pub fn add(&self, value: T) {
        self.0.add(value);
    }
}

impl<T> fmt::Debug for BoundCounter<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("BoundCounter<{}>", std::any::type_name::<T>()))
    }
}
