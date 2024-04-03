use crate::{
    metrics::{InstrumentBuilder, MetricsError},
    KeyValue,
};
use core::fmt;
use std::any::Any;
use std::sync::Arc;

use super::{AsyncInstrument, AsyncInstrumentBuilder};

/// An SDK implemented instrument that records increasing or decreasing values.
pub trait SyncUpDownCounter<T> {
    /// Records an increment or decrement to the counter.
    fn add(&self, value: T, attributes: &[KeyValue]);
}

/// An instrument that records increasing or decreasing values.
#[derive(Clone)]
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

impl TryFrom<InstrumentBuilder<'_, UpDownCounter<i64>>> for UpDownCounter<i64> {
    type Error = MetricsError;

    fn try_from(builder: InstrumentBuilder<'_, UpDownCounter<i64>>) -> Result<Self, Self::Error> {
        builder.meter.instrument_provider.i64_up_down_counter(
            builder.name,
            builder.description,
            builder.unit,
        )
    }
}

impl TryFrom<InstrumentBuilder<'_, UpDownCounter<f64>>> for UpDownCounter<f64> {
    type Error = MetricsError;

    fn try_from(builder: InstrumentBuilder<'_, UpDownCounter<f64>>) -> Result<Self, Self::Error> {
        builder.meter.instrument_provider.f64_up_down_counter(
            builder.name,
            builder.description,
            builder.unit,
        )
    }
}

/// An async instrument that records increasing or decreasing values.
#[derive(Clone)]
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

    /// Records the increment or decrement to the counter.
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

impl<T> AsyncInstrument<T> for ObservableUpDownCounter<T> {
    fn observe(&self, measurement: T, attributes: &[KeyValue]) {
        self.0.observe(measurement, attributes)
    }

    fn as_any(&self) -> Arc<dyn std::any::Any> {
        self.0.as_any()
    }
}

impl TryFrom<AsyncInstrumentBuilder<'_, ObservableUpDownCounter<i64>, i64>>
    for ObservableUpDownCounter<i64>
{
    type Error = MetricsError;

    fn try_from(
        builder: AsyncInstrumentBuilder<'_, ObservableUpDownCounter<i64>, i64>,
    ) -> Result<Self, Self::Error> {
        builder
            .meter
            .instrument_provider
            .i64_observable_up_down_counter(
                builder.name,
                builder.description,
                builder.unit,
                builder.callbacks,
            )
    }
}

impl TryFrom<AsyncInstrumentBuilder<'_, ObservableUpDownCounter<f64>, f64>>
    for ObservableUpDownCounter<f64>
{
    type Error = MetricsError;

    fn try_from(
        builder: AsyncInstrumentBuilder<'_, ObservableUpDownCounter<f64>, f64>,
    ) -> Result<Self, Self::Error> {
        builder
            .meter
            .instrument_provider
            .f64_observable_up_down_counter(
                builder.name,
                builder.description,
                builder.unit,
                builder.callbacks,
            )
    }
}
