//! Wrapper for error from trace, logs and metrics part of open telemetry.

use std::{fmt, result::Result, time::Duration};

/// Trait for errors returned by exporters
pub trait ExportError: std::error::Error + Send + Sync + 'static {
    /// The name of exporter that returned this error
    fn exporter_name(&self) -> &'static str;
}

#[derive(Debug)]
/// Errors that can occur during SDK operations export(), force_flush() and shutdown().
pub enum OTelSdkError {
    /// Shutdown has already been invoked.
    ///
    /// While shutdown is idempotent and calling it multiple times has no
    /// impact, this error suggests that another part of the application is
    /// invoking `shutdown` earlier than intended. Users should review their
    /// code to identify unintended or duplicate shutdown calls and ensure it is
    /// only triggered once at the correct place.
    AlreadyShutdown,

    /// Operation timed out before completing.
    ///
    /// This does not necessarily indicate a failure—operation may still be
    /// complete. If this occurs frequently, consider increasing the timeout
    /// duration to allow more time for completion.
    Timeout(Duration),

    /// Operation failed due to an internal error.
    ///
    /// The error message is intended for logging purposes only and should not
    /// be used to make programmatic decisions. It is implementation-specific
    /// and subject to change without notice. Consumers of this error should not
    /// rely on its content beyond logging.
    InternalFailure(String),
}

impl fmt::Display for OTelSdkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OTelSdkError::AlreadyShutdown => write!(f, "Shutdown already invoked"),
            OTelSdkError::Timeout(duration) => {
                write!(f, "Operation timed out after {duration:?}")
            }
            OTelSdkError::InternalFailure(reason) => write!(f, "Operation failed: {reason}"),
        }
    }
}

impl std::error::Error for OTelSdkError {}

#[cfg(any(feature = "testing", test))]
impl<T> From<std::sync::PoisonError<T>> for OTelSdkError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        OTelSdkError::InternalFailure(format!("Mutex poison error: {err}"))
    }
}

/// A specialized `Result` type for Shutdown operations.
pub type OTelSdkResult = Result<(), OTelSdkError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_matches_variants() {
        assert_eq!(
            OTelSdkError::AlreadyShutdown.to_string(),
            "Shutdown already invoked"
        );
        assert_eq!(
            OTelSdkError::Timeout(Duration::from_secs(5)).to_string(),
            "Operation timed out after 5s"
        );
        assert_eq!(
            OTelSdkError::InternalFailure("db unreachable".into()).to_string(),
            "Operation failed: db unreachable"
        );
    }
}
