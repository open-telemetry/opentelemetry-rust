//! SDK API

// mod async_instrument;
mod descriptor;
mod instrument_kind;
mod number;
mod wrap;
// mod sync_instrument;

use std::any::Any;
use std::sync::Arc;

pub use descriptor::*;
pub use instrument_kind::*;
pub use number::*;
use opentelemetry_api::{metrics::Result, Context, KeyValue};
pub use wrap::wrap_meter_core;

/// The interface an SDK must implement to supply a Meter implementation.
pub trait MeterCore {
    /// Create a new synchronous instrument implementation.
    fn new_sync_instrument(
        &self,
        descriptor: Descriptor,
    ) -> Result<Arc<dyn SyncInstrumentCore + Send + Sync>>;

    /// Create a new asynchronous instrument implementation.
    ///
    /// Runner is `None` if used in batch as the batch runner is registered separately.
    fn new_async_instrument(
        &self,
        descriptor: Descriptor,
    ) -> Result<Arc<dyn AsyncInstrumentCore + Send + Sync>>;

    /// Register a batch observer
    fn register_callback(&self, f: Box<dyn Fn(&Context) + Send + Sync>) -> Result<()>;
}

/// A utility extension to allow upcasting.
///
/// Can be removed once [trait_upcasting] is stablized.
///
/// [trait_upcasting]: https://doc.rust-lang.org/unstable-book/language-features/trait-upcasting.html
pub trait AsDynInstrumentCore {
    /// Create an `Arc<dyn InstrumentCore>` from an impl of `InstrumentCore`.
    fn as_dyn_core<'a>(self: Arc<Self>) -> Arc<dyn InstrumentCore + Send + Sync + 'a>
    where
        Self: 'a;
}

impl<T: InstrumentCore + Sized + Send + Sync> AsDynInstrumentCore for T {
    fn as_dyn_core<'a>(self: Arc<Self>) -> Arc<dyn InstrumentCore + Send + Sync + 'a>
    where
        Self: 'a,
    {
        self
    }
}

/// A common interface for synchronous and asynchronous instruments.
pub trait InstrumentCore: AsDynInstrumentCore {
    /// Description of the instrument's descriptor
    fn descriptor(&self) -> &Descriptor;

    /// Returns self as any
    fn as_any(&self) -> &dyn Any;
}

/// The implementation-level interface to a generic synchronous instrument
/// (e.g., Histogram and Counter instruments).
pub trait SyncInstrumentCore: InstrumentCore {
    /// Capture a single synchronous metric event.
    fn record_one(&self, cx: &Context, number: Number, attributes: &'_ [KeyValue]);
}

/// An implementation-level interface to an asynchronous instrument (e.g.,
/// Observable instruments).
pub trait AsyncInstrumentCore: InstrumentCore {
    /// Captures a single asynchronous metric event.
    fn observe_one(&self, cx: &Context, number: Number, attributes: &'_ [KeyValue]);
}
