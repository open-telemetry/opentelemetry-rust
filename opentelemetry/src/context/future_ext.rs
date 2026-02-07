use crate::Context;
use futures_core::Stream;
use futures_sink::Sink;
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::Context as TaskContext;
use std::task::Poll;
#[allow(deprecated)]
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

pin_project! {
    /// A future, stream, or sink that has an associated context.
    #[derive(Clone, Debug)]
    pub struct WithContext<T> {
        #[pin]
        inner: T,
        otel_cx: Context,
    }
}

impl<I, T: Sink<I>> Sink<I> for WithContext<T> {
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
#[deprecated = "Overly general extension trait impl, use FutureContextExt, StreamContextExt, or SinkContextExt instead"]
pub trait FutureExt: Sized {
    /// Attaches the provided [`Context`] to this type, returning a `WithContext`
    /// wrapper.
    ///
    /// When the wrapped type is a future, stream, or sink, the attached context
    /// will be set as current while it is being polled.
    ///
    /// [`Context`]: Context
    #[deprecated = "Overly general extension trait impl, use FutureContextExt, StreamContextExt, or SinkContextExt instead"]
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
    /// [`Context`]: Context
    #[deprecated = "Overly general extension trait impl, use FutureContextExt, StreamContextExt, or SinkContextExt instead"]
    #[allow(deprecated)]
    fn with_current_context(self) -> WithContext<Self> {
        let otel_cx = Context::current();
        self.with_context(otel_cx)
    }
}

// The following three extension traits are _almost_ identical,
// but need to be separate to avoid overlapping implemenetation errors.

impl<F: std::future::Future> FutureContextExt for F {}
/// Extension trait allowing futures to be traced with a span.
pub trait FutureContextExt: Sized {
    /// Attaches the provided [`Context`] to this future, returning a `WithContext`
    /// wrapper.
    ///
    /// The attached context will be set as current while this future is being polled.
    ///
    /// [`Context`]: Context
    fn with_context(self, otel_cx: Context) -> WithContext<Self> {
        WithContext {
            inner: self,
            otel_cx,
        }
    }

    /// Attaches the current [`Context`] to this future, returning a `WithContext`
    /// wrapper.
    ///
    /// The attached context will be set as current while this future is being polled.
    ///
    /// [`Context`]: Context
    fn with_current_context(self) -> WithContext<Self> {
        let otel_cx = Context::current();
        self.with_context(otel_cx)
    }
}

impl<S: Stream> StreamContextExt for S {}
/// Extension trait allowing streams to be traced with a span.
pub trait StreamContextExt: Sized {
    /// Attaches the provided [`Context`] to this stream, returning a `WithContext`
    /// wrapper.
    ///
    /// The attached context will be set as current while this stream is being polled.
    ///
    /// [`Context`]: Context
    fn with_context(self, otel_cx: Context) -> WithContext<Self> {
        WithContext {
            inner: self,
            otel_cx,
        }
    }

    /// Attaches the current [`Context`] to this stream, returning a `WithContext`
    /// wrapper.
    ///
    /// The attached context will be set as current while this stream is being polled.
    ///
    /// [`Context`]: Context
    fn with_current_context(self) -> WithContext<Self> {
        let otel_cx = Context::current();
        self.with_context(otel_cx)
    }
}

impl<_I, S: Sink<_I>> SinkContextExt<_I> for S {}
/// Extension trait allowing sinks to be traced with a span.
///
/// The generic argument is unused.
pub trait SinkContextExt<_I>: Sized {
    /// Attaches the provided [`Context`] to this sink, returning a `WithContext`
    /// wrapper.
    ///
    /// The attached context will be set as current while this sink is being polled.
    ///
    /// [`Context`]: Context
    fn with_context(self, otel_cx: Context) -> WithContext<Self> {
        WithContext {
            inner: self,
            otel_cx,
        }
    }

    /// Attaches the current [`Context`] to this sink, returning a `WithContext`
    /// wrapper.
    ///
    /// The attached context will be set as current while this sink is being polled.
    ///
    /// [`Context`]: Context
    fn with_current_context(self) -> WithContext<Self> {
        let otel_cx = Context::current();
        self.with_context(otel_cx)
    }
}
