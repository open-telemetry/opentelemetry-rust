//! Internal utilities

/// Helper which wraps `tokio::time::interval` and makes it return a stream
#[cfg(feature = "rt-tokio")]
pub fn tokio_interval_stream(
    period: std::time::Duration,
) -> tokio_stream::wrappers::IntervalStream {
    tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(period))
}
