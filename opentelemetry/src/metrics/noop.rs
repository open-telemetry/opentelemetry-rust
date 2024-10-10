//! # No-op OpenTelemetry Metrics Implementation
//!
//! This implementation is returned as the global Meter if no `MeterProvider`
//! has been set. It is expected to have minimal resource utilization and
//! runtime impact.
use crate::{
    metrics::{
        AsyncInstrument, InstrumentProvider, Meter, MeterProvider, SyncCounter, SyncGauge,
        SyncHistogram, SyncUpDownCounter,
    },
    KeyValue,
};
use std::sync::Arc;

/// A no-op instance of a `MetricProvider`
#[derive(Debug, Default)]
pub struct NoopMeterProvider {
    _private: (),
}

impl NoopMeterProvider {
    /// Create a new no-op meter provider.
    pub fn new() -> Self {
        NoopMeterProvider { _private: () }
    }
}

impl MeterProvider for NoopMeterProvider {
    fn versioned_meter(
        &self,
        _name: &'static str,
        _version: Option<&'static str>,
        _schema_url: Option<&'static str>,
        _attributes: Option<Vec<KeyValue>>,
    ) -> Meter {
        Meter::new(Arc::new(NoopMeter::new()))
    }
}

/// A no-op instance of a `Meter`
#[derive(Debug, Default)]
pub struct NoopMeter {
    _private: (),
}

impl NoopMeter {
    /// Create a new no-op meter core.
    pub fn new() -> Self {
        NoopMeter { _private: () }
    }
}

impl InstrumentProvider for NoopMeter {}

/// A no-op sync instrument
#[derive(Debug, Default)]
pub struct NoopSyncInstrument {
    _private: (),
}

impl NoopSyncInstrument {
    /// Create a new no-op sync instrument
    pub fn new() -> Self {
        NoopSyncInstrument { _private: () }
    }
}

impl<T> SyncCounter<T> for NoopSyncInstrument {
    fn add(&self, _value: T, _attributes: &[KeyValue]) {
        // Ignored
    }
}

impl<T> SyncUpDownCounter<T> for NoopSyncInstrument {
    fn add(&self, _value: T, _attributes: &[KeyValue]) {
        // Ignored
    }
}

impl<T> SyncHistogram<T> for NoopSyncInstrument {
    fn record(&self, _value: T, _attributes: &[KeyValue]) {
        // Ignored
    }
}

impl<T> SyncGauge<T> for NoopSyncInstrument {
    fn record(&self, _value: T, _attributes: &[KeyValue]) {
        // Ignored
    }
}

/// A no-op async instrument.
#[derive(Debug, Default)]
pub struct NoopAsyncInstrument {
    _private: (),
}

impl NoopAsyncInstrument {
    /// Create a new no-op async instrument
    pub fn new() -> Self {
        NoopAsyncInstrument { _private: () }
    }
}

impl<T> AsyncInstrument<T> for NoopAsyncInstrument {
    fn observe(&self, _value: T, _attributes: &[KeyValue]) {
        // Ignored
    }
}
