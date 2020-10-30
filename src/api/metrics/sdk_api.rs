//! Metrics SDK API
use crate::metrics::{AsyncRunner, Descriptor, Measurement, Number, Result};
use crate::{Context, KeyValue};
use std::any::Any;
use std::fmt;
use std::sync::Arc;

/// The interface an SDK must implement to supply a Meter implementation.
pub trait MeterCore: fmt::Debug {
    /// Atomically record a batch of measurements.
    fn record_batch_with_context(
        &self,
        cx: &Context,
        labels: &[KeyValue],
        measurements: Vec<Measurement>,
    );

    /// Create a new synchronous instrument implementation.
    fn new_sync_instrument(
        &self,
        descriptor: Descriptor,
    ) -> Result<Arc<dyn SyncInstrumentCore + Send + Sync>>;

    /// Create a new asynchronous instrument implementation.
    fn new_async_instrument(
        &self,
        descriptor: Descriptor,
        runner: AsyncRunner,
    ) -> Result<Arc<dyn AsyncInstrumentCore + Send + Sync>>;
}

/// A common interface for synchronous and asynchronous instruments.
pub trait InstrumentCore: fmt::Debug {
    /// Description of the instrument's descriptor
    fn descriptor(&self) -> &Descriptor;
}

/// The implementation-level interface to a generic synchronous instrument
/// (e.g., ValueRecorder and Counter instruments).
pub trait SyncInstrumentCore: InstrumentCore + Send + Sync {
    /// Creates an implementation-level bound instrument, binding a label set
    /// with this instrument implementation.
    fn bind(&self, labels: &'_ [KeyValue]) -> Arc<dyn SyncBoundInstrumentCore + Send + Sync>;

    /// Capture a single synchronous metric event.
    fn record_one(&self, number: Number, labels: &'_ [KeyValue]);

    /// Returns self as any
    fn as_any(&self) -> &dyn Any;
}

/// The implementation-level interface to a generic synchronous bound instrument
pub trait SyncBoundInstrumentCore: fmt::Debug + Send + Sync {
    /// Capture a single synchronous metric event.
    fn record_one(&self, number: Number);
}

/// An implementation-level interface to an asynchronous instrument (e.g.,
/// Observer instruments).
pub trait AsyncInstrumentCore: InstrumentCore {
    /// The underlying type as `Any` to support downcasting.
    fn as_any(&self) -> &dyn Any;
}
