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

fn generate_jitter(max_jitter: u64) -> u64 {
    let now = SystemTime::now();
    let nanos = now.duration_since(SystemTime::UNIX_EPOCH).unwrap().subsec_nanos();
    nanos as u64 % (max_jitter + 1)
}

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
            Ok(result) => return Ok(result),
            Err(err) if attempt < policy.max_retries => {
                attempt += 1;
                // Log the error and retry after a delay with jitter
                otel_warn!(name: "OtlpRetry", message = format!("Retrying operation {:?} due to error: {:?}", operation_name, err));
                let jitter = generate_jitter(policy.jitter_ms);
                let delay_with_jitter = std::cmp::min(delay + jitter, policy.max_delay_ms);
                sleep(Duration::from_millis(delay_with_jitter)).await;
                delay = std::cmp::min(delay * 2, policy.max_delay_ms);
            }
            Err(err) => return Err(err),
        }
    }
}