//! Abstracts away details for acquiring a `Tracer` by instrumented libraries.
use once_cell::sync::OnceCell;
use opentelemetry::global::BoxedTracer;
use std::fmt::Debug;

/// Holds either a borrowed `BoxedTracer` or a factory that can produce one when
/// and if needed.
///
/// This unifies handling of obtaining a `Tracer` by library code optimizing for
/// common cases when it will never be needed.
#[derive(Debug)]
pub struct TracerSource<'a> {
    variant: Variant<'a>,
    tracer: OnceCell<BoxedTracer>,
}

enum Variant<'a> {
    Borrowed(&'a BoxedTracer),
    Lazy(&'a dyn Fn() -> BoxedTracer),
}

impl<'a> TracerSource<'a> {
    /// Construct an instance by borrowing the specified `BoxedTracer`.
    pub fn borrowed(tracer: &'a BoxedTracer) -> Self {
        Self {
            variant: Variant::Borrowed(tracer),
            tracer: OnceCell::new(),
        }
    }

    /// Construct an instance which may lazily produce a `BoxedTracer` using
    /// the specified factory function.
    pub fn lazy(factory: &'a dyn Fn() -> BoxedTracer) -> Self {
        Self {
            variant: Variant::Lazy(factory),
            tracer: OnceCell::new(),
        }
    }

    /// Get the associated `BoxedTracer`, producing it if necessary.
    pub fn get(&self) -> &BoxedTracer {
        use Variant::*;
        match self.variant {
            Borrowed(tracer) => tracer,
            Lazy(factory) => self.tracer.get_or_init(factory),
        }
    }
}

impl<'a> Debug for Variant<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Variant::*;
        match self {
            Borrowed(arg0) => f.debug_tuple("Borrowed").field(arg0).finish(),
            Lazy(_arg0) => f.debug_tuple("Lazy").finish(),
        }
    }
}
