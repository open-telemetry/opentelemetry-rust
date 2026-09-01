/// Configuration for retry policy.
///
/// The default is [`RetryPolicy::recommended()`]. Use
/// [`RetryPolicy::disabled()`] to perform a single export attempt without
/// retrying.
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
    /// Returns a retry policy that performs a single export attempt with no retries.
    pub fn disabled() -> Self {
        Self {
            max_retries: 0,
            initial_delay_ms: 0,
            max_delay_ms: 0,
            jitter_ms: 0,
        }
    }

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
