//! # No-op OpenTelemetry Metrics Implementation
//!
//! This implementation is returned as the global Meter if no `Meter`
//! has been set. It is also useful for testing purposes as it is intended
//! to have minimal resource utilization and runtime impact.
use crate::{
    metrics::{
        AsyncInstrument, CallbackRegistration, InstrumentProvider, Meter, MeterProvider, Observer,
        Result, SyncCounter, SyncGauge, SyncHistogram, SyncUpDownCounter,
    },
    KeyValue,
};
use std::{any::Any, borrow::Cow, sync::Arc};

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
        _name: impl Into<Cow<'static, str>>,
        _version: Option<impl Into<Cow<'static, str>>>,
        _schema_url: Option<impl Into<Cow<'static, str>>>,
        _attributes: Option<Vec<KeyValue>>,
    ) -> Meter {
        Meter::new(Arc::new(NoopMeterCore::new()))
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
    fn register_callback(
        &self,
        _instruments: &[Arc<dyn Any>],
        _callback: Box<dyn Fn(&dyn Observer) + Send + Sync>,
    ) -> Result<Box<dyn CallbackRegistration>> {
        Ok(Box::new(NoopRegistration::new()))
    }
}

/// A no-op instance of a callback [CallbackRegistration].
#[derive(Debug, Default)]
pub struct NoopRegistration {
    _private: (),
}

impl NoopRegistration {
    /// Create a new no-op registration.
    pub fn new() -> Self {
        NoopRegistration { _private: () }
    }
}

impl CallbackRegistration for NoopRegistration {
    fn unregister(&mut self) -> Result<()> {
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

    fn as_any(&self) -> Arc<dyn Any> {
        Arc::new(())
    }
}
