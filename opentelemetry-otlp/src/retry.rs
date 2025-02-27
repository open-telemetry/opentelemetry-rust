use std::future::Future;
use std::time::{Duration, SystemTime};
use opentelemetry::otel_warn;
use tokio::time::sleep;

pub(crate) struct RetryPolicy {
    pub max_retries: usize,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub jitter_ms: u64,
}

// Generates a random jitter value up to max_jitter
fn generate_jitter(max_jitter: u64) -> u64 {
    let now = SystemTime::now();
    let nanos = now.duration_since(SystemTime::UNIX_EPOCH).unwrap().subsec_nanos();
    nanos as u64 % (max_jitter + 1)
}

// Retries the given operation with exponential backoff and jitter
pub(crate) async fn retry_with_exponential_backoff<F, Fut, T, E>(
    policy: RetryPolicy,
    operation_name: &str,
    mut operation: F,
) -> Result<T, E>
where
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
                sleep(Duration::from_millis(delay_with_jitter)).await;
                delay = std::cmp::min(delay * 2, policy.max_delay_ms); // Exponential backoff
            }
            Err(err) => return Err(err), // Return the error if max retries are reached
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

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
        let policy = RetryPolicy {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 1600,
            jitter_ms: 100,
        };

        let result = retry_with_exponential_backoff(policy, "test_operation", || {
            Box::pin(async { Ok::<_, ()>("success") })
        }).await;

        assert_eq!(result, Ok("success"));
    }

    // Test to ensure retry_with_exponential_backoff retries the operation and eventually succeeds
    #[tokio::test]
    async fn test_retry_with_exponential_backoff_retries() {
        let policy = RetryPolicy {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 1600,
            jitter_ms: 100,
        };

        let attempts = AtomicUsize::new(0);

        let result = retry_with_exponential_backoff(policy, "test_operation", || {
            let attempt = attempts.fetch_add(1, Ordering::SeqCst);
            Box::pin(async move {
                if attempt < 2 {
                    Err::<&str, &str>("error") // Fail the first two attempts
                } else {
                    Ok::<&str, &str>("success") // Succeed on the third attempt
                }
            })
        }).await;

        assert_eq!(result, Ok("success"));
        assert_eq!(attempts.load(Ordering::SeqCst), 3); // Ensure there were 3 attempts
    }

    // Test to ensure retry_with_exponential_backoff fails after max retries
    #[tokio::test]
    async fn test_retry_with_exponential_backoff_failure() {
        let policy = RetryPolicy {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 1600,
            jitter_ms: 100,
        };

        let attempts = AtomicUsize::new(0);

        let result = retry_with_exponential_backoff(policy, "test_operation", || {
            attempts.fetch_add(1, Ordering::SeqCst);
            Box::pin(async { Err::<(), _>("error") }) // Always fail
        }).await;

        assert_eq!(result, Err("error"));
        assert_eq!(attempts.load(Ordering::SeqCst), 4); // Ensure there were 4 attempts (initial + 3 retries)
    }

    // Test to ensure retry_with_exponential_backoff respects the timeout
    #[tokio::test]
    async fn test_retry_with_exponential_backoff_timeout() {
        let policy = RetryPolicy {
            max_retries: 12, // Increase the number of retries
            initial_delay_ms: 100,
            max_delay_ms: 1600,
            jitter_ms: 100,
        };

        let result = timeout(Duration::from_secs(1), retry_with_exponential_backoff(policy, "test_operation", || {
            Box::pin(async { Err::<(), _>("error") }) // Always fail
        })).await;

        assert!(result.is_err()); // Ensure the operation times out
    }
}