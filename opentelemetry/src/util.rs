//! Internal utilities

/// Helper which wraps `tokio::time::interval` and makes it return a stream
#[cfg(any(test, feature = "tokio_support"))]
pub fn tokio_interval_stream(
    period: std::time::Duration,
) -> impl futures::Stream<Item = tokio::time::Instant> {
    let mut interval = tokio::time::interval(period);
    async_stream::stream! {
        let tick = interval.tick().await;
        yield tick;
    }
}
