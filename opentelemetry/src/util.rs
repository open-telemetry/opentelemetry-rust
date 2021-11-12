//! Internal utilities
use std::sync::Arc;

/// Helper which wraps `tokio::time::interval` and makes it return a stream
#[cfg(any(feature = "rt-tokio", feature = "rt-tokio-current-thread"))]
pub fn tokio_interval_stream(
    period: std::time::Duration,
) -> tokio_stream::wrappers::IntervalStream {
    tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(period))
}

/// Take ownership of a unshared value or clone if it is shared still
pub fn take_or_else_clone<T: Clone>(value: Arc<T>) -> T {
    Arc::try_unwrap(value).unwrap_or_else(|arc| (*arc).clone())
}
