//! Id Generator
pub(super) mod aws;
#[cfg(feature = "testing")]
pub use increment::IncrementIdGenerator;

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
        CURRENT_RNG.with(|rng| TraceId::from(rng.borrow_mut().gen::<u128>()))
    }

    fn new_span_id(&self) -> SpanId {
        CURRENT_RNG.with(|rng| SpanId::from(rng.borrow_mut().gen::<u64>()))
    }
}

thread_local! {
    /// Store random number generator for each thread
    static CURRENT_RNG: RefCell<rngs::SmallRng> = RefCell::new(rngs::SmallRng::from_entropy());
}

#[cfg(feature = "testing")]
mod increment {
    use crate::trace::IdGenerator;
    use opentelemetry::trace::{SpanId, TraceId};
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;

    /// [`IdGenerator`] implementation that increments a counter for each new ID. This helps produce
    /// predictable IDs for testing.
    #[derive(Clone, Debug)]
    pub struct IncrementIdGenerator(Arc<AtomicU64>);

    impl IncrementIdGenerator {
        /// Create a new [`IncrementIdGenerator`]
        pub fn new() -> Self {
            Self::default()
        }
    }

    impl Default for IncrementIdGenerator {
        fn default() -> Self {
            Self(Arc::new(AtomicU64::new(1)))
        }
    }

    impl IdGenerator for IncrementIdGenerator {
        fn new_trace_id(&self) -> TraceId {
            TraceId::from(self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst) as u128)
        }

        fn new_span_id(&self) -> SpanId {
            SpanId::from(self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
        }
    }
}
