use opentelemetry::trace::{SpanId, TraceId};
use rand::{rngs, Rng, SeedableRng};
use std::cell::RefCell;
use std::fmt;

/// Interface for generating IDs
pub trait IdGenerator: Send + Sync + fmt::Debug {
    /// Generate a new `TraceId`
    fn new_trace_id(&self) -> TraceId;

    /// Generate a new `SpanId`
    fn new_span_id(&self) -> SpanId;

    /// Returns `true` if every trace ID returned by [`new_trace_id`] has at least its
    /// right-most 7 bytes generated randomly or pseudo-randomly with uniform
    /// distribution, as required by
    /// [W3C Trace Context Level 2](https://www.w3.org/TR/trace-context-2/#random-trace-id-flag).
    ///
    /// When this returns `true`, the SDK sets [`TraceFlags::RANDOM`] on the span
    /// context of every root span. The flag is a positive assertion about the
    /// trace ID, so this defaults to `false` and implementations must opt in explicitly.
    ///
    /// [`new_trace_id`]: IdGenerator::new_trace_id
    /// [`TraceFlags::RANDOM`]: opentelemetry::trace::TraceFlags::RANDOM
    fn is_random(&self) -> bool {
        false
    }
}

/// Default [`IdGenerator`] implementation.
///
/// Generates Trace and Span ids using a random number generator.
#[derive(Clone, Debug, Default)]
pub struct RandomIdGenerator {
    _private: (),
}

impl IdGenerator for RandomIdGenerator {
    fn new_trace_id(&self) -> TraceId {
        CURRENT_RNG.with(|rng| TraceId::from(rng.borrow_mut().random::<u128>()))
    }

    fn new_span_id(&self) -> SpanId {
        CURRENT_RNG.with(|rng| SpanId::from(rng.borrow_mut().random::<u64>()))
    }

    fn is_random(&self) -> bool {
        true
    }
}

#[cfg(all(test, feature = "testing", feature = "trace"))]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct CounterIdGenerator;

    impl IdGenerator for CounterIdGenerator {
        fn new_trace_id(&self) -> TraceId {
            TraceId::from(1)
        }

        fn new_span_id(&self) -> SpanId {
            SpanId::from(1)
        }
    }

    #[test]
    fn id_generator_is_random_defaults_to_false() {
        assert!(!CounterIdGenerator.is_random());
    }

    #[test]
    fn random_id_generator_is_random() {
        assert!(RandomIdGenerator::default().is_random());
    }
}

thread_local! {
    /// Store random number generator for each thread
    static CURRENT_RNG: RefCell<rngs::SmallRng> = RefCell::new(rngs::SmallRng::from_os_rng());
}
