//! Context extensions for tracing
use crate::api;

lazy_static::lazy_static! {
    static ref NOOP_SPAN: api::trace::NoopSpan = api::trace::NoopSpan::new();
}

struct Span(Box<dyn api::trace::Span + Send + Sync>);
struct RemoteSpanReference(api::trace::SpanReference);

/// Methods for storing and retrieving trace data in a context.
pub trait TraceContextExt {
    /// Returns a clone of the current context with the included span.
    ///
    /// This is useful for building tracers.
    fn current_with_span<T: api::trace::Span + Send + Sync>(span: T) -> Self;

    /// Returns a clone of this context with the included span.
    ///
    /// This is useful for building tracers.
    fn with_span<T: api::trace::Span + Send + Sync>(&self, span: T) -> Self;

    /// Returns a reference to this context's span, or the default no-op span if
    /// none has been set.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::{api, api::{Context, trace::{TracerProvider, TraceContextExt, Tracer}}, sdk::trace as sdktrace};
    ///
    /// // returns a reference to an empty span by default
    /// assert_eq!(Context::current().span().span_reference(), api::trace::SpanReference::empty_context());
    ///
    /// sdktrace::TracerProvider::default().get_tracer("my-component", None).in_span("my-span", |cx| {
    ///     // Returns a reference to the current span if set
    ///     assert_ne!(cx.span().span_reference(), api::trace::SpanReference::empty_context());
    /// });
    /// ```
    fn span(&self) -> &dyn api::trace::Span;

    /// Returns a copy of this context with the span context included.
    ///
    /// This is useful for building propagators.
    fn with_remote_span_reference(&self, span_reference: api::trace::SpanReference) -> Self;

    /// Returns a reference to the remote span context data stored in this context,
    /// or none if no remote span context has been set.
    ///
    /// This is useful for building tracers.
    fn remote_span_reference(&self) -> Option<&api::trace::SpanReference>;
}

impl TraceContextExt for api::Context {
    fn current_with_span<T: api::trace::Span + Send + Sync>(span: T) -> Self {
        api::Context::current_with_value(Span(Box::new(span)))
    }

    fn with_span<T: api::trace::Span + Send + Sync>(&self, span: T) -> Self {
        self.with_value(Span(Box::new(span)))
    }

    fn span(&self) -> &dyn api::trace::Span {
        if let Some(span) = self.get::<Span>() {
            span.0.as_ref()
        } else {
            &*NOOP_SPAN
        }
    }

    fn with_remote_span_reference(&self, span_reference: api::trace::SpanReference) -> Self {
        self.with_value(RemoteSpanReference(span_reference))
    }

    fn remote_span_reference(&self) -> Option<&api::trace::SpanReference> {
        self.get::<RemoteSpanReference>()
            .map(|span_reference| &span_reference.0)
    }
}
