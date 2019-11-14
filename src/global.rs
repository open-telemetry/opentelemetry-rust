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
    fn add_link(&mut self, link: api::SpanContext) {
        self.0.add_link(link)
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
    fn set_status(&mut self, status: String) {
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
}

/// Boxed Tracer allows `GlobalTracer`'s to contain and use a `Tracer` type object.
pub trait BoxedTracer: Send + Sync {
    /// Create a new invalid span for use in cases where there are no active spans.
    fn invalid_boxed(&self) -> Box<dyn api::Span>;

    /// Returns a trait object so the underlying implementation can be swapped
    /// out at runtime.
    fn start_boxed(
        &self,
        name: &'static str,
        parent: Option<api::SpanContext>,
    ) -> Box<dyn api::Span>;

    /// Returns the currently active span as a BoxedSpan
    fn get_active_span_boxed(&self) -> Box<dyn api::Span>;

    /// Returns the currently active span as a BoxedSpan
    fn mark_span_as_active_boxed(&self, span: &dyn api::Span);

    /// Marks the current span as inactive
    fn mark_span_as_inactive_boxed(&self, span_id: u64);
}

impl<S: api::Span + 'static> BoxedTracer for Box<dyn api::Tracer<Span = S>> {
    /// Create a new invalid span for use in cases where there are no active spans.
    fn invalid_boxed(&self) -> Box<dyn api::Span> {
        Box::new(self.invalid())
    }

    /// Returns a trait object so the underlying implementation can be swapped
    /// out at runtime.
    fn start_boxed(
        &self,
        name: &'static str,
        parent: Option<api::SpanContext>,
    ) -> Box<dyn api::Span> {
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
}

impl Tracer for dyn BoxedTracer {
    /// BoxedTracer returns a BoxedSpan so that it doesn't need a generic type parameter.
    type Span = BoxedSpan;

    /// Returns an invalid boxed span
    fn invalid(&self) -> Self::Span {
        BoxedSpan(self.invalid_boxed())
    }

    /// Starts a new boxed span.
    fn start(&self, name: &'static str, parent_span: Option<api::SpanContext>) -> Self::Span {
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
}

/// GlobalTracer maintains a global singleton that allows any thread to access
/// the same generic `Tracer` implementation.
#[allow(missing_debug_implementations)]
pub struct GlobalTracer {
    tracer: Box<dyn BoxedTracer>,
}

impl GlobalTracer {
    /// Create a new global tracer via any `Tracer`.
    fn new<S: api::Span + 'static>(tracer: Box<dyn api::Tracer<Span = S>>) -> Self {
        Self {
            tracer: Box::new(tracer),
        }
    }
}

impl api::Tracer for GlobalTracer {
    /// Global tracer uses `BoxedSpan`s so that it can be a global singleton,
    /// which is not possible if it takes generic type parameters.
    type Span = BoxedSpan;

    /// Returns a span with an invalid `SpanContext`.
    fn invalid(&self) -> Self::Span {
        self.tracer.invalid()
    }

    /// Starts a new `Span`.
    fn start(&self, name: &'static str, parent_span: Option<api::SpanContext>) -> Self::Span {
        self.tracer.start(name, parent_span)
    }

    /// Returns the current active span.
    fn get_active_span(&self) -> Self::Span {
        self.tracer.get_active_span()
    }

    /// Mark a given `Span` as active.
    fn mark_span_as_active(&self, span: &Self::Span) {
        self.tracer.mark_span_as_active(span)
    }

    /// Mark a given `Span` as inactive.
    fn mark_span_as_inactive(&self, span_id: u64) {
        self.tracer.mark_span_as_inactive(span_id)
    }
}

lazy_static::lazy_static! {
    /// The global `Tracer` singleton.
    static ref GLOBAL_TRACER: RwLock<Arc<GlobalTracer>> = RwLock::new(Arc::new(GlobalTracer::new(Box::new(api::NoopTracer {}))));
}

/// Returns a reference to the global `Tracer`
pub fn global_tracer() -> Arc<GlobalTracer> {
    GLOBAL_TRACER
        .read()
        .expect("GLOBAL_TRACER RwLock poisoned")
        .clone()
}

/// Assigns the global `Tracer`
pub fn set_tracer<S: api::Span + 'static>(new_tracer: Box<dyn api::Tracer<Span = S>>) {
    let mut global_tracer = GLOBAL_TRACER
        .write()
        .expect("GLOBAL_TRACER RwLock poisoned");
    *global_tracer = Arc::new(GlobalTracer::new(new_tracer));
}

/// Returns `NoopMeter` for now
pub fn global_meter() -> crate::api::NoopMeter {
    crate::api::NoopMeter {}
}
