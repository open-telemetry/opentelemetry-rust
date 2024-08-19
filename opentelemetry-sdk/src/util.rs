//! Internal utilities
use std::future::Future;

/// Helper which wraps `tokio::time::interval` and makes it return a stream
#[cfg(any(feature = "rt-tokio", feature = "rt-tokio-current-thread"))]
pub fn tokio_interval_stream(
    period: std::time::Duration,
) -> tokio_stream::wrappers::IntervalStream {
    tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(period))
}

#[cfg(any(
    not(target_arch = "wasm32"),
    all(target_arch = "wasm32", target_os = "wasi")
))]
pub(crate) fn spawn_future<F>(f: F)
where
    F: Future + 'static,
{
    futures_executor::block_on(f);
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
pub(crate) fn spawn_future<F>(f: F)
where
    F: Future + 'static,
{
    wasm_bindgen_futures::spawn_local(async move {
        f.await;
    })
}
