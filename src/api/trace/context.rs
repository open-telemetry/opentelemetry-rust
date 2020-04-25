//! Context extensions for tracing
use crate::api;

lazy_static::lazy_static! {
    static ref NOOP_SPAN: api::NoopSpan = api::NoopSpan::new();
}

struct Span(Box<dyn api::Span>);
struct RemoteSpanContext(api::SpanContext);

/// Methods for storing and retrieving trace data in a context.
pub trait TraceContextExt {
    /// Returns a clone of the current context with the included span.
    ///
    /// This is useful for building tracers.
    fn current_with_span<T: api::Span>(span: T) -> Self;

    /// Returns a clone of this context with the included span.
    ///
    /// This is useful for building tracers.
    fn with_span<T: api::Span>(&self, span: T) -> Self;

    /// Returns a reference to this context's span, or the default no-op span if
    /// none has been set.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::{api, api::{Context, Provider, TraceContextExt, Tracer}, sdk};
    ///
    /// // returns a reference to an empty span by default
    /// assert_eq!(Context::current().span().span_context(), api::SpanContext::empty_context());
    ///
    /// sdk::Provider::default().get_tracer("my-component").in_span("my-span", |cx| {
    ///     // Returns a reference to the current span if set
    ///     assert_ne!(cx.span().span_context(), api::SpanContext::empty_context());
    /// });
    /// ```
    fn span(&self) -> &dyn api::Span;

    /// Returns a copy of the current context with the span context included.
    ///
    /// This is useful for building propagators.
    fn current_with_remote_span_context(span_context: api::SpanContext) -> Self;

    /// Returns a reference to the remote span context data stored in this context,
    /// or none if no remote span context has been set.
    ///
    /// This is useful for building tracers.
    fn remote_span_context(&self) -> Option<&api::SpanContext>;
}

impl TraceContextExt for api::Context {
    fn current_with_span<T: api::Span>(span: T) -> Self {
        api::Context::current_with_value(Span(Box::new(span)))
    }

    fn with_span<T: api::Span>(&self, span: T) -> Self {
        self.with_value(Span(Box::new(span)))
    }

    fn span(&self) -> &dyn api::Span {
        if let Some(span) = self.get::<Span>() {
            span.0.as_ref()
        } else {
            &*NOOP_SPAN
        }
    }

    fn current_with_remote_span_context(span_context: api::SpanContext) -> Self {
        api::Context::current_with_value(RemoteSpanContext(span_context))
    }

    fn remote_span_context(&self) -> Option<&api::SpanContext> {
        self.get::<RemoteSpanContext>()
            .map(|span_context| &span_context.0)
    }
}
