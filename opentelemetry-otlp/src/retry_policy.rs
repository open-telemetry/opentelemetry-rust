use std::time::Duration;

/// Configuration for retry policy.
///
/// The default is [`RetryPolicy::recommended()`]. Use
/// [`RetryPolicy::disabled()`] to perform a single export attempt without
/// retrying.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub(crate) max_retries: usize,
    pub(crate) initial_delay: Duration,
    pub(crate) max_delay: Duration,
    pub(crate) max_jitter: Duration,
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
            initial_delay: Duration::ZERO,
            max_delay: Duration::ZERO,
            max_jitter: Duration::ZERO,
        }
    }

    /// Recommended retry policy per the OTLP spec: 3 retries with exponential
    /// backoff (100ms initial, 1600ms max, 100ms jitter).
    pub fn recommended() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_millis(1600),
            max_jitter: Duration::from_millis(100),
        }
    }

    /// Sets the maximum number of retry attempts after the initial export attempt.
    pub fn with_max_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Sets the delay before the first retry attempt.
    pub fn with_initial_delay(mut self, initial_delay: Duration) -> Self {
        self.initial_delay = initial_delay;
        self
    }

    /// Sets the maximum delay between retry attempts.
    pub fn with_max_delay(mut self, max_delay: Duration) -> Self {
        self.max_delay = max_delay;
        self
    }

    /// Sets the maximum random jitter added to each retry delay.
    pub fn with_max_jitter(mut self, max_jitter: Duration) -> Self {
        self.max_jitter = max_jitter;
        self
    }
}
