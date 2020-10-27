//! Context extensions for tracing
use crate::{Context, ContextGuard};
lazy_static::lazy_static! {
    static ref NOOP_SPAN: crate::trace::NoopSpan = crate::trace::NoopSpan::new();
}

struct Span(Box<dyn crate::trace::Span + Send + Sync>);

struct RemoteSpanContext(crate::trace::SpanContext);

/// Methods for storing and retrieving trace data in a context.
pub trait TraceContextExt {
    /// Returns a clone of the current context with the included span.
    ///
    /// This is useful for building tracers.
    fn current_with_span<T: crate::trace::Span + Send + Sync>(span: T) -> Self;

    /// Returns a clone of this context with the included span.
    ///
    /// This is useful for building tracers.
    fn with_span<T: crate::trace::Span + Send + Sync>(&self, span: T) -> Self;

    /// Returns a reference to this context's span, or the default no-op span if
    /// none has been set.
    ///
    /// # Examples
    ///
    /// ```
    /// use opentelemetry::{
    ///     sdk::trace as sdktrace,
    ///     trace::{SpanContext, TraceContextExt, Tracer, TracerProvider},
    ///     Context,
    /// };
    ///
    /// // returns a reference to an empty span by default
    /// assert_eq!(Context::current().span().span_context(), SpanContext::empty_context());
    ///
    /// sdktrace::TracerProvider::default().get_tracer("my-component", None).in_span("my-span", |cx| {
    ///     // Returns a reference to the current span if set
    ///     assert_ne!(cx.span().span_context(), SpanContext::empty_context());
    /// });
    /// ```
    fn span(&self) -> &dyn crate::trace::Span;

    /// Returns a copy of this context with the span context included.
    ///
    /// This is useful for building propagators.
    fn with_remote_span_context(&self, span_context: crate::trace::SpanContext) -> Self;

    /// Returns a reference to the remote span context data stored in this context,
    /// or none if no remote span context has been set.
    ///
    /// This is useful for building tracers.
    fn remote_span_context(&self) -> Option<&crate::trace::SpanContext>;
}

impl TraceContextExt for Context {
    fn current_with_span<T: crate::trace::Span + Send + Sync>(span: T) -> Self {
        Context::current_with_value(Span(Box::new(span)))
    }

    fn with_span<T: crate::trace::Span + Send + Sync>(&self, span: T) -> Self {
        self.with_value(Span(Box::new(span)))
    }

    fn span(&self) -> &dyn crate::trace::Span {
        if let Some(span) = self.get::<Span>() {
            span.0.as_ref()
        } else {
            &*NOOP_SPAN
        }
    }

    fn with_remote_span_context(&self, span_context: crate::trace::SpanContext) -> Self {
        self.with_value(RemoteSpanContext(span_context))
    }

    fn remote_span_context(&self) -> Option<&crate::trace::SpanContext> {
        self.get::<RemoteSpanContext>()
            .map(|span_context| &span_context.0)
    }
}

/// Mark a given `Span` as active.
///
/// The `Tracer` MUST provide a way to update its active `Span`, and MAY provide convenience
/// methods to manage a `Span`'s lifetime and the scope in which a `Span` is active. When an
/// active `Span` is made inactive, the previously-active `Span` SHOULD be made active. A `Span`
/// maybe finished (i.e. have a non-null end time) but still be active. A `Span` may be active
/// on one thread after it has been made inactive on another.
///
/// # Examples
///
/// ```
/// use opentelemetry::{global, trace::{Span, Tracer}, KeyValue};
/// use opentelemetry::trace::{get_active_span, mark_span_as_active};
///
/// fn my_function() {
///     let tracer = global::tracer("my-component-a");
///     // start an active span in one function
///     let span = tracer.start("span-name");
///     let _guard = mark_span_as_active(span);
///     // anything happening in functions we call can still access the active span...
///     my_other_function();
/// }
///
/// fn my_other_function() {
///     // call methods on the current span from
///     get_active_span(|span| {
///         span.add_event("An event!".to_string(), vec![KeyValue::new("happened", true)]);
///     });
/// }
/// ```
#[must_use = "Dropping the guard detaches the context."]
pub fn mark_span_as_active<T: crate::trace::Span + Send + Sync>(span: T) -> ContextGuard {
    let cx = Context::current_with_span(span);
    cx.attach()
}

/// Executes a closure with a reference to this thread's current span.
///
/// # Examples
///
/// ```
/// use opentelemetry::{global, trace::{Span, Tracer}, KeyValue};
/// use opentelemetry::trace::get_active_span;
///
/// fn my_function() {
///     // start an active span in one function
///     global::tracer("my-component").in_span("span-name", |_cx| {
///         // anything happening in functions we call can still access the active span...
///         my_other_function();
///     })
/// }
///
/// fn my_other_function() {
///     // call methods on the current span from
///     get_active_span(|span| {
///         span.add_event("An event!".to_string(), vec![KeyValue::new("happened", true)]);
///     })
/// }
/// ```
pub fn get_active_span<F, T>(f: F) -> T
where
    F: FnOnce(&dyn crate::trace::Span) -> T,
{
    f(Context::current().span())
}
