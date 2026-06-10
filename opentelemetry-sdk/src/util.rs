//! Internal utilities

/// Helper which wraps `tokio::time::interval` and makes it return a stream
#[cfg(any(feature = "rt-tokio", feature = "rt-tokio-current-thread"))]
pub fn tokio_interval_stream(
    period: std::time::Duration,
) -> tokio_stream::wrappers::IntervalStream {
    tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(period))
}

/// Extracts a human-readable message from a panic payload, such as the value
/// returned by [`std::thread::JoinHandle::join`] or
/// [`std::panic::catch_unwind`].
///
/// Panic payloads are almost always `&str` (from `panic!("literal")`) or
/// `String` (from `panic!("{}", ...)`); any other payload type yields a generic
/// fallback, as its value cannot be rendered without knowing its concrete type.
#[cfg(any(feature = "trace", feature = "logs"))]
pub(crate) fn panic_message(payload: &(dyn std::any::Any + Send)) -> &str {
    payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("unknown cause")
}

#[cfg(all(test, any(feature = "trace", feature = "logs")))]
mod tests {
    use super::panic_message;
    use std::any::Any;

    #[test]
    fn panic_message_extracts_str_payload() {
        // `panic!("literal")` produces a `&str` payload.
        let payload: Box<dyn Any + Send> = Box::new("boom");
        assert_eq!(panic_message(&*payload), "boom");
    }

    #[test]
    fn panic_message_extracts_string_payload() {
        // `panic!("{}", ...)` produces a `String` payload.
        let payload: Box<dyn Any + Send> = Box::new(String::from("kaboom"));
        assert_eq!(panic_message(&*payload), "kaboom");
    }

    #[test]
    fn panic_message_falls_back_for_other_payloads() {
        let payload: Box<dyn Any + Send> = Box::new(42i32);
        assert_eq!(panic_message(&*payload), "unknown cause");
    }
}
