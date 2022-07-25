use crate::{
    metrics::{InstrumentBuilder, MetricsError},
    Context, KeyValue,
};
use core::fmt;
use std::convert::TryFrom;
use std::sync::Arc;

/// An SDK implemented instrument that records independent readings.
pub trait AsyncGauge<T>: Send + Sync {
    /// Records the state of the instrument.
    ///
    /// It is only valid to call this within a callback. If called outside of the
    /// registered callback it should have no effect on the instrument, and an
    /// error will be reported via the error handler.
    fn observe(&self, cx: &Context, value: T, attributes: &[KeyValue]);
}

/// An instrument that records independent readings.
#[derive(Clone)]
pub struct ObservableGauge<T>(Arc<dyn AsyncGauge<T>>);

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
    /// Records the state of the instrument.
    ///
    /// It is only valid to call this within a callback. If called outside of the
    /// registered callback it should have no effect on the instrument, and an
    /// error will be reported via the error handler.
    pub fn observe(&self, cx: &Context, value: T, attributes: &[KeyValue]) {
        self.0.observe(cx, value, attributes)
    }
}

impl<T> ObservableGauge<T> {
    /// Create a new gauge
    pub fn new(inner: Arc<dyn AsyncGauge<T>>) -> Self {
        ObservableGauge(inner)
    }
}

impl TryFrom<InstrumentBuilder<'_, ObservableGauge<u64>>> for ObservableGauge<u64> {
    type Error = MetricsError;

    fn try_from(builder: InstrumentBuilder<'_, ObservableGauge<u64>>) -> Result<Self, Self::Error> {
        builder.meter.instrument_provider.u64_observable_gauge(
            builder.name,
            builder.description,
            builder.unit,
        )
    }
}

impl TryFrom<InstrumentBuilder<'_, ObservableGauge<f64>>> for ObservableGauge<f64> {
    type Error = MetricsError;

    fn try_from(builder: InstrumentBuilder<'_, ObservableGauge<f64>>) -> Result<Self, Self::Error> {
        builder.meter.instrument_provider.f64_observable_gauge(
            builder.name,
            builder.description,
            builder.unit,
        )
    }
}

impl TryFrom<InstrumentBuilder<'_, ObservableGauge<i64>>> for ObservableGauge<i64> {
    type Error = MetricsError;

    fn try_from(builder: InstrumentBuilder<'_, ObservableGauge<i64>>) -> Result<Self, Self::Error> {
        builder.meter.instrument_provider.i64_observable_gauge(
            builder.name,
            builder.description,
            builder.unit,
        )
    }
}
