#[cfg(feature = "metrics")]
#[cfg_attr(docsrs, doc(cfg(feature = "metrics")))]
use crate::api::metrics::MetricsError;
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
#[derive(Debug)]
pub enum OpenTelemetryError {
    #[cfg(feature = "trace")]
    #[cfg_attr(docsrs, doc(cfg(feature = "trace")))]
    TraceErr(TraceError),
    #[cfg(feature = "metrics")]
    #[cfg_attr(docsrs, doc(cfg(feature = "metrics")))]
    MetricErr(MetricsError),
    Other(String),
}

#[cfg(feature = "trace")]
#[cfg_attr(docsrs, doc(cfg(feature = "trace")))]
impl From<TraceError> for OpenTelemetryError {
    fn from(err: TraceError) -> Self {
        OpenTelemetryError::TraceErr(err)
    }
}

#[cfg(feature = "metrics")]
#[cfg_attr(docsrs, doc(cfg(feature = "metrics")))]
impl From<MetricsError> for OpenTelemetryError {
    fn from(err: MetricsError) -> Self {
        OpenTelemetryError::MetricErr(err)
    }
}

impl<T> From<PoisonError<T>> for OpenTelemetryError {
    fn from(err: PoisonError<T>) -> Self {
        OpenTelemetryError::Other(err.to_string())
    }
}
