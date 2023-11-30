use crate::attributes::AttributeSet;
use crate::metrics::{AsyncInstrument, AsyncInstrumentBuilder, MetricsError};
use core::fmt;
use std::sync::Arc;
use std::{any::Any, convert::TryFrom};

/// An instrument that records independent readings.
#[derive(Clone)]
pub struct ObservableGauge<T>(Arc<dyn AsyncInstrument<T>>);

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
    pub fn observe(&self, measurement: T, attributes: AttributeSet) {
        self.0.observe(measurement, attributes)
    }

    /// Used by SDKs to downcast instruments in callbacks.
    pub fn as_any(&self) -> Arc<dyn Any> {
        self.0.as_any()
    }
}

impl<M> AsyncInstrument<M> for ObservableGauge<M> {
    fn observe(&self, measurement: M, attributes: AttributeSet) {
        self.observe(measurement, attributes)
    }

    fn as_any(&self) -> Arc<dyn Any> {
        self.0.as_any()
    }
}

impl<T> ObservableGauge<T> {
    /// Create a new gauge
    pub fn new(inner: Arc<dyn AsyncInstrument<T>>) -> Self {
        ObservableGauge(inner)
    }
}

impl TryFrom<AsyncInstrumentBuilder<'_, ObservableGauge<u64>, u64>> for ObservableGauge<u64> {
    type Error = MetricsError;

    fn try_from(
        builder: AsyncInstrumentBuilder<'_, ObservableGauge<u64>, u64>,
    ) -> Result<Self, Self::Error> {
        builder.meter.instrument_provider.u64_observable_gauge(
            builder.name,
            builder.description,
            builder.unit,
            builder.callbacks,
        )
    }
}

impl TryFrom<AsyncInstrumentBuilder<'_, ObservableGauge<f64>, f64>> for ObservableGauge<f64> {
    type Error = MetricsError;

    fn try_from(
        builder: AsyncInstrumentBuilder<'_, ObservableGauge<f64>, f64>,
    ) -> Result<Self, Self::Error> {
        builder.meter.instrument_provider.f64_observable_gauge(
            builder.name,
            builder.description,
            builder.unit,
            builder.callbacks,
        )
    }
}

impl TryFrom<AsyncInstrumentBuilder<'_, ObservableGauge<i64>, i64>> for ObservableGauge<i64> {
    type Error = MetricsError;

    fn try_from(
        builder: AsyncInstrumentBuilder<'_, ObservableGauge<i64>, i64>,
    ) -> Result<Self, Self::Error> {
        builder.meter.instrument_provider.i64_observable_gauge(
            builder.name,
            builder.description,
            builder.unit,
            builder.callbacks,
        )
    }
}
