//! # No-op OpenTelemetry Metrics Implementation
//!
//! This implementation is returned as the global Meter if no `Meter`
//! has been set. It is also useful for testing purposes as it is intended
//! to have minimal resource utilization and runtime impact.
use crate::api::{
    metrics::{
        sdk_api::{
            AsyncInstrumentCore, InstrumentCore, MeterCore, SyncBoundInstrumentCore,
            SyncInstrumentCore,
        },
        AsyncRunner, Descriptor, InstrumentKind, Measurement, Meter, MeterProvider, Number,
        NumberKind, Result,
    },
    Context, KeyValue,
};
use std::any::Any;
use std::sync::Arc;

lazy_static::lazy_static! {
    static ref NOOP_DESCRIPTOR: Descriptor = Descriptor::new(String::new(), "noop".to_string(), InstrumentKind::Counter, NumberKind::U64);
}

/// A no-op instance of a `MetricProvider`
#[derive(Debug)]
pub struct NoopMeterProvider;
impl MeterProvider for NoopMeterProvider {
    fn meter(&self, name: &str) -> Meter {
        Meter::new(name, Arc::new(NoopMeterCore))
    }
}

/// A no-op instance of a `Meter`
#[derive(Debug)]
pub struct NoopMeterCore;

impl MeterCore for NoopMeterCore {
    fn new_sync_instrument(
        &self,
        _descriptor: Descriptor,
    ) -> Result<Arc<dyn SyncInstrumentCore + Send + Sync>> {
        Ok(Arc::new(NoopSyncInstrument))
    }

    fn new_async_instrument(
        &self,
        _descriptor: Descriptor,
        _runner: AsyncRunner,
    ) -> Result<Arc<dyn AsyncInstrumentCore + Send + Sync>> {
        Ok(Arc::new(NoopAsyncInstrument))
    }

    fn record_batch_with_context(
        &self,
        _cx: &Context,
        _labels: &[KeyValue],
        _measurements: Vec<Measurement>,
    ) {
        // Ignored
    }
}

/// A no-op sync instrument
#[derive(Debug)]
pub struct NoopSyncInstrument;

impl InstrumentCore for NoopSyncInstrument {
    fn descriptor(&self) -> &Descriptor {
        &NOOP_DESCRIPTOR
    }
}

impl SyncInstrumentCore for NoopSyncInstrument {
    fn bind<'a>(
        &self,
        _labels: &'a [crate::api::KeyValue],
    ) -> Arc<dyn SyncBoundInstrumentCore + Send + Sync> {
        Arc::new(NoopBoundSyncInstrument)
    }
    fn record_one_with_context<'a>(
        &self,
        _cx: &crate::api::Context,
        _number: super::Number,
        _labels: &'a [crate::api::KeyValue],
    ) {
        // Ignored
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// A no-op bound sync instrument
#[derive(Debug)]
pub struct NoopBoundSyncInstrument;

impl SyncBoundInstrumentCore for NoopBoundSyncInstrument {
    fn record_one_with_context<'a>(&self, _cx: &Context, _number: Number) {
        // Ignored
    }
}

/// A no-op async instrument.
#[derive(Debug)]
pub struct NoopAsyncInstrument;

impl InstrumentCore for NoopAsyncInstrument {
    fn descriptor(&self) -> &Descriptor {
        &NOOP_DESCRIPTOR
    }
}

impl AsyncInstrumentCore for NoopAsyncInstrument {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
