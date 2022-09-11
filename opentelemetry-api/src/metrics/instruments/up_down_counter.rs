use crate::{
    metrics::{InstrumentBuilder, MetricsError},
    Context, KeyValue,
};
use core::fmt;
use std::convert::TryFrom;
use std::sync::Arc;

/// An SDK implemented instrument that records increasing or decreasing values.
pub trait SyncUpDownCounter<T> {
    /// Records an increment or decrement to the counter.
    fn add(&self, cx: &Context, value: T, attributes: &[KeyValue]);
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
    pub fn add(&self, cx: &Context, value: T, attributes: &[KeyValue]) {
        self.0.add(cx, value, attributes)
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

/// An SDK implemented async instrument that records increasing or decreasing values.
pub trait AsyncUpDownCounter<T> {
    /// Records the increment or decrement to the counter.
    ///
    /// It is only valid to call this within a callback. If called outside of the
    /// registered callback it should have no effect on the instrument, and an
    /// error will be reported via the error handler.
    fn observe(&self, cx: &Context, value: T, attributes: &[KeyValue]);
}

/// An async instrument that records increasing or decreasing values.
#[derive(Clone)]
pub struct ObservableUpDownCounter<T>(Arc<dyn AsyncUpDownCounter<T> + Send + Sync>);

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
    pub fn new(inner: Arc<dyn AsyncUpDownCounter<T> + Send + Sync>) -> Self {
        ObservableUpDownCounter(inner)
    }

    /// Records the increment or decrement to the counter.
    ///
    /// It is only valid to call this within a callback. If called outside of the
    /// registered callback it should have no effect on the instrument, and an
    /// error will be reported via the error handler.
    pub fn observe(&self, cx: &Context, value: T, attributes: &[KeyValue]) {
        self.0.observe(cx, value, attributes)
    }
}

impl TryFrom<InstrumentBuilder<'_, ObservableUpDownCounter<i64>>> for ObservableUpDownCounter<i64> {
    type Error = MetricsError;

    fn try_from(
        builder: InstrumentBuilder<'_, ObservableUpDownCounter<i64>>,
    ) -> Result<Self, Self::Error> {
        builder
            .meter
            .instrument_provider
            .i64_observable_up_down_counter(builder.name, builder.description, builder.unit)
    }
}

impl TryFrom<InstrumentBuilder<'_, ObservableUpDownCounter<f64>>> for ObservableUpDownCounter<f64> {
    type Error = MetricsError;

    fn try_from(
        builder: InstrumentBuilder<'_, ObservableUpDownCounter<f64>>,
    ) -> Result<Self, Self::Error> {
        builder
            .meter
            .instrument_provider
            .f64_observable_up_down_counter(builder.name, builder.description, builder.unit)
    }
}
