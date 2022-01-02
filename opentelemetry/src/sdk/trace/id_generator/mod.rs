//! Id Generator
pub(super) mod aws;

use crate::trace::{SpanId, TraceId};
use rand::{rngs, Rng};
use std::cell::RefCell;

/// Default [`crate::trace::IdGenerator`] implementation.
/// Generates Trace and Span ids using a random number generator.
#[derive(Clone, Debug, Default)]
pub struct IdGenerator {
    _private: (),
}

impl crate::trace::IdGenerator for IdGenerator {
    /// Generate new `TraceId` using thread local rng
    fn new_trace_id(&self) -> TraceId {
        CURRENT_RNG.with(|rng| TraceId::from(rng.borrow_mut().gen::<[u8; 16]>()))
    }

    /// Generate new `SpanId` using thread local rng
    fn new_span_id(&self) -> SpanId {
        CURRENT_RNG.with(|rng| SpanId::from(rng.borrow_mut().gen::<[u8; 8]>()))
    }
}

thread_local! {
    /// Store random number generator for each thread
    static CURRENT_RNG: RefCell<rngs::ThreadRng> = RefCell::new(rngs::ThreadRng::default());
}
