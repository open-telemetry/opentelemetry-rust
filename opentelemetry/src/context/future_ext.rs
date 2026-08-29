use crate::Context;
use futures_core::Stream;
use futures_sink::Sink;
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::Context as TaskContext;
use std::task::Poll;

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
///
/// This trait relied on an overly general blanket implementation
/// (`impl<T> FutureExt for T`) that exposed `with_context` on *every* type, which
/// could collide with similarly named methods from other crates (for example
/// `anyhow::Context::with_context`). It is replaced by the precise
/// [`FutureContextExt`], [`StreamContextExt`], and [`SinkContextExt`] traits.
#[deprecated(
    since = "0.33.0",
    note = "overly general; use FutureContextExt, StreamContextExt, or SinkContextExt instead"
)]
pub trait FutureExt: Sized {
    /// Attaches the provided [`Context`] to this type, returning a `WithContext`
    /// wrapper.
    fn with_context(self, otel_cx: Context) -> WithContext<Self> {
        WithContext {
            inner: self,
            otel_cx,
        }
    }

    /// Attaches the current [`Context`] to this type, returning a `WithContext`
    /// wrapper.
    fn with_current_context(self) -> WithContext<Self> {
        let otel_cx = Context::current();
        self.with_context(otel_cx)
    }
}

// The three extension traits below are almost identical, but must be separate to
// avoid overlapping blanket-implementation errors. As a result, a type that is more
// than one of Future, Stream, or Sink must disambiguate the call, e.g.
// `FutureContextExt::with_context(value, cx)`.

/// Extension trait allowing futures to be traced with a span.
pub trait FutureContextExt: Sized {
    /// Attaches the provided [`Context`] to this future; it is set as the current
    /// context while the future is polled.
    fn with_context(self, otel_cx: Context) -> WithContext<Self> {
        WithContext {
            inner: self,
            otel_cx,
        }
    }

    /// Attaches the current [`Context`] to this future.
    fn with_current_context(self) -> WithContext<Self> {
        let otel_cx = Context::current();
        self.with_context(otel_cx)
    }
}

impl<F: std::future::Future> FutureContextExt for F {}

/// Extension trait allowing streams to be traced with a span.
pub trait StreamContextExt: Sized {
    /// Attaches the provided [`Context`] to this stream; it is set as the current
    /// context while the stream is polled.
    fn with_context(self, otel_cx: Context) -> WithContext<Self> {
        WithContext {
            inner: self,
            otel_cx,
        }
    }

    /// Attaches the current [`Context`] to this stream.
    fn with_current_context(self) -> WithContext<Self> {
        let otel_cx = Context::current();
        self.with_context(otel_cx)
    }
}

impl<S: Stream> StreamContextExt for S {}

/// Extension trait allowing sinks to be traced with a span.
///
/// The `I` type parameter is the sink item type; it only selects the
/// implementation and does not appear in the trait's methods.
pub trait SinkContextExt<I>: Sized {
    /// Attaches the provided [`Context`] to this sink; it is set as the current
    /// context while the sink is polled.
    fn with_context(self, otel_cx: Context) -> WithContext<Self> {
        WithContext {
            inner: self,
            otel_cx,
        }
    }

    /// Attaches the current [`Context`] to this sink.
    fn with_current_context(self) -> WithContext<Self> {
        let otel_cx = Context::current();
        self.with_context(otel_cx)
    }
}

impl<I, S: Sink<I>> SinkContextExt<I> for S {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::future::Future;
    use std::task::{RawWaker, RawWakerVTable, Waker};

    struct EmptyStream;

    impl Stream for EmptyStream {
        type Item = i32;

        fn poll_next(self: Pin<&mut Self>, _cx: &mut TaskContext<'_>) -> Poll<Option<Self::Item>> {
            Poll::Ready(None)
        }
    }

    struct NullSink;

    impl Sink<i32> for NullSink {
        type Error = ();

        fn poll_ready(self: Pin<&mut Self>, _cx: &mut TaskContext<'_>) -> Poll<Result<(), ()>> {
            Poll::Ready(Ok(()))
        }

        fn start_send(self: Pin<&mut Self>, _item: i32) -> Result<(), ()> {
            Ok(())
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut TaskContext<'_>) -> Poll<Result<(), ()>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(self: Pin<&mut Self>, _cx: &mut TaskContext<'_>) -> Poll<Result<(), ()>> {
            Poll::Ready(Ok(()))
        }
    }

    fn noop_waker() -> Waker {
        fn clone(_: *const ()) -> RawWaker {
            raw()
        }
        fn noop(_: *const ()) {}
        static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
        fn raw() -> RawWaker {
            RawWaker::new(std::ptr::null(), &VTABLE)
        }
        // SAFETY: every vtable function is a no-op that ignores the (null) data pointer.
        unsafe { Waker::from_raw(raw()) }
    }

    // `with_context` / `with_current_context` are available on futures, streams, and
    // sinks via the precise extension traits, and the returned `WithContext` wrapper
    // forwards to the underlying Future/Stream/Sink. Distinct stream-only and sink-only
    // types are used on purpose: a type implementing more than one of these would be
    // ambiguous at the call site.
    #[test]
    fn context_extensions_apply_to_future_stream_and_sink() {
        let waker = noop_waker();
        let mut cx = TaskContext::from_waker(&waker);

        // Future
        let mut fut = Box::pin(async {}.with_context(Context::current()));
        assert_eq!(fut.as_mut().poll(&mut cx), Poll::Ready(()));
        let mut fut = Box::pin(async {}.with_current_context());
        assert_eq!(fut.as_mut().poll(&mut cx), Poll::Ready(()));

        // Stream
        let mut stream = Box::pin(EmptyStream.with_context(Context::current()));
        assert_eq!(stream.as_mut().poll_next(&mut cx), Poll::Ready(None));
        let mut stream = Box::pin(EmptyStream.with_current_context());
        assert_eq!(stream.as_mut().poll_next(&mut cx), Poll::Ready(None));

        // Sink
        let mut sink = Box::pin(NullSink.with_context(Context::current()));
        assert_eq!(sink.as_mut().poll_ready(&mut cx), Poll::Ready(Ok(())));
        assert!(sink.as_mut().start_send(1).is_ok());
        assert_eq!(sink.as_mut().poll_flush(&mut cx), Poll::Ready(Ok(())));
        assert_eq!(sink.as_mut().poll_close(&mut cx), Poll::Ready(Ok(())));
        let mut sink = Box::pin(NullSink.with_current_context());
        assert_eq!(sink.as_mut().poll_ready(&mut cx), Poll::Ready(Ok(())));
    }
}
