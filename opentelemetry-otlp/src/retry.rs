//! This module provides functionality for retrying operations with exponential backoff and jitter.
//!
//! The `RetryPolicy` struct defines the configuration for the retry behavior, including the maximum
//! number of retries, initial delay, maximum delay, and jitter.
//!
//! The `retry_with_backoff` function retries the given operation according to the
//! specified retry policy, using exponential backoff and jitter to determine the delay between
//! retries. The function uses error classification to determine retry behavior and can honor
//! server-provided throttling hints.

use opentelemetry::{otel_debug, otel_info, otel_warn};
use std::future::Future;
use std::hash::{DefaultHasher, Hasher};
use std::time::{Duration, Instant, SystemTime};

/// Sleeps for the given duration, using `tokio::time::sleep` if running inside
/// a Tokio runtime (cooperative, won't block other tasks), or falling back to
/// `std::thread::sleep` when on a bare OS thread (e.g. the SDK's default
/// batch-processor export thread).
async fn sleep_for(duration: Duration) {
    #[cfg(feature = "tokio")]
    {
        if tokio::runtime::Handle::try_current().is_ok() {
            tokio::time::sleep(duration).await;
            return;
        }
    }
    std::thread::sleep(duration);
}

/// Classification of errors for retry purposes.
#[derive(Debug, Clone, PartialEq)]
pub enum RetryErrorType {
    /// Error is not retryable (e.g., authentication failure, bad request).
    NonRetryable,
    /// Error is retryable with exponential backoff (e.g., server error, network timeout).
    Retryable,
    /// Error indicates throttling - wait for the specified duration before retrying.
    /// This overrides exponential backoff timing.
    Throttled(Duration),
}

/// Configuration for retry policy.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts.
    pub max_retries: usize,
    /// Initial delay in milliseconds before the first retry.
    pub initial_delay_ms: u64,
    /// Maximum delay in milliseconds between retries.
    pub max_delay_ms: u64,
    /// Maximum jitter in milliseconds to add to the delay.
    pub jitter_ms: u64,
}

impl Default for RetryPolicy {
    /// Default retry policy performs no retries (single attempt only).
    /// Use `RetryPolicy::recommended()` or configure explicitly to enable retry.
    fn default() -> Self {
        Self {
            max_retries: 0,
            initial_delay_ms: 100,
            max_delay_ms: 1600,
            jitter_ms: 100,
        }
    }
}

impl RetryPolicy {
    /// Recommended retry policy per the OTLP spec: 3 retries with exponential
    /// backoff (100ms initial, 1600ms max, 100ms jitter).
    pub fn recommended() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 1600,
            jitter_ms: 100,
        }
    }
}

/// Maximum duration a server-provided throttle delay (Retry-After / RetryInfo) is allowed
/// to block the export thread. Any server-provided delay exceeding this cap is truncated.
/// This prevents a misbehaving server from stalling the export pipeline indefinitely.
const MAX_THROTTLE_DURATION: Duration = Duration::from_secs(30);

// Generates a random jitter value up to max_jitter
fn generate_jitter(max_jitter: u64) -> u64 {
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();

    let mut hasher = DefaultHasher::default();
    hasher.write_u32(nanos);
    hasher.finish() % (max_jitter + 1)
}

/// Retries the given operation with exponential backoff, jitter, and error classification.
///
/// This function provides retry behavior by classifying errors and honoring server-provided
/// throttling hints (e.g., HTTP Retry-After, gRPC RetryInfo).
///
/// Delays between retries adapt to the calling context: when running inside a Tokio
/// runtime (gRPC/async HTTP exporters), `tokio::time::sleep` is used cooperatively;
/// on a bare OS thread (the SDK's default batch processors), `std::thread::sleep`
/// is used instead.
///
/// A time budget (deadline) bounds the total retry duration. If the budget is exhausted,
/// the function returns the last error without further retries.
///
/// # Arguments
///
/// * `policy` - The retry policy configuration.
/// * `deadline` - Maximum total time allowed for all retry attempts combined.
/// * `error_classifier` - Function to classify errors for retry decisions.
/// * `operation_name` - The name of the operation being retried.
/// * `operation` - The operation to be retried.
///
/// # Returns
///
/// A `Result` containing the operation's result or an error if max retries are reached
/// or a non-retryable error occurs.
pub async fn retry_with_backoff<F, Fut, T, E, C>(
    policy: &RetryPolicy,
    deadline: Duration,
    error_classifier: C,
    operation_name: &str,
    mut operation: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    E: std::fmt::Debug,
    Fut: Future<Output = Result<T, E>>,
    C: Fn(&E) -> RetryErrorType,
{
    let start = Instant::now();
    let mut attempt = 0;
    let mut delay = policy.initial_delay_ms;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(err) => {
                // Check time budget before deciding to retry
                let elapsed = start.elapsed();
                if elapsed >= deadline {
                    otel_warn!(name: "Export.Failed.DeadlineExceeded",
                        operation = operation_name,
                        retries = attempt,
                        elapsed_ms = elapsed.as_millis(),
                        message = "OTLP export deadline exceeded - telemetry data will be lost"
                    );
                    return Err(err);
                }

                let error_type = error_classifier(&err);

                match error_type {
                    RetryErrorType::NonRetryable => {
                        otel_warn!(name: "Export.Failed.NonRetryable",
                            operation = operation_name,
                            message = "OTLP export failed with non-retryable error - telemetry data will be lost"
                        );
                        return Err(err);
                    }
                    RetryErrorType::Retryable if attempt < policy.max_retries => {
                        attempt += 1;
                        let jitter = generate_jitter(policy.jitter_ms);
                        let delay_ms = std::cmp::min(delay + jitter, policy.max_delay_ms);
                        let sleep_duration = Duration::from_millis(delay_ms);

                        // Don't sleep longer than the remaining budget
                        let remaining = deadline.saturating_sub(start.elapsed());
                        let actual_sleep = sleep_duration.min(remaining);

                        if actual_sleep.is_zero() {
                            otel_warn!(name: "Export.Failed.DeadlineExceeded",
                                operation = operation_name,
                                retries = attempt,
                                message = "OTLP export deadline exceeded during backoff - telemetry data will be lost"
                            );
                            return Err(err);
                        }

                        otel_debug!(name: "Export.InProgress.Retrying",
                            operation = operation_name,
                            attempt = attempt,
                            delay_ms = actual_sleep.as_millis(),
                            message = "OTLP export failed with retryable error - retrying"
                        );
                        sleep_for(actual_sleep).await;
                        delay = std::cmp::min(delay * 2, policy.max_delay_ms);
                    }
                    RetryErrorType::Throttled(server_delay) if attempt < policy.max_retries => {
                        attempt += 1;
                        // Cap server-provided delay to prevent excessive blocking
                        let capped_delay = server_delay.min(MAX_THROTTLE_DURATION);

                        // Further constrain to remaining time budget
                        let remaining = deadline.saturating_sub(start.elapsed());
                        let actual_sleep = capped_delay.min(remaining);

                        if actual_sleep.is_zero() {
                            otel_warn!(name: "Export.Failed.DeadlineExceeded",
                                operation = operation_name,
                                retries = attempt,
                                message = "OTLP export deadline exceeded during throttle - telemetry data will be lost"
                            );
                            return Err(err);
                        }

                        otel_info!(name: "Export.InProgress.Throttled",
                            operation = operation_name,
                            attempt = attempt,
                            delay_ms = actual_sleep.as_millis(),
                            server_requested_ms = server_delay.as_millis(),
                            message = "OTLP export throttled by OTLP endpoint - delaying and retrying"
                        );
                        sleep_for(actual_sleep).await;
                        // Don't update exponential backoff delay since server provided specific timing
                    }
                    _ => {
                        // Max retries reached
                        otel_warn!(name: "Export.Failed.Exhausted",
                            operation = operation_name,
                            retries = attempt,
                            message = "OTLP export exhausted retries - telemetry data will be lost"
                        );
                        return Err(err);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_generate_jitter() {
        let max_jitter = 100;
        let jitter = generate_jitter(max_jitter);
        assert!(jitter <= max_jitter);
    }

    #[test]
    fn test_default_retry_policy() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_retries, 0);
        assert_eq!(policy.initial_delay_ms, 100);
        assert_eq!(policy.max_delay_ms, 1600);
        assert_eq!(policy.jitter_ms, 100);
    }

    #[test]
    fn test_recommended_retry_policy() {
        let policy = RetryPolicy::recommended();
        assert_eq!(policy.max_retries, 3);
        assert_eq!(policy.initial_delay_ms, 100);
        assert_eq!(policy.max_delay_ms, 1600);
        assert_eq!(policy.jitter_ms, 100);
    }

    #[test]
    fn test_retry_error_type_equality() {
        assert_eq!(RetryErrorType::NonRetryable, RetryErrorType::NonRetryable);
        assert_eq!(RetryErrorType::Retryable, RetryErrorType::Retryable);
        assert_eq!(
            RetryErrorType::Throttled(Duration::from_secs(30)),
            RetryErrorType::Throttled(Duration::from_secs(30))
        );
        assert_ne!(RetryErrorType::Retryable, RetryErrorType::NonRetryable);
    }

    #[tokio::test]
    async fn test_retry_success_on_first_attempt() {
        let policy = RetryPolicy::default();
        let deadline = Duration::from_secs(10);

        let result = retry_with_backoff(
            &policy,
            deadline,
            |_: &()| RetryErrorType::Retryable,
            "test_operation",
            || Box::pin(async { Ok::<_, ()>("success") }),
        )
        .await;

        assert_eq!(result, Ok("success"));
    }

    #[tokio::test]
    async fn test_retry_succeeds_after_retries() {
        let policy = RetryPolicy {
            max_retries: 3,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            jitter_ms: 5,
        };
        let deadline = Duration::from_secs(10);
        let attempts = AtomicUsize::new(0);

        let result = retry_with_backoff(
            &policy,
            deadline,
            |_: &&str| RetryErrorType::Retryable,
            "test_operation",
            || {
                let attempt = attempts.fetch_add(1, Ordering::SeqCst);
                Box::pin(async move {
                    if attempt < 2 {
                        Err("error")
                    } else {
                        Ok("success")
                    }
                })
            },
        )
        .await;

        assert_eq!(result, Ok("success"));
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_fails_after_max_retries() {
        let policy = RetryPolicy {
            max_retries: 3,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            jitter_ms: 5,
        };
        let deadline = Duration::from_secs(10);
        let attempts = AtomicUsize::new(0);

        let result = retry_with_backoff(
            &policy,
            deadline,
            |_: &&str| RetryErrorType::Retryable,
            "test_operation",
            || {
                attempts.fetch_add(1, Ordering::SeqCst);
                Box::pin(async { Err::<(), _>("error") })
            },
        )
        .await;

        assert_eq!(result, Err("error"));
        assert_eq!(attempts.load(Ordering::SeqCst), 4); // initial + 3 retries
    }

    #[tokio::test]
    async fn test_retry_non_retryable_stops_immediately() {
        let policy = RetryPolicy::default();
        let deadline = Duration::from_secs(10);
        let attempts = AtomicUsize::new(0);

        let result = retry_with_backoff(
            &policy,
            deadline,
            |_: &()| RetryErrorType::NonRetryable,
            "test_operation",
            || {
                attempts.fetch_add(1, Ordering::SeqCst);
                Box::pin(async { Err::<(), _>(()) })
            },
        )
        .await;

        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_throttled_uses_server_delay() {
        let policy = RetryPolicy {
            max_retries: 2,
            initial_delay_ms: 1000, // high default delay
            max_delay_ms: 5000,
            jitter_ms: 0,
        };
        let deadline = Duration::from_secs(10);
        let attempts = AtomicUsize::new(0);

        let start = Instant::now();
        let result = retry_with_backoff(
            &policy,
            deadline,
            |_: &()| RetryErrorType::Throttled(Duration::from_millis(50)),
            "test_operation",
            || {
                let attempt = attempts.fetch_add(1, Ordering::SeqCst);
                Box::pin(async move {
                    if attempt < 1 {
                        Err(())
                    } else {
                        Ok("success")
                    }
                })
            },
        )
        .await;

        let elapsed = start.elapsed();
        assert_eq!(result, Ok("success"));
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
        // Should have waited ~50ms (the server delay), not 1000ms (the default)
        assert!(elapsed >= Duration::from_millis(50));
        assert!(elapsed < Duration::from_millis(200));
    }

    #[tokio::test]
    async fn test_retry_deadline_exceeded() {
        let policy = RetryPolicy {
            max_retries: 100, // many retries allowed
            initial_delay_ms: 50,
            max_delay_ms: 200,
            jitter_ms: 0,
        };
        // Very short deadline
        let deadline = Duration::from_millis(120);
        let attempts = AtomicUsize::new(0);

        let start = Instant::now();
        let result = retry_with_backoff(
            &policy,
            deadline,
            |_: &()| RetryErrorType::Retryable,
            "test_operation",
            || {
                attempts.fetch_add(1, Ordering::SeqCst);
                Box::pin(async { Err::<(), _>(()) })
            },
        )
        .await;

        let elapsed = start.elapsed();
        assert!(result.is_err());
        // Should have stopped before exhausting 100 retries due to deadline
        assert!(attempts.load(Ordering::SeqCst) < 10);
        // Shouldn't have taken much longer than the deadline
        assert!(elapsed < Duration::from_millis(300));
    }

    #[tokio::test]
    async fn test_retry_throttle_capped_at_max() {
        let policy = RetryPolicy {
            max_retries: 2,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            jitter_ms: 0,
        };
        let attempts = AtomicUsize::new(0);

        let start = Instant::now();
        // Server requests 120s delay (exceeds MAX_THROTTLE_DURATION of 30s)
        // But deadline is short so we don't wait 30s in tests
        let short_deadline = Duration::from_millis(200);
        let result = retry_with_backoff(
            &policy,
            short_deadline,
            |_: &()| RetryErrorType::Throttled(Duration::from_secs(120)),
            "test_operation",
            || {
                attempts.fetch_add(1, Ordering::SeqCst);
                Box::pin(async { Err::<(), _>(()) })
            },
        )
        .await;

        let elapsed = start.elapsed();
        assert!(result.is_err());
        // Should be capped by the deadline, not sleep for 120s or even 30s
        assert!(elapsed < Duration::from_millis(500));
    }
}
