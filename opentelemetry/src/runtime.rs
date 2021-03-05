//! Provides an abstraction of several runtimes, which allows OpenTelemetry to work with any
//! current or future runtime. There are currently builtin implementations for Tokio and async-std.

use futures::{future::BoxFuture, stream::BoxStream};
use std::time::Duration;

/// A runtime in an abstraction of a (possibly async runtime) like tokio or async-std. It allows
/// OpenTelemetry to work with any current and future runtime implementation.
pub trait Runtime: Clone + Send + Sync + 'static {
    /// Return if the runtime supports the functions required for batch processing (interval, spawn
    /// and delay).
    fn supports_batch_processing(&self) -> bool;

    /// Create a [Stream][futures::Stream], which returns a new item every
    /// [Duration][std::time::Duration].
    fn interval(&self, duration: Duration) -> BoxStream<'static, ()>;

    /// Spawn a new task or thread, which executes the given future.
    fn spawn(&self, future: BoxFuture<'static, ()>);

    /// Return a new future, which resolves after the specified [Duration][std::time::Duration].
    fn delay(&self, duration: Duration) -> BoxFuture<'static, ()>;
}

impl Runtime for () {
    fn supports_batch_processing(&self) -> bool {
        false
    }

    fn interval(&self, _duration: Duration) -> BoxStream<'static, ()> {
        unimplemented!()
    }

    fn spawn(&self, _future: BoxFuture<'static, ()>) {
        unimplemented!()
    }

    fn delay(&self, _duration: Duration) -> BoxFuture<'static, ()> {
        unimplemented!()
    }
}

/// Runtime implementation, which works with Tokio's multi thread runtime.
#[cfg(feature = "rt-tokio")]
#[derive(Debug, Clone)]
pub struct Tokio;

#[cfg(feature = "rt-tokio")]
impl Runtime for Tokio {
    fn supports_batch_processing(&self) -> bool {
        true
    }

    fn interval(&self, duration: Duration) -> BoxStream<'static, ()> {
        use futures::StreamExt as _;
        Box::pin(crate::util::tokio_interval_stream(duration).map(|_| ()))
    }

    fn spawn(&self, future: BoxFuture<'static, ()>) {
        let _ = tokio::spawn(future);
    }

    fn delay(&self, duration: Duration) -> BoxFuture<'static, ()> {
        Box::pin(tokio::time::sleep(duration))
    }
}

/// Runtime implementation, which works with Tokio's current thread runtime.
#[cfg(feature = "rt-tokio-current-thread")]
#[derive(Debug, Clone)]
pub struct TokioCurrentThread;

#[cfg(feature = "rt-tokio-current-thread")]
impl Runtime for TokioCurrentThread {
    fn supports_batch_processing(&self) -> bool {
        true
    }

    fn interval(&self, duration: Duration) -> BoxStream<'static, ()> {
        use futures::StreamExt as _;
        Box::pin(crate::util::tokio_interval_stream(duration).map(|_| ()))
    }

    fn spawn(&self, future: BoxFuture<'static, ()>) {
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(future);
        });
    }

    fn delay(&self, duration: Duration) -> BoxFuture<'static, ()> {
        Box::pin(tokio::time::sleep(duration))
    }
}

/// Runtime implementation, which works with async-std.
#[cfg(feature = "rt-async-std")]
#[derive(Debug, Clone)]
pub struct AsyncStd;

#[cfg(feature = "rt-async-std")]
impl Runtime for AsyncStd {
    fn supports_batch_processing(&self) -> bool {
        true
    }

    fn interval(&self, duration: Duration) -> BoxStream<'static, ()> {
        Box::pin(async_std::stream::interval(duration))
    }

    fn spawn(&self, future: BoxFuture<'static, ()>) {
        let _ = async_std::task::spawn(future);
    }

    fn delay(&self, duration: Duration) -> BoxFuture<'static, ()> {
        Box::pin(async_std::task::sleep(duration))
    }
}
