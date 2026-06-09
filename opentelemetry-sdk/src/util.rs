//! Internal utilities

/// Extracts a human-readable message from a panic payload, such as the value
/// returned by [`std::thread::JoinHandle::join`] or [`std::panic::catch_unwind`].
///
/// Panic payloads are almost always `&str` (from `panic!("literal")`) or
/// `String` (from `panic!("{}", ...)`); any other payload type yields a generic
/// fallback, as its value cannot be rendered without knowing its concrete type.
#[cfg(any(
    feature = "experimental_trace_batch_span_processor_with_async_runtime",
    feature = "experimental_logs_batch_log_processor_with_async_runtime",
))]
pub(crate) fn panic_message(payload: &(dyn std::any::Any + Send)) -> &str {
    payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("unknown cause")
}

/// Helper which wraps `tokio::time::interval` and makes it return a stream
#[cfg(feature = "rt-tokio")]
pub fn tokio_interval_stream(
    period: std::time::Duration,
) -> tokio_stream::wrappers::IntervalStream {
    tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(period))
}
