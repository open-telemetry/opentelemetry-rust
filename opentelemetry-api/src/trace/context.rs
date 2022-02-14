//! Context extensions for tracing
use crate::{
    global,
    trace::{Span, SpanContext},
    Context, ContextGuard, KeyValue,
};
use futures_util::{sink::Sink, stream::Stream};
use pin_project::pin_project;
use std::error::Error;
use std::sync::Mutex;
use std::{
    borrow::Cow,
    pin::Pin,
    task::{Context as TaskContext, Poll},
};

lazy_static::lazy_static! {
    static ref NOOP_SPAN: SynchronizedSpan = SynchronizedSpan {
        span_context: SpanContext::empty_context(),
        inner: None,
    };
}

/// A reference to the currently active span in this context.
#[derive(Debug)]
pub struct SpanRef<'a>(&'a SynchronizedSpan);

#[derive(Debug)]
struct SynchronizedSpan {
    /// Immutable span context
    span_context: SpanContext,
    /// Mutable span inner that requires synchronization
    inner: Option<Mutex<global::BoxedSpan>>,
}

impl SpanRef<'_> {
    fn with_inner_mut<F: FnOnce(&mut global::BoxedSpan)>(&self, f: F) {
        if let Some(ref inner) = self.0.inner {
            match inner.lock() {
                Ok(mut locked) => f(&mut *locked),
                Err(err) => global::handle_error(err),
            }
        }
    }
}

impl SpanRef<'_> {
    /// An API to record events in the context of a given `Span`.
    pub fn add_event<T>(&self, name: T, attributes: Vec<KeyValue>)
    where
        T: Into<Cow<'static, str>>,
    {
        self.with_inner_mut(|inner| inner.add_event(name, attributes))
    }

    /// Convenience method to record an exception/error as an `Event`
    pub fn record_exception(&self, err: &dyn Error) {
        self.with_inner_mut(|inner| inner.record_exception(err))
    }

    /// Convenience method to record a exception/error as an `Event` with custom stacktrace
    pub fn record_exception_with_stacktrace<T>(&self, err: &dyn Error, stacktrace: T)
    where
        T: Into<Cow<'static, str>>,
    {
        self.with_inner_mut(|inner| inner.record_exception_with_stacktrace(err, stacktrace))
    }

    /// An API to record events at a specific time in the context of a given `Span`.
    pub fn add_event_with_timestamp<T>(
        &self,
        name: T,
        timestamp: std::time::SystemTime,
        attributes: Vec<crate::KeyValue>,
    ) where
        T: Into<Cow<'static, str>>,
    {
        self.with_inner_mut(move |inner| {
            inner.add_event_with_timestamp(name, timestamp, attributes)
        })
    }

    /// Returns the `SpanContext` for the given `Span`.
    pub fn span_context(&self) -> &SpanContext {
        &self.0.span_context
    }

    /// Returns true if this `Span` is recording information like events with the `add_event`
    /// operation, attributes using `set_attributes`, status with `set_status`, etc.
    pub fn is_recording(&self) -> bool {
        self.0
            .inner
            .as_ref()
            .and_then(|inner| inner.lock().ok().map(|active| active.is_recording()))
            .unwrap_or(false)
    }

    /// An API to set a single `Attribute` where the attribute properties are passed
    /// as arguments. To avoid extra allocations some implementations may offer a separate API for
    /// each of the possible value types.
    pub fn set_attribute(&self, attribute: crate::KeyValue) {
        self.with_inner_mut(move |inner| inner.set_attribute(attribute))
    }

    /// Sets the status of the `Span`. If used, this will override the default `Span`
    /// status, which is `Unset`. `message` MUST be ignored when the status is `OK` or `Unset`
    pub fn set_status<T>(&self, code: super::StatusCode, message: T)
    where
        T: Into<Cow<'static, str>>,
    {
        self.with_inner_mut(move |inner| inner.set_status(code, message))
    }

    /// Updates the `Span`'s name. After this update, any sampling behavior based on the
    /// name will depend on the implementation.
    pub fn update_name<T>(&self, new_name: T)
    where
        T: Into<Cow<'static, str>>,
    {
        self.with_inner_mut(move |inner| inner.update_name(new_name))
    }

    /// Finishes the `Span`.
    pub fn end(&self) {
        self.end_with_timestamp(crate::time::now());
    }

    /// Finishes the `Span` with given timestamp
    pub fn end_with_timestamp(&self, timestamp: std::time::SystemTime) {
        self.with_inner_mut(move |inner| inner.end_with_timestamp(timestamp))
    }
}

/// Methods for storing and retrieving trace data in a context.
pub trait TraceContextExt {
    /// Returns a clone of the current context with the included span.
    ///
    /// This is useful for building tracers.
    fn current_with_span<T: crate::trace::Span + Send + Sync + 'static>(span: T) -> Self;

    /// Returns a clone of this context with the included span.
    ///
    /// This is useful for building tracers.
    fn with_span<T: crate::trace::Span + Send + Sync + 'static>(&self, span: T) -> Self;

    /// Returns a reference to this context's span, or the default no-op span if
    /// none has been set.
    fn span(&self) -> SpanRef<'_>;

    /// Used to see if a span has been marked as active
    ///
    /// This is useful for building tracers.
    fn has_active_span(&self) -> bool;

    /// Returns a copy of this context with the span context included.
    ///
    /// This is useful for building propagators.
    fn with_remote_span_context(&self, span_context: crate::trace::SpanContext) -> Self;
}

impl TraceContextExt for Context {
    fn current_with_span<T: crate::trace::Span + Send + Sync + 'static>(span: T) -> Self {
        Context::current_with_value(SynchronizedSpan {
            span_context: span.span_context().clone(),
            inner: Some(Mutex::new(global::BoxedSpan::new(span))),
        })
    }

    fn with_span<T: crate::trace::Span + Send + Sync + 'static>(&self, span: T) -> Self {
        self.with_value(SynchronizedSpan {
            span_context: span.span_context().clone(),
            inner: Some(Mutex::new(global::BoxedSpan::new(span))),
        })
    }

    fn span(&self) -> SpanRef<'_> {
        if let Some(span) = self.get::<SynchronizedSpan>() {
            SpanRef(span)
        } else {
            SpanRef(&*NOOP_SPAN)
        }
    }

    fn has_active_span(&self) -> bool {
        self.get::<SynchronizedSpan>().is_some()
    }

    fn with_remote_span_context(&self, span_context: crate::trace::SpanContext) -> Self {
        self.with_value(SynchronizedSpan {
            span_context,
            inner: None,
        })
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
/// use opentelemetry_api::{global, trace::{Span, Tracer}, KeyValue};
/// use opentelemetry_api::trace::{get_active_span, mark_span_as_active};
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
pub fn mark_span_as_active<T: crate::trace::Span + Send + Sync + 'static>(span: T) -> ContextGuard {
    let cx = Context::current_with_span(span);
    cx.attach()
}

/// Executes a closure with a reference to this thread's current span.
///
/// # Examples
///
/// ```
/// use opentelemetry_api::{global, trace::{Span, Tracer}, KeyValue};
/// use opentelemetry_api::trace::get_active_span;
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
    F: FnOnce(SpanRef<'_>) -> T,
{
    f(Context::current().span())
}

/// A future, stream, or sink that has an associated context.
#[pin_project]
#[derive(Clone, Debug)]
pub struct WithContext<T> {
    #[pin]
    inner: T,
    otel_cx: Context,
}

impl<T: Sized> FutureExt for T {}

impl<T: std::future::Future> std::future::Future for WithContext<T> {
    type Output = T::Output;

    fn poll(self: Pin<&mut Self>, task_cx: &mut TaskContext<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let _guard = this.otel_cx.clone().attach();

        this.inner.poll(task_cx)
    }
}

impl<T: Stream> Stream for WithContext<T> {
    type Item = T::Item;

    fn poll_next(self: Pin<&mut Self>, task_cx: &mut TaskContext<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let _guard = this.otel_cx.clone().attach();
        T::poll_next(this.inner, task_cx)
    }
}

impl<I, T: Sink<I>> Sink<I> for WithContext<T>
where
    T: Sink<I>,
{
    type Error = T::Error;

    fn poll_ready(
        self: Pin<&mut Self>,
        task_cx: &mut TaskContext<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        let _guard = this.otel_cx.clone().attach();
        T::poll_ready(this.inner, task_cx)
    }

    fn start_send(self: Pin<&mut Self>, item: I) -> Result<(), Self::Error> {
        let this = self.project();
        let _guard = this.otel_cx.clone().attach();
        T::start_send(this.inner, item)
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        task_cx: &mut TaskContext<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        let _guard = this.otel_cx.clone().attach();
        T::poll_flush(this.inner, task_cx)
    }

    fn poll_close(
        self: Pin<&mut Self>,
        task_cx: &mut TaskContext<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        let _enter = this.otel_cx.clone().attach();
        T::poll_close(this.inner, task_cx)
    }
}

/// Extension trait allowing futures, streams, and sinks to be traced with a span.
pub trait FutureExt: Sized {
    /// Attaches the provided [`Context`] to this type, returning a `WithContext`
    /// wrapper.
    ///
    /// When the wrapped type is a future, stream, or sink, the attached context
    /// will be set as current while it is being polled.
    ///
    /// [`Context`]: crate::Context
    fn with_context(self, otel_cx: Context) -> WithContext<Self> {
        WithContext {
            inner: self,
            otel_cx,
        }
    }

    /// Attaches the current [`Context`] to this type, returning a `WithContext`
    /// wrapper.
    ///
    /// When the wrapped type is a future, stream, or sink, the attached context
    /// will be set as the default while it is being polled.
    ///
    /// [`Context`]: crate::Context
    fn with_current_context(self) -> WithContext<Self> {
        let otel_cx = Context::current();
        self.with_context(otel_cx)
    }
}
