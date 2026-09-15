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
    feature = "experimental_metrics_periodicreader_with_async_runtime",
))]
pub(crate) fn panic_message(payload: &(dyn std::any::Any + Send)) -> &str {
    payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("unknown cause")
}

/// Tracks a background worker's join handle across the lifetime of a
/// processor/reader, so that repeated shutdown calls observe a single,
/// cached final result instead of racing to consume the same handle.
#[cfg(any(
    feature = "experimental_trace_batch_span_processor_with_async_runtime",
    feature = "experimental_logs_batch_log_processor_with_async_runtime",
    feature = "experimental_metrics_periodicreader_with_async_runtime",
))]
pub(crate) enum WorkerState<H> {
    Running(H),
    Complete(Result<(), String>),
}

/// Error from [`join_with_timeout`]: either the join itself failed, or the
/// timeout elapsed before the worker finished.
#[cfg(any(
    feature = "experimental_trace_batch_span_processor_with_async_runtime",
    feature = "experimental_logs_batch_log_processor_with_async_runtime",
    feature = "experimental_metrics_periodicreader_with_async_runtime",
))]
pub(crate) enum JoinTimeoutError {
    TimedOut,
    Join(crate::runtime::JoinError),
}

/// Joins a [`crate::runtime::JoinHandle`] with a bound on how long to wait.
///
/// `JoinHandle::join` itself has no timeout support, so the actual join is
/// performed on a dedicated OS thread; the result is relayed back over a
/// channel so the caller can bound its wait with `recv_timeout`. If the
/// timeout elapses, the background thread is abandoned (it will still
/// complete the join, but nothing is listening for the result).
#[cfg(any(
    feature = "experimental_trace_batch_span_processor_with_async_runtime",
    feature = "experimental_logs_batch_log_processor_with_async_runtime",
    feature = "experimental_metrics_periodicreader_with_async_runtime",
))]
pub(crate) fn join_with_timeout<H, T>(
    handle: H,
    timeout: std::time::Duration,
) -> Result<T, JoinTimeoutError>
where
    H: crate::runtime::JoinHandle<T>,
    T: Send + 'static,
{
    let (tx, rx) = std::sync::mpsc::channel();
    let _ = std::thread::spawn(move || {
        let _ = tx.send(handle.join());
    });

    match rx.recv_timeout(timeout) {
        Ok(join_result) => join_result.map_err(JoinTimeoutError::Join),
        Err(_) => Err(JoinTimeoutError::TimedOut),
    }
}

/// Converts a [`JoinTimeoutError`] into the [`crate::error::OTelSdkError`]
/// reported from a processor/reader's `shutdown_with_timeout`.
///
/// `component` names the worker in the resulting message, e.g. `"batch log
/// processor"`.
#[cfg(any(
    feature = "experimental_trace_batch_span_processor_with_async_runtime",
    feature = "experimental_logs_batch_log_processor_with_async_runtime",
    feature = "experimental_metrics_periodicreader_with_async_runtime",
))]
pub(crate) fn join_timeout_error_to_otel_error(
    err: JoinTimeoutError,
    component: &str,
    timeout: std::time::Duration,
) -> crate::error::OTelSdkError {
    match err {
        JoinTimeoutError::TimedOut => crate::error::OTelSdkError::Timeout(timeout),
        JoinTimeoutError::Join(crate::runtime::JoinError::Panic(payload)) => {
            crate::error::OTelSdkError::InternalFailure(format!(
                "{component} worker panicked during shutdown: {}",
                panic_message(&*payload)
            ))
        }
        #[cfg(feature = "rt-tokio")]
        JoinTimeoutError::Join(crate::runtime::JoinError::Cancelled) => {
            crate::error::OTelSdkError::InternalFailure(format!(
                "{component} worker was cancelled during shutdown"
            ))
        }
    }
}

/// Helper which wraps `tokio::time::interval` and makes it return a stream
#[cfg(feature = "rt-tokio")]
pub fn tokio_interval_stream(
    period: std::time::Duration,
) -> tokio_stream::wrappers::IntervalStream {
    tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(period))
}
