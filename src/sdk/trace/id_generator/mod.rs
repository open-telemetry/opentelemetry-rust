//! Id Generator
pub(super) mod aws;

use crate::api;
use rand::{rngs, Rng};
use std::cell::RefCell;

/// Default [api::IdGenerator] implementation.
/// Generates Trace and Span ids using a random number generator.
#[derive(Clone, Debug, Default)]
pub struct IdGenerator {
    _private: (),
}

impl api::trace::IdGenerator for IdGenerator {
    /// Generate new `TraceId` using thread local rng
    fn new_trace_id(&self) -> api::trace::TraceId {
        CURRENT_RNG.with(|rng| api::trace::TraceId::from_u128(rng.borrow_mut().gen()))
    }

    /// Generate new `SpanId` using thread local rng
    fn new_span_id(&self) -> api::trace::SpanId {
        CURRENT_RNG.with(|rng| api::trace::SpanId::from_u64(rng.borrow_mut().gen()))
    }
}

thread_local! {
    /// Store random number generator for each thread
    static CURRENT_RNG: RefCell<rngs::ThreadRng> = RefCell::new(rngs::ThreadRng::default());
}
