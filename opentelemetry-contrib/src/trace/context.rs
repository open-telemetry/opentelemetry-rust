use super::TracerSource;
use opentelemetry::{
    trace::{SpanBuilder, TraceContextExt as _, Tracer as _},
    Context, ContextGuard,
};
use std::{
    fmt::{Debug, Formatter},
    ops::{Deref, DerefMut},
};

/// Lazily creates a new span only if the current context has an active span,
/// which will used as the new span's parent.
///
/// This is useful for instrumenting library crates whose activities would be
/// undesirable to see as root spans, by themselves, outside of any application
/// context.
///
/// # Examples
///
/// ```
/// use opentelemetry::trace::{SpanBuilder};
/// use opentelemetry_contrib::trace::{new_span_if_parent_sampled, TracerSource};
///
/// fn my_lib_fn() {
///     let _guard = new_span_if_parent_sampled(
///         || SpanBuilder::from_name("my span"),
///         TracerSource::lazy(&|| opentelemetry::global::tracer(module_path!())),
///     )
///     .map(|cx| cx.attach());
/// }
/// ```
pub fn new_span_if_parent_sampled(
    builder_fn: impl Fn() -> SpanBuilder,
    tracer: TracerSource<'_>,
) -> Option<Context> {
    Context::map_current(|current| {
        current.span().span_context().is_sampled().then(|| {
            let builder = builder_fn();
            let span = tracer.get().build_with_context(builder, current);
            current.with_span(span)
        })
    })
}

/// Lazily creates a new span only if the current context has a recording span,
/// which will used as the new span's parent.
///
/// This is useful for instrumenting library crates whose activities would be
/// undesirable to see as root spans, by themselves, outside of any application
/// context.
///
/// # Examples
///
/// ```
/// use opentelemetry::trace::{SpanBuilder};
/// use opentelemetry_contrib::trace::{new_span_if_recording, TracerSource};
///
/// fn my_lib_fn() {
///     let _guard = new_span_if_recording(
///         || SpanBuilder::from_name("my span"),
///         TracerSource::lazy(&|| opentelemetry::global::tracer(module_path!())),
///     )
///     .map(|cx| cx.attach());
/// }
/// ```
pub fn new_span_if_recording(
    builder_fn: impl Fn() -> SpanBuilder,
    tracer: TracerSource<'_>,
) -> Option<Context> {
    Context::map_current(|current| {
        current.span().is_recording().then(|| {
            let builder = builder_fn();
            let span = tracer.get().build_with_context(builder, current);
            current.with_span(span)
        })
    })
}

/// Carries anything with an optional `opentelemetry::Context`.
///
/// A `Contextualized<T>` is a smart pointer which owns and instance of `T` and
/// dereferences to it automatically.  The instance of `T` and its associated
/// optional `Context` can be reacquired using the `Into` trait for the associated
/// tuple type.
///
/// This type is mostly useful when sending `T`'s through channels with logical
/// context propagation.
///
/// # Examples
///
/// ```
/// use opentelemetry::trace::{SpanBuilder, TraceContextExt as _};
/// use opentelemetry_contrib::trace::{new_span_if_parent_sampled, Contextualized, TracerSource};

/// enum Message{Command};
/// let (tx, rx) = std::sync::mpsc::channel();
///
/// let cx = new_span_if_parent_sampled(
///     || SpanBuilder::from_name("my command"),
///     TracerSource::lazy(&|| opentelemetry::global::tracer(module_path!())),
/// );
/// tx.send(Contextualized::new(Message::Command, cx));
///
/// let msg = rx.recv().unwrap();
/// let (msg, cx) = msg.into_inner();
/// let _guard = cx.filter(|cx| cx.has_active_span()).map(|cx| {
///     cx.span().add_event("command received", vec![]);
///     cx.attach()
/// });
/// ```
pub struct Contextualized<T>(T, Option<Context>);

impl<T> Contextualized<T> {
    /// Creates a new instance using the specified value and optional context.
    pub fn new(value: T, cx: Option<Context>) -> Self {
        Self(value, cx)
    }

    /// Creates a new instance using the specified value and current context if
    /// it has an active span.
    pub fn pass_thru(value: T) -> Self {
        Self::new(
            value,
            Context::map_current(|current| current.has_active_span().then(|| current.clone())),
        )
    }

    /// Convert self into its constituent parts, returning a tuple.
    pub fn into_inner(self) -> (T, Option<Context>) {
        (self.0, self.1)
    }

    /// Attach the contained context if it exists and return both the
    /// associated value and an optional guard for the attached context.
    pub fn attach(self) -> (T, Option<ContextGuard>) {
        (self.0, self.1.map(|cx| cx.attach()))
    }
}

impl<T: Clone> Clone for Contextualized<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}

impl<T: Debug> Debug for Contextualized<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Contextualized")
            .field(&self.0)
            .field(&self.1)
            .finish()
    }
}

impl<T> Deref for Contextualized<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Contextualized<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cover_contextualized() {
        let cx = Contextualized::new(17, None);
        let (i, cx) = cx.into_inner();
        assert_eq!(i, 17);
        assert!(cx.is_none());

        let cx = Contextualized::pass_thru(17);
        let (i, _guard) = cx.attach();
        assert_eq!(i, 17);
    }
}
