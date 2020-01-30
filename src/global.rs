//! OpenTelemetry global `Tracer` and `Meter` singletons.
use crate::api::{self, KeyValue, SpanContext, Tracer};
use std::any::Any;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

/// Boxed span wraps a generic trait object so that `BoxedTracer`s
/// can return whichever type of span they were configured to use.
#[derive(Debug)]
pub struct BoxedSpan(Box<dyn api::Span>);

impl api::Span for BoxedSpan {
    /// Delegates to inner span.0
    fn add_event_with_timestamp(&mut self, message: String, timestamp: SystemTime) {
        self.0.add_event_with_timestamp(message, timestamp)
    }

    /// Delegates to inner span.
    fn get_context(&self) -> SpanContext {
        self.0.get_context()
    }

    /// Delegates to inner span.
    fn is_recording(&self) -> bool {
        self.0.is_recording()
    }

    /// Delegates to inner span.
    fn set_attribute(&mut self, attribute: KeyValue) {
        self.0.set_attribute(attribute)
    }

    /// Delegates to inner span.
    fn set_status(&mut self, status: api::SpanStatus) {
        self.0.set_status(status)
    }

    /// Delegates to inner span.
    fn update_name(&mut self, new_name: String) {
        self.0.update_name(new_name)
    }

    /// Delegates to inner span.
    fn end(&mut self) {
        self.0.end()
    }

    /// Returns self as any
    fn as_any(&self) -> &dyn Any {
        self
    }

    /// Mark span as currently active
    fn mark_as_active(&self) {
        self.0.mark_as_active()
    }

    /// Mark span as no longer active
    fn mark_as_inactive(&self) {
        self.0.mark_as_inactive()
    }
}

/// `GenericTracer` allows `BoxedTracer`'s to contain and use a `Tracer` trait object.
pub trait GenericTracer: Send + Sync {
    /// Create a new invalid span for use in cases where there are no active spans.
    fn invalid_boxed(&self) -> Box<dyn api::Span>;

    /// Returns a trait object so the underlying implementation can be swapped
    /// out at runtime.
    fn start_boxed(&self, name: &str, parent: Option<api::SpanContext>) -> Box<dyn api::Span>;

    /// Returns the currently active span as a BoxedSpan
    fn get_active_span_boxed(&self) -> Box<dyn api::Span>;

    /// Returns the currently active span as a BoxedSpan
    fn mark_span_as_active_boxed(&self, span: &dyn api::Span);

    /// Marks the current span as inactive
    fn mark_span_as_inactive_boxed(&self, span_id: u64);

    /// Clone span
    fn clone_span_boxed(&self, span: &dyn api::Span) -> Box<dyn api::Span>;
}

impl<S: api::Span + 'static> GenericTracer for Box<dyn api::Tracer<Span = S>> {
    /// Create a new invalid span for use in cases where there are no active spans.
    fn invalid_boxed(&self) -> Box<dyn api::Span> {
        Box::new(self.invalid())
    }

    /// Returns a trait object so the underlying implementation can be swapped
    /// out at runtime.
    fn start_boxed(&self, name: &str, parent: Option<api::SpanContext>) -> Box<dyn api::Span> {
        Box::new(self.start(name, parent))
    }

    /// Returns the current active span.
    fn get_active_span_boxed(&self) -> Box<dyn api::Span> {
        Box::new(self.get_active_span())
    }

    /// Mark span as active.
    fn mark_span_as_active_boxed(&self, some_span: &dyn api::Span) {
        if let Some(span) = some_span.as_any().downcast_ref::<S>() {
            self.mark_span_as_active(span)
        };
    }

    /// Mark span as inactive.
    fn mark_span_as_inactive_boxed(&self, span_id: u64) {
        self.mark_span_as_inactive(span_id)
    }

    /// Clone span
    fn clone_span_boxed(&self, some_span: &dyn api::Span) -> Box<dyn api::Span> {
        if let Some(span) = some_span.as_any().downcast_ref::<S>() {
            Box::new(self.clone_span(span))
        } else {
            self.invalid_boxed()
        }
    }
}

impl Tracer for dyn GenericTracer {
    /// BoxedTracer returns a BoxedSpan so that it doesn't need a generic type parameter.
    type Span = BoxedSpan;

    /// Returns an invalid boxed span
    fn invalid(&self) -> Self::Span {
        BoxedSpan(self.invalid_boxed())
    }

    /// Starts a new boxed span.
    fn start(&self, name: &str, parent_span: Option<api::SpanContext>) -> Self::Span {
        BoxedSpan(self.start_boxed(name, parent_span))
    }

    /// Returns the current active span.
    fn get_active_span(&self) -> Self::Span {
        BoxedSpan(self.get_active_span_boxed())
    }

    /// Marks a given `Span` as active.
    fn mark_span_as_active(&self, span: &Self::Span) {
        self.mark_span_as_active_boxed(&(*span.0))
    }

    /// Marks a given `Span` as inactive.
    fn mark_span_as_inactive(&self, span_id: u64) {
        self.mark_span_as_inactive_boxed(span_id)
    }

    /// Clone span
    fn clone_span(&self, span: &Self::Span) -> Self::Span {
        BoxedSpan(self.clone_span_boxed(&(*span.0)))
    }
}

/// BoxedTracer is an instance of a generic tracer that can be returned by the
/// global provider to represent.
#[allow(missing_debug_implementations)]
pub struct BoxedTracer(Box<dyn GenericTracer>);

impl api::Tracer for BoxedTracer {
    /// Global tracer uses `BoxedSpan`s so that it can be a global singleton,
    /// which is not possible if it takes generic type parameters.
    type Span = BoxedSpan;

    /// Returns a span with an invalid `SpanContext`.
    fn invalid(&self) -> Self::Span {
        self.0.invalid()
    }

    /// Starts a new `Span`.
    fn start(&self, name: &str, parent_span: Option<api::SpanContext>) -> Self::Span {
        self.0.start(name, parent_span)
    }

    /// Returns the current active span.
    fn get_active_span(&self) -> Self::Span {
        self.0.get_active_span()
    }

    /// Mark a given `Span` as active.
    fn mark_span_as_active(&self, span: &Self::Span) {
        self.0.mark_span_as_active(span)
    }

    /// Mark a given `Span` as inactive.
    fn mark_span_as_inactive(&self, span_id: u64) {
        self.0.mark_span_as_inactive(span_id)
    }

    /// Clone span
    fn clone_span(&self, span: &Self::Span) -> Self::Span {
        self.0.clone_span(span)
    }
}

/// `GenericProvider` allows `GlobalProvider`'s to contain and use a `Provider` trait object.
pub trait GenericProvider: Send + Sync {
    /// Creates a named tracer instance that is a trait object through the underlying `Provider`.
    fn get_tracer_boxed(&self, name: &'static str) -> Box<dyn GenericTracer>;
}

impl api::Provider for dyn GenericProvider {
    /// Tracer is a boxed tracer so it can wrap any implementation of `Tracer`.
    type Tracer = BoxedTracer;

    /// Find or create a named instance of `BoxedTracer`.
    fn get_tracer(&self, name: &'static str) -> Self::Tracer {
        BoxedTracer(self.get_tracer_boxed(name))
    }
}

impl<T, S> GenericProvider for Box<dyn api::Provider<Tracer = T>>
where
    S: api::Span + 'static,
    T: api::Tracer<Span = S> + 'static,
{
    /// Return a boxed generic tracer, used
    fn get_tracer_boxed(&self, name: &'static str) -> Box<dyn GenericTracer> {
        // Has to first be boxed to impl `GenericTracer`
        let generic_tracer: Box<dyn api::Tracer<Span = S>> = Box::new(self.get_tracer(name));
        // Then boxed again to impl `Box<dyn GenericTracer>`.
        Box::new(generic_tracer)
    }
}

/// GlobalProvider maintains a global singleton that allows any thread to access
/// the same generic `Provider` implementation.
#[allow(missing_debug_implementations)]
pub struct GlobalProvider {
    provider: Box<dyn GenericProvider>,
}

impl GlobalProvider {
    /// Create a new GlobalProvider instance from a struct that implements `Provider`.
    fn new<P, T, S>(provider: P) -> Self
    where
        S: api::Span + 'static,
        T: api::Tracer<Span = S> + 'static,
        P: api::Provider<Tracer = T> + 'static,
    {
        // Has to first be boxed to impl `GenericProvider`.
        let generic_provider: Box<dyn api::Provider<Tracer = T>> = Box::new(provider);

        // Then boxed again to for `Box<dyn GenericProvider>`.
        GlobalProvider {
            provider: Box::new(generic_provider),
        }
    }
}

impl api::Provider for GlobalProvider {
    type Tracer = BoxedTracer;

    /// Find or create a named tracer using the global provider.
    fn get_tracer(&self, name: &'static str) -> Self::Tracer {
        self.provider.get_tracer(name)
    }
}

lazy_static::lazy_static! {
    /// The global `Tracer` singleton.
    static ref GLOBAL_TRACER_PROVIDER: RwLock<Arc<GlobalProvider>> = RwLock::new(Arc::new(GlobalProvider::new(api::NoopProvider {})));
}

/// Returns a reference to the global `Provider`
pub fn trace_provider() -> Arc<GlobalProvider> {
    GLOBAL_TRACER_PROVIDER
        .read()
        .expect("GLOBAL_TRACER_PROVIDER RwLock poisoned")
        .clone()
}

/// Assigns the global `Tracer`
pub fn set_provider<P, T, S>(new_provider: P)
where
    S: api::Span + 'static,
    T: api::Tracer<Span = S> + 'static,
    P: api::Provider<Tracer = T> + 'static,
{
    let mut global_provider = GLOBAL_TRACER_PROVIDER
        .write()
        .expect("GLOBAL_TRACER_PROVIDER RwLock poisoned");
    *global_provider = Arc::new(GlobalProvider::new(new_provider));
}

/// Returns `NoopMeter` for now
pub fn global_meter() -> crate::api::NoopMeter {
    crate::api::NoopMeter {}
}
