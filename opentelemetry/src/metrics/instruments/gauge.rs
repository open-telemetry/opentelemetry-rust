use crate::{metrics::AsyncInstrument, KeyValue};
use core::fmt;
use std::sync::Arc;

use super::SyncInstrument;

/// An instrument that records independent values
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
}

/// An async instrument that records independent readings.
#[derive(Clone)]
#[non_exhaustive]
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

impl<M> AsyncInstrument<M> for ObservableGauge<M> {
    fn observe(&self, measurement: M, attributes: &[KeyValue]) {
        self.0.observe(measurement, attributes)
    }
}

impl<T> ObservableGauge<T> {
    /// Create a new gauge
    pub fn new(inner: Arc<dyn AsyncInstrument<T>>) -> Self {
        ObservableGauge(inner)
    }
}
