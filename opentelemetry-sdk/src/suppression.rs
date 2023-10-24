use tokio::task_local;

// Define the async local storage for suppression.
task_local! {
    static SUPPRESSION_FLAG: bool;
}

/// Represents a scope within which logging is suppressed.
/// Logging is suppressed for the duration of the guard's lifetime.
#[derive(Debug)]
pub struct SuppressionGuard(bool); // Capture the original state

impl SuppressionGuard {
    /// doc1
    pub fn new() -> Self {
        let original_state = SUPPRESSION_FLAG.try_with(|&flag| flag).unwrap_or(false);
        SUPPRESSION_FLAG.scope(true, async {});
        SuppressionGuard(original_state)
    }

    /// doc2
    pub fn is_logging_suppressed() -> bool {
        SUPPRESSION_FLAG.try_with(|&flag| flag).unwrap_or(false)
    }
}

impl Drop for SuppressionGuard {
    fn drop(&mut self) {
        SUPPRESSION_FLAG.scope(self.0, async {}); // Restore the original state
    }
}
