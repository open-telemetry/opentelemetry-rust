use std::result;
use std::sync::PoisonError;
use thiserror::Error;

use crate::export::ExportError;

/// A specialized `Result` type for metric operations.
pub type MetricResult<T> = result::Result<T, MetricError>;

/// Errors returned by the metrics API.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum MetricError {
    /// Other errors not covered by specific cases.
    #[error("Metrics error: {0}")]
    Other(String),
    /// Invalid configuration
    #[error("Config error {0}")]
    Config(String),
    /// Shutdown already invoked
    #[error("Shutdown already invoked")]
    AlreadyShutdown,
    /// Shutdown failed due to timeout exceeding
    #[error("Shutdown failed due to timeout exceeding")]
    ShutdownTimeout,
    /// Fail to export metrics
    #[error("Metrics exporter {0} failed with {name}", name = .0.exporter_name())]
    ExportErr(Box<dyn ExportError>),
    /// Invalid instrument configuration such invalid instrument name, invalid instrument description, invalid instrument unit, etc.
    /// See [spec](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/api.md#general-characteristics)
    /// for full list of requirements.
    #[error("Invalid instrument configuration: {0}")]
    InvalidInstrumentConfiguration(&'static str),
}

impl PartialEq for MetricError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (MetricError::Other(a), MetricError::Other(b)) => a == b,
            (MetricError::Config(a), MetricError::Config(b)) => a == b,
            (MetricError::AlreadyShutdown, MetricError::AlreadyShutdown) => true,
            (MetricError::ShutdownTimeout, MetricError::ShutdownTimeout) => true,
            (MetricError::ExportErr(a), MetricError::ExportErr(b)) => {
                a.exporter_name() == b.exporter_name()
            }
            (
                MetricError::InvalidInstrumentConfiguration(a),
                MetricError::InvalidInstrumentConfiguration(b),
            ) => a == b,
            _ => false,
        }
    }
}

impl<T: ExportError> From<T> for MetricError {
    fn from(err: T) -> Self {
        MetricError::ExportErr(Box::new(err))
    }
}

impl<T> From<PoisonError<T>> for MetricError {
    fn from(err: PoisonError<T>) -> Self {
        MetricError::Other(err.to_string())
    }
}
