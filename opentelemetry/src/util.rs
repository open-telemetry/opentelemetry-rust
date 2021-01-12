//! Internal utilities

/// Helper which wraps `tokio::time::interval` and makes it return a stream
#[cfg(any(test, feature = "tokio_support"))]
pub fn tokio_interval_stream(
    period: std::time::Duration,
) -> tokio_stream::wrappers::IntervalStream {
    tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(period))
}
