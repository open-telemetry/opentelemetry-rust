//! Wrapper for error from trace, logs and metrics part of open telemetry.
use std::sync::PoisonError;

#[cfg(feature = "logs")]
use crate::logs::LogError;
#[cfg(feature = "metrics")]
use crate::metrics::MetricError;
use opentelemetry::propagation::PropagationError;
#[cfg(feature = "trace")]
use opentelemetry::trace::TraceError;

/// Wrapper for error from both tracing and metrics part of open telemetry.
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

    #[error("{0}")]
    /// Other types of failures not covered by the variants above.
    Other(String),
}

impl<T> From<PoisonError<T>> for Error {
    fn from(err: PoisonError<T>) -> Self {
        Error::Other(err.to_string())
    }
}
