//! # No-op OpenTelemetry Metrics Implementation
//!
//! This implementation is returned as the global Meter if no `Meter`
//! has been set. It is also useful for testing purposes as it is intended
//! to have minimal resource utilization and runtime impact.
use crate::{
    metrics::{
        AsyncCounter, AsyncGauge, AsyncUpDownCounter, InstrumentProvider, Meter, MeterProvider,
        Result, SyncCounter, SyncHistogram, SyncUpDownCounter,
    },
    Context, InstrumentationLibrary, KeyValue,
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
        name: &'static str,
        version: Option<&'static str>,
        schema_url: Option<&'static str>,
    ) -> Meter {
        let library = InstrumentationLibrary::new(name, version, schema_url);
        Meter::new(library, Arc::new(NoopMeterCore::new()))
    }
}

/// A no-op instance of a `Meter`
#[derive(Debug, Default)]
pub struct NoopMeterCore {
    _private: (),
}

impl NoopMeterCore {
    /// Create a new no-op meter core.
    pub fn new() -> Self {
        NoopMeterCore { _private: () }
    }
}

impl InstrumentProvider for NoopMeterCore {
    fn register_callback(&self, _callback: Box<dyn Fn(&Context) + Send + Sync>) -> Result<()> {
        Ok(())
    }
}

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
    fn add(&self, _cx: &Context, _value: T, _attributes: &[KeyValue]) {
        // Ignored
    }
}

impl<T> SyncUpDownCounter<T> for NoopSyncInstrument {
    fn add(&self, _cx: &Context, _value: T, _attributes: &[KeyValue]) {
        // Ignored
    }
}

impl<T> SyncHistogram<T> for NoopSyncInstrument {
    fn record(&self, _cx: &Context, _value: T, _attributes: &[KeyValue]) {
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

impl<T> AsyncGauge<T> for NoopAsyncInstrument {
    fn observe(&self, _cx: &Context, _value: T, _attributes: &[KeyValue]) {
        // Ignored
    }
}

impl<T> AsyncCounter<T> for NoopAsyncInstrument {
    fn observe(&self, _cx: &Context, _value: T, _attributes: &[KeyValue]) {
        // Ignored
    }
}

impl<T> AsyncUpDownCounter<T> for NoopAsyncInstrument {
    fn observe(&self, _cx: &Context, _value: T, _attributes: &[KeyValue]) {
        // Ignored
    }
}
