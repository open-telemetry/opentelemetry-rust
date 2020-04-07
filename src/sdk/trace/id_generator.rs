//! Id Generator
use crate::api;
use rand::{rngs, Rng};
use std::cell::RefCell;

/// Generates Trace and Span ids
#[derive(Clone, Debug, Default)]
pub struct IdGenerator {
    _private: (),
}

impl api::IdGenerator for IdGenerator {
    /// Generate new `TraceId` using thread local rng
    fn new_trace_id(&self) -> api::TraceId {
        CURRENT_RNG.with(|rng| api::TraceId::from_u128(rng.borrow_mut().gen()))
    }

    /// Generate new `SpanId` using thread local rng
    fn new_span_id(&self) -> api::SpanId {
        CURRENT_RNG.with(|rng| api::SpanId::from_u64(rng.borrow_mut().gen()))
    }
}

thread_local! {
    /// Store random number generator for each thread
    static CURRENT_RNG: RefCell<rngs::ThreadRng> = RefCell::new(rngs::ThreadRng::default());
}
