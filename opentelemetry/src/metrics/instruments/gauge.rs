use crate::{
    metrics::{AsyncInstrument, AsyncInstrumentBuilder, InstrumentBuilder, MetricsError},
    KeyValue,
};
use core::fmt;
use std::any::Any;
use std::sync::Arc;

/// An SDK implemented instrument that records independent values
pub trait SyncGauge<T> {
    /// Records an independent value.
    fn record(&self, value: T, attributes: &[KeyValue]);
}

/// An instrument that records independent values
#[derive(Clone)]
pub struct Gauge<T>(Arc<dyn SyncGauge<T> + Send + Sync>);

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
    pub fn new(inner: Arc<dyn SyncGauge<T> + Send + Sync>) -> Self {
        Gauge(inner)
    }

    /// Records an independent value.
    pub fn record(&self, value: T, attributes: &[KeyValue]) {
        self.0.record(value, attributes)
    }
}

impl TryFrom<InstrumentBuilder<Gauge<u64>>> for Gauge<u64> {
    type Error = MetricsError;

    fn try_from(builder: InstrumentBuilder<Gauge<u64>>) -> Result<Self, Self::Error> {
        builder
            .instrument_provider
            .u64_gauge(builder.name, builder.description, builder.unit)
    }
}

impl TryFrom<InstrumentBuilder<Gauge<f64>>> for Gauge<f64> {
    type Error = MetricsError;

    fn try_from(builder: InstrumentBuilder<Gauge<f64>>) -> Result<Self, Self::Error> {
        builder
            .instrument_provider
            .f64_gauge(builder.name, builder.description, builder.unit)
    }
}

impl TryFrom<InstrumentBuilder<Gauge<i64>>> for Gauge<i64> {
    type Error = MetricsError;

    fn try_from(builder: InstrumentBuilder<Gauge<i64>>) -> Result<Self, Self::Error> {
        builder
            .instrument_provider
            .i64_gauge(builder.name, builder.description, builder.unit)
    }
}

/// An async instrument that records independent readings.
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
    pub fn observe(&self, measurement: T, attributes: &[KeyValue]) {
        self.0.observe(measurement, attributes)
    }

    /// Used by SDKs to downcast instruments in callbacks.
    pub fn as_any(&self) -> Arc<dyn Any> {
        self.0.as_any()
    }
}

impl<M> AsyncInstrument<M> for ObservableGauge<M> {
    fn observe(&self, measurement: M, attributes: &[KeyValue]) {
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
