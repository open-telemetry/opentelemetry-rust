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
use std::collections::hash_map::DefaultHasher;
use std::future::Future;
use std::hash::Hasher;
use std::time::{Duration, Instant, SystemTime};

/// Sleeps for the given duration.
///
/// Uses `tokio::time::sleep` when the `tokio` feature is enabled and a Tokio
/// runtime is present (cooperative, won't block the reactor); otherwise falls
/// back to `std::thread::sleep`.
///
/// Note: `tokio` is not a default feature, so default builds
/// (`reqwest-blocking-client`) always use `std::thread::sleep`. It is pulled in
/// by `grpc-tonic`, `reqwest-client`, and `hyper-client`.
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
    /// Error indicates throttling - wait for at least the specified duration before retrying.
    /// The server delay seeds exponential backoff: subsequent retries grow from it.
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
    /// Returns the recommended OTLP retry policy.
    fn default() -> Self {
        Self::recommended()
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
/// When a throttle delay is received, it seeds the exponential backoff: the delay is set
/// to `max(current_delay, server_delay)` and subsequent retries double from there. If the
/// server delay exceeds the configured `max_delay_ms`, the backoff cap is raised to
/// `MAX_THROTTLE_DURATION` (30 seconds) so that repeated failures produce increasing
/// delays rather than getting stuck at the server value.
///
/// Delays between retries adapt to the calling context: when running inside a Tokio
/// runtime (gRPC/async HTTP exporters), `tokio::time::sleep` is used cooperatively;
/// on a bare OS thread (the SDK's default batch processors), `std::thread::sleep`
/// is used instead.
///
/// A time budget (deadline) bounds the total retry duration. If the budget is exhausted,
/// the function returns the last error without further retries.
///
/// **Note:** The deadline governs whether to start another retry and caps inter-retry
/// sleep durations, but it does not cancel an in-flight export operation. On dedicated
/// threads (the SDK's default batch processors), individual export calls do not have a
/// timeout today, so a single slow export can exceed the deadline. This is an existing
/// limitation.
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
    let mut delay = Duration::from_millis(policy.initial_delay_ms);
    // The delay cap may be raised beyond the configured max if a server-provided
    // throttle delay exceeds it, since retrying sooner than the server requested
    // would defeat the purpose of backpressure signalling. The overall exporter
    // timeout remains the final bound.
    let mut delay_cap = Duration::from_millis(policy.max_delay_ms);

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
                        let jitter = Duration::from_millis(generate_jitter(policy.jitter_ms));
                        let sleep_duration = delay.saturating_add(jitter).min(delay_cap);

                        // If the backoff delay would consume all remaining budget, fail now
                        let remaining = deadline.saturating_sub(start.elapsed());
                        if sleep_duration >= remaining {
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
                            delay_ms = sleep_duration.as_millis(),
                            message = "OTLP export failed with retryable error - retrying"
                        );
                        sleep_for(sleep_duration).await;
                        delay = delay.saturating_mul(2).min(delay_cap);
                    }
                    RetryErrorType::Throttled(server_delay) if attempt < policy.max_retries => {
                        attempt += 1;
                        let capped_server_delay = server_delay.min(MAX_THROTTLE_DURATION);

                        // RetryInfo is the base for subsequent exponential backoff. A later
                        // RetryInfo may extend, but must not shorten, the current backoff.
                        // When the server delay exceeds the configured max, allow backoff
                        // to grow up to MAX_THROTTLE_DURATION so repeated failures
                        // produce increasing delays rather than getting stuck.
                        if capped_server_delay > delay_cap {
                            delay_cap = MAX_THROTTLE_DURATION;
                        }
                        if !capped_server_delay.is_zero() {
                            delay = delay.max(capped_server_delay);
                        }

                        let jitter = Duration::from_millis(generate_jitter(policy.jitter_ms));
                        let sleep_duration = delay.saturating_add(jitter).min(delay_cap);

                        // If the throttle delay would consume all remaining budget, fail now
                        let remaining = deadline.saturating_sub(start.elapsed());
                        if sleep_duration >= remaining {
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
                            delay_ms = sleep_duration.as_millis(),
                            server_requested_ms = capped_server_delay.as_millis(),
                            message = "OTLP export throttled by OTLP endpoint - delaying and retrying"
                        );
                        sleep_for(sleep_duration).await;
                        delay = delay.saturating_mul(2).min(delay_cap);
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
        assert_eq!(policy.max_retries, 3);
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

    // -- Throttle/backoff interaction tests --
    //
    // These tests form a progression: server delay within max_delay_ms (the
    // easy case where the configured cap doesn't constrain growth), then
    // server delay exceeding max_delay_ms (the case where the cap must be
    // lifted to MAX_THROTTLE_DURATION to allow continued growth).

    #[tokio::test]
    async fn test_throttle_seeds_backoff_within_max() {
        // Server delay (50ms) is below max_delay_ms (5000ms), so the
        // configured cap never constrains growth. Verifies the basic
        // mechanism: server delay raises the backoff base from initial.
        let policy = RetryPolicy {
            max_retries: 2,
            initial_delay_ms: 10,
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
        assert!(elapsed >= Duration::from_millis(50));
        assert!(elapsed < Duration::from_millis(500));
    }

    #[tokio::test]
    async fn test_repeated_throttle_doubles_within_max() {
        // Repeated throttles (20ms each) with max_delay_ms = 100ms. Server
        // delay is within the cap, so doubling works without needing to lift
        // the cap: 20ms -> 40ms. Total >= 55ms.
        let policy = RetryPolicy {
            max_retries: 2,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            jitter_ms: 0,
        };
        let attempts = AtomicUsize::new(0);
        let start = Instant::now();

        let result = retry_with_backoff(
            &policy,
            Duration::from_secs(2),
            |_: &usize| RetryErrorType::Throttled(Duration::from_millis(20)),
            "test_operation",
            || {
                let attempt = attempts.fetch_add(1, Ordering::SeqCst);
                Box::pin(async move {
                    if attempt < 2 {
                        Err(attempt)
                    } else {
                        Ok("success")
                    }
                })
            },
        )
        .await;

        assert_eq!(result, Ok("success"));
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
        assert!(start.elapsed() >= Duration::from_millis(55));
    }

    #[tokio::test]
    async fn test_retryable_after_throttle_continues_backoff_within_max() {
        // Throttle (20ms) then a retryable error, with max_delay_ms = 100ms.
        // Server delay is within the cap, so backoff doubles normally:
        // 20ms (throttled) -> 40ms (retryable, doubled from 20ms base).
        let policy = RetryPolicy {
            max_retries: 2,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            jitter_ms: 0,
        };
        let attempts = AtomicUsize::new(0);
        let start = Instant::now();

        let result = retry_with_backoff(
            &policy,
            Duration::from_secs(2),
            |attempt: &usize| {
                if *attempt == 0 {
                    RetryErrorType::Throttled(Duration::from_millis(20))
                } else {
                    RetryErrorType::Retryable
                }
            },
            "test_operation",
            || {
                let attempt = attempts.fetch_add(1, Ordering::SeqCst);
                Box::pin(async move {
                    if attempt < 2 {
                        Err(attempt)
                    } else {
                        Ok("success")
                    }
                })
            },
        )
        .await;

        assert_eq!(result, Ok("success"));
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
        // First: throttled 20ms, second: retryable 40ms (doubled from 20ms base)
        assert!(start.elapsed() >= Duration::from_millis(55));
    }

    #[tokio::test]
    async fn test_retryable_after_throttle_continues_backoff_exceeding_max() {
        // Same shape as the test above, but the server delay (100ms) exceeds
        // max_delay_ms (50ms). This is the critical case: the cap must be
        // lifted to MAX_THROTTLE_DURATION so that doubling actually grows
        // the delay instead of clamping it back to the server value.
        let policy = RetryPolicy {
            max_retries: 3,
            initial_delay_ms: 10,
            max_delay_ms: 50, // intentionally below server delay
            jitter_ms: 0,
        };
        let attempts = AtomicUsize::new(0);
        let start = Instant::now();

        let result = retry_with_backoff(
            &policy,
            Duration::from_secs(10),
            |attempt: &usize| {
                if *attempt == 0 {
                    // Server asks for 100ms, which exceeds max_delay_ms (50ms)
                    RetryErrorType::Throttled(Duration::from_millis(100))
                } else {
                    RetryErrorType::Retryable
                }
            },
            "test_operation",
            || {
                let attempt = attempts.fetch_add(1, Ordering::SeqCst);
                Box::pin(async move {
                    if attempt < 3 {
                        Err(attempt)
                    } else {
                        Ok("success")
                    }
                })
            },
        )
        .await;

        assert_eq!(result, Ok("success"));
        assert_eq!(attempts.load(Ordering::SeqCst), 4);
        // Attempt 0 fails Throttled(100ms): delay = max(10ms, 100ms) = 100ms,
        //   sleep 100ms, delay -> 200ms (doubled, NOT capped back to 100ms)
        // Attempt 1 fails Retryable: sleep 200ms, delay -> 400ms
        // Attempt 2 fails Retryable: sleep 400ms, delay -> 800ms
        // Attempt 3 succeeds
        // Total: 100 + 200 + 400 = 700ms
        let elapsed = start.elapsed();
        assert!(
            elapsed >= Duration::from_millis(680),
            "Expected >= 680ms (backoff should grow beyond server delay), got {:?}",
            elapsed
        );
        assert!(
            elapsed < Duration::from_millis(1500),
            "Expected < 1500ms, got {:?}",
            elapsed
        );
    }

    #[tokio::test]
    async fn test_throttle_after_normal_backoff_does_not_reduce_delay() {
        let policy = RetryPolicy {
            max_retries: 3,
            initial_delay_ms: 50,
            max_delay_ms: 500,
            jitter_ms: 0,
        };
        let attempts = AtomicUsize::new(0);
        let start = Instant::now();

        let result = retry_with_backoff(
            &policy,
            Duration::from_secs(5),
            |attempt: &usize| {
                if *attempt == 1 {
                    // Server asks for 20ms, but backoff has already grown to 100ms
                    RetryErrorType::Throttled(Duration::from_millis(20))
                } else {
                    RetryErrorType::Retryable
                }
            },
            "test_operation",
            || {
                let attempt = attempts.fetch_add(1, Ordering::SeqCst);
                Box::pin(async move {
                    if attempt < 3 {
                        Err(attempt)
                    } else {
                        Ok("success")
                    }
                })
            },
        )
        .await;

        assert_eq!(result, Ok("success"));
        assert_eq!(attempts.load(Ordering::SeqCst), 4);
        // Attempt 0 fails Retryable: sleep 50ms, delay -> 100ms
        // Attempt 1 fails Throttled(20ms): delay = max(100ms, 20ms) = 100ms, sleep 100ms, delay -> 200ms
        // Attempt 2 fails Retryable: sleep 200ms, delay -> 400ms
        // Attempt 3 succeeds
        // Total: 50 + 100 + 200 = 350ms
        assert!(start.elapsed() >= Duration::from_millis(340));
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
        assert!(elapsed < Duration::from_millis(500));
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

    // Tests exercising the dedicated-thread (std::thread::sleep) path.
    // These use futures_executor::block_on with no Tokio runtime present.

    #[test]
    fn test_retry_succeeds_after_retries_no_tokio() {
        let policy = RetryPolicy {
            max_retries: 3,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            jitter_ms: 5,
        };
        let deadline = Duration::from_secs(10);
        let attempts = AtomicUsize::new(0);

        let start = Instant::now();
        let result = futures_executor::block_on(retry_with_backoff(
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
        ));

        let elapsed = start.elapsed();
        assert_eq!(result, Ok("success"));
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
        // Should have slept at least ~10ms for each of 2 retries
        assert!(elapsed >= Duration::from_millis(20));
    }

    #[test]
    fn test_retry_deadline_exceeded_no_tokio() {
        let policy = RetryPolicy {
            max_retries: 100,
            initial_delay_ms: 50,
            max_delay_ms: 200,
            jitter_ms: 0,
        };
        let deadline = Duration::from_millis(120);
        let attempts = AtomicUsize::new(0);

        let start = Instant::now();
        let result = futures_executor::block_on(retry_with_backoff(
            &policy,
            deadline,
            |_: &()| RetryErrorType::Retryable,
            "test_operation",
            || {
                attempts.fetch_add(1, Ordering::SeqCst);
                Box::pin(async { Err::<(), _>(()) })
            },
        ));

        let elapsed = start.elapsed();
        assert!(result.is_err());
        // Should have stopped before exhausting 100 retries due to deadline
        assert!(attempts.load(Ordering::SeqCst) < 10);
        // Shouldn't have taken much longer than the deadline
        assert!(elapsed < Duration::from_millis(500));
    }

    #[test]
    fn test_retry_throttled_uses_server_delay_no_tokio() {
        let policy = RetryPolicy {
            max_retries: 2,
            initial_delay_ms: 10,
            max_delay_ms: 5000,
            jitter_ms: 0,
        };
        let deadline = Duration::from_secs(10);
        let attempts = AtomicUsize::new(0);

        let start = Instant::now();
        let result = futures_executor::block_on(retry_with_backoff(
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
        ));

        let elapsed = start.elapsed();
        assert_eq!(result, Ok("success"));
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
        // Should have waited ~50ms (the server delay raises it from the 10ms default)
        assert!(elapsed >= Duration::from_millis(50));
        assert!(elapsed < Duration::from_millis(500));
    }
}
