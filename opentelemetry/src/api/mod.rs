#[cfg(feature = "metrics")]
#[cfg_attr(docsrs, doc(cfg(feature = "metrics")))]
use crate::api::metrics::MetricsError;
#[cfg(feature = "trace")]
#[cfg_attr(docsrs, doc(cfg(feature = "trace")))]
use crate::api::trace::TraceError;
use std::sync::PoisonError;

pub mod baggage;
pub(crate) mod context;
pub(crate) mod core;
#[cfg(feature = "metrics")]
#[cfg_attr(docsrs, doc(cfg(feature = "metrics")))]
pub mod labels;
#[cfg(feature = "metrics")]
#[cfg_attr(docsrs, doc(cfg(feature = "metrics")))]
pub mod metrics;
pub mod propagation;
#[cfg(feature = "trace")]
#[cfg_attr(docsrs, doc(cfg(feature = "trace")))]
pub mod trace;

/// Wrapper for error from both tracing and metrics part of open telemetry.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[cfg(feature = "trace")]
    #[cfg_attr(docsrs, doc(cfg(feature = "trace")))]
    #[error(transparent)]
    Trace(#[from] TraceError),
    #[cfg(feature = "metrics")]
    #[cfg_attr(docsrs, doc(cfg(feature = "metrics")))]
    #[error(transparent)]
    Metric(#[from] MetricsError),
    #[error("{0}")]
    Other(String),
}

impl<T> From<PoisonError<T>> for Error {
    fn from(err: PoisonError<T>) -> Self {
        Error::Other(err.to_string())
    }
}
