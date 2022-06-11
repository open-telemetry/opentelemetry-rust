use crate::{
    metrics::{InstrumentBuilder, MetricsError},
    Context, KeyValue,
};
use core::fmt;
use std::convert::TryFrom;
use std::sync::Arc;

/// An SDK implemented instrument that records increasing values.
pub trait SyncCounter<T> {
    /// Records an increment to the counter.
    fn add(&self, cx: &Context, value: T, attributes: &[KeyValue]);
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
    pub fn add(&self, cx: &Context, value: T, attributes: &[KeyValue]) {
        self.0.add(cx, value, attributes)
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

/// An SDK implemented async instrument that records increasing values.
pub trait AsyncCounter<T> {
    /// Records an increment to the counter.
    ///
    /// It is only valid to call this within a callback. If called outside of the
    /// registered callback it should have no effect on the instrument, and an
    /// error will be reported via the error handler.
    fn observe(&self, cx: &Context, value: T, attributes: &[KeyValue]);
}

/// An async instrument that records increasing values.
pub struct ObservableCounter<T>(Arc<dyn AsyncCounter<T> + Send + Sync>);

impl<T> ObservableCounter<T> {
    /// Create a new observable counter.
    pub fn new(inner: Arc<dyn AsyncCounter<T> + Send + Sync>) -> Self {
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
    pub fn observe(&self, cx: &Context, value: T, attributes: &[KeyValue]) {
        self.0.observe(cx, value, attributes)
    }
}

impl TryFrom<InstrumentBuilder<'_, ObservableCounter<u64>>> for ObservableCounter<u64> {
    type Error = MetricsError;

    fn try_from(
        builder: InstrumentBuilder<'_, ObservableCounter<u64>>,
    ) -> Result<Self, Self::Error> {
        builder.meter.instrument_provider.u64_observable_counter(
            builder.name,
            builder.description,
            builder.unit,
        )
    }
}

impl TryFrom<InstrumentBuilder<'_, ObservableCounter<f64>>> for ObservableCounter<f64> {
    type Error = MetricsError;

    fn try_from(
        builder: InstrumentBuilder<'_, ObservableCounter<f64>>,
    ) -> Result<Self, Self::Error> {
        builder.meter.instrument_provider.f64_observable_counter(
            builder.name,
            builder.description,
            builder.unit,
        )
    }
}
