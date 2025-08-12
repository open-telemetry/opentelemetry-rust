//! This module provides functionality for retrying operations with exponential backoff and jitter.
//!
//! The `RetryPolicy` struct defines the configuration for the retry behavior, including the maximum
//! number of retries, initial delay, maximum delay, and jitter.
//!
//! The `retry_with_exponential_backoff` function retries the given operation according to the
//! specified retry policy, using exponential backoff and jitter to determine the delay between
//! retries. The function logs errors and retries the operation until it succeeds or the maximum
//! number of retries is reached.

#[cfg(feature = "experimental_async_runtime")]
use opentelemetry::otel_warn;
#[cfg(feature = "experimental_async_runtime")]
use std::future::Future;
#[cfg(feature = "experimental_async_runtime")]
use std::time::{Duration, SystemTime};

#[cfg(feature = "experimental_async_runtime")]
use crate::runtime::Runtime;

/// Configuration for retry policy.
#[derive(Debug)]
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

/// A runtime stub for when experimental_async_runtime is not enabled.
/// This allows retry policy to be configured but no actual retries occur.
#[cfg(not(feature = "experimental_async_runtime"))]
#[derive(Debug, Clone)]
pub struct NoOpRuntime;

#[cfg(not(feature = "experimental_async_runtime"))]
impl NoOpRuntime {
    /// Creates a new no-op runtime.
    pub fn new() -> Self {
        Self
    }
}

// Generates a random jitter value up to max_jitter
#[cfg(feature = "experimental_async_runtime")]
fn generate_jitter(max_jitter: u64) -> u64 {
    let now = SystemTime::now();
    let nanos = now
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    nanos as u64 % (max_jitter + 1)
}

/// Retries the given operation with exponential backoff and jitter.
///
/// # Arguments
///
/// * `runtime` - The async runtime to use for delays.
/// * `policy` - The retry policy configuration.
/// * `operation_name` - The name of the operation being retried.
/// * `operation` - The operation to be retried.
///
/// # Returns
///
/// A `Result` containing the operation's result or an error if the maximum retries are reached.
#[cfg(feature = "experimental_async_runtime")]
pub async fn retry_with_exponential_backoff<R, F, Fut, T, E>(
    runtime: R,
    policy: RetryPolicy,
    operation_name: &str,
    mut operation: F,
) -> Result<T, E>
where
    R: Runtime,
    F: FnMut() -> Fut,
    E: std::fmt::Debug,
    Fut: Future<Output = Result<T, E>>,
{
    let mut attempt = 0;
    let mut delay = policy.initial_delay_ms;

    loop {
        match operation().await {
            Ok(result) => return Ok(result), // Return the result if the operation succeeds
            Err(err) if attempt < policy.max_retries => {
                attempt += 1;
                // Log the error and retry after a delay with jitter
                otel_warn!(name: "OtlpRetry", message = format!("Retrying operation {:?} due to error: {:?}", operation_name, err));
                let jitter = generate_jitter(policy.jitter_ms);
                let delay_with_jitter = std::cmp::min(delay + jitter, policy.max_delay_ms);
                runtime
                    .delay(Duration::from_millis(delay_with_jitter))
                    .await;
                delay = std::cmp::min(delay * 2, policy.max_delay_ms); // Exponential backoff
            }
            Err(err) => return Err(err), // Return the error if max retries are reached
        }
    }
}

/// No-op retry function for when experimental_async_runtime is not enabled.
/// This function will execute the operation exactly once without any retries.
#[cfg(not(feature = "experimental_async_runtime"))]
pub async fn retry_with_exponential_backoff<R, F, Fut, T, E>(
    _runtime: R,
    _policy: RetryPolicy,
    _operation_name: &str,
    mut operation: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    // Without experimental_async_runtime, we just execute once without retries
    operation().await
}

#[cfg(all(test, feature = "experimental_async_runtime", feature = "rt-tokio"))]
mod tests {
    use super::*;
    use crate::runtime::Tokio;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;
    use tokio::time::timeout;

    // Test to ensure generate_jitter returns a value within the expected range
    #[tokio::test]
    async fn test_generate_jitter() {
        let max_jitter = 100;
        let jitter = generate_jitter(max_jitter);
        assert!(jitter <= max_jitter);
    }

    // Test to ensure retry_with_exponential_backoff succeeds on the first attempt
    #[tokio::test]
    async fn test_retry_with_exponential_backoff_success() {
        let runtime = Tokio;
        let policy = RetryPolicy {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 1600,
            jitter_ms: 100,
        };

        let result = retry_with_exponential_backoff(runtime, policy, "test_operation", || {
            Box::pin(async { Ok::<_, ()>("success") })
        })
        .await;

        assert_eq!(result, Ok("success"));
    }

    // Test to ensure retry_with_exponential_backoff retries the operation and eventually succeeds
    #[tokio::test]
    async fn test_retry_with_exponential_backoff_retries() {
        let runtime = Tokio;
        let policy = RetryPolicy {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 1600,
            jitter_ms: 100,
        };

        let attempts = AtomicUsize::new(0);

        let result = retry_with_exponential_backoff(runtime, policy, "test_operation", || {
            let attempt = attempts.fetch_add(1, Ordering::SeqCst);
            Box::pin(async move {
                if attempt < 2 {
                    Err::<&str, &str>("error") // Fail the first two attempts
                } else {
                    Ok::<&str, &str>("success") // Succeed on the third attempt
                }
            })
        })
        .await;

        assert_eq!(result, Ok("success"));
        assert_eq!(attempts.load(Ordering::SeqCst), 3); // Ensure there were 3 attempts
    }

    // Test to ensure retry_with_exponential_backoff fails after max retries
    #[tokio::test]
    async fn test_retry_with_exponential_backoff_failure() {
        let runtime = Tokio;
        let policy = RetryPolicy {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 1600,
            jitter_ms: 100,
        };

        let attempts = AtomicUsize::new(0);

        let result = retry_with_exponential_backoff(runtime, policy, "test_operation", || {
            attempts.fetch_add(1, Ordering::SeqCst);
            Box::pin(async { Err::<(), _>("error") }) // Always fail
        })
        .await;

        assert_eq!(result, Err("error"));
        assert_eq!(attempts.load(Ordering::SeqCst), 4); // Ensure there were 4 attempts (initial + 3 retries)
    }

    // Test to ensure retry_with_exponential_backoff respects the timeout
    #[tokio::test]
    async fn test_retry_with_exponential_backoff_timeout() {
        let runtime = Tokio;
        let policy = RetryPolicy {
            max_retries: 12, // Increase the number of retries
            initial_delay_ms: 100,
            max_delay_ms: 1600,
            jitter_ms: 100,
        };

        let result = timeout(
            Duration::from_secs(1),
            retry_with_exponential_backoff(runtime, policy, "test_operation", || {
                Box::pin(async { Err::<(), _>("error") }) // Always fail
            }),
        )
        .await;

        assert!(result.is_err()); // Ensure the operation times out
    }
}
