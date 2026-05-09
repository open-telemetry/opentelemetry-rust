//! Internal utilities

/// Helper which wraps `tokio::time::interval` and makes it return a stream
#[cfg(any(feature = "rt-tokio", feature = "rt-tokio-current-thread"))]
pub fn tokio_interval_stream(
    period: std::time::Duration,
) -> tokio_stream::wrappers::IntervalStream {
    tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(period))
}

/// Strategy for blocking on async futures from synchronous contexts.
///
/// When constructed within a tokio runtime, captures the runtime handle
/// and enters the runtime context via [`tokio::runtime::Handle::enter()`]
/// before blocking with [`futures_executor::block_on()`]. This makes tokio
/// types (spawn, timers, IO resources) available on dedicated background
/// threads without taking ownership of the reactor — IO continues to be
/// driven by the runtime's own threads.
///
/// Falls back to plain [`futures_executor::block_on()`] when no tokio runtime
/// is available (e.g., non-tokio environments).
#[cfg(any(feature = "trace", feature = "logs", feature = "metrics"))]
#[derive(Clone, Debug)]
pub(crate) enum BlockingStrategy {
    #[cfg(feature = "rt-tokio")]
    TokioHandle(tokio::runtime::Handle),
    FuturesExecutor,
}

#[cfg(any(feature = "trace", feature = "logs", feature = "metrics"))]
impl BlockingStrategy {
    pub(crate) fn new() -> Self {
        #[cfg(feature = "rt-tokio")]
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            return Self::TokioHandle(handle);
        }
        Self::FuturesExecutor
    }

    pub(crate) fn block_on<F: std::future::Future>(&self, future: F) -> F::Output {
        match self {
            #[cfg(feature = "rt-tokio")]
            Self::TokioHandle(handle) => {
                let _guard = handle.enter();
                futures_executor::block_on(future)
            }
            Self::FuturesExecutor => futures_executor::block_on(future),
        }
    }
}

/// Test-only: exercise the tokio timer and IO drivers from a spawned task.
/// Used by mock exporters to verify processors keep the runtime drivers alive
/// on their dedicated worker threads. Without this, the deadlock fix could
/// silently regress for exporters that depend on either driver (e.g. tonic).
#[cfg(test)]
pub(crate) async fn exercise_tokio_drivers() {
    // Timer driver.
    tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    // IO driver — bind registers an FD with the reactor; immediate drop is fine.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let _ = listener.local_addr().unwrap();
}
