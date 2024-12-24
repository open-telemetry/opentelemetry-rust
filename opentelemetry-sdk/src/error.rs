//! Wrapper for error from trace, logs and metrics part of open telemetry.
#[cfg(feature = "logs")]
use crate::logs::LogError;
#[cfg(feature = "metrics")]
use crate::metrics::MetricError;
use opentelemetry::propagation::PropagationError;
#[cfg(feature = "trace")]
use opentelemetry::trace::TraceError;
use std::sync::PoisonError;
use std::time::Duration;
use thiserror::Error;

/// Wrapper for error from both tracing and metrics part of open telemetry. This
/// gives us a common error type where we _need_ to return errors that may come
/// from various components.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[cfg(feature = "trace")]
    #[cfg_attr(docsrs, doc(cfg(feature = "trace")))]
    #[error(transparent)]
    /// Failed to export traces.
    Trace(#[from] TraceError),
    #[cfg(feature = "metrics")]
    #[cfg_attr(docsrs, doc(cfg(feature = "metrics")))]
    #[error(transparent)]
    /// An issue raised by the metrics module.
    Metric(#[from] MetricError),

    #[cfg(feature = "logs")]
    #[cfg_attr(docsrs, doc(cfg(feature = "logs")))]
    #[error(transparent)]
    /// Failed to export logs.
    Log(#[from] LogError),

    #[error(transparent)]
    /// Error happens when injecting and extracting information using propagators.
    Propagation(#[from] PropagationError),

    /// Failed to shutdown an exporter
    #[error(transparent)]
    Shutdown(#[from] ShutdownError),

    #[error("{0}")]
    /// Other types of failures not covered by the variants above.
    Other(String),
}

impl<T> From<PoisonError<T>> for Error {
    fn from(err: PoisonError<T>) -> Self {
        Error::Other(err.to_string())
    }
}

/// Errors returned by shutdown operations in the Export API.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ShutdownError {
    /// Shutdown timed out before completing.
    #[error("Shutdown timed out after {0:?}")]
    Timeout(Duration),

    /// The export client failed while holding the client lock. It is not
    /// possible to complete the shutdown and a retry will not help.
    /// This is something that should not happen and should likely emit some diagnostic.
    #[error("export client failed while holding lock; cannot retry.")]
    ClientFailed(String),

    /// An unexpected error occurred during shutdown.
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl<T> From<PoisonError<T>> for ShutdownError {
    fn from(err: PoisonError<T>) -> Self {
        ShutdownError::ClientFailed(format!("Mutex poisoned during shutdown: {}", err))
    }
}
