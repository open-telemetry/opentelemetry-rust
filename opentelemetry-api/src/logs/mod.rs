//! # OpenTelemetry Logs API

use crate::ExportError;
use futures_channel::{mpsc::TrySendError, oneshot::Canceled};
use std::time::Duration;
use thiserror::Error;

mod logger;
mod noop;
mod record;

pub use logger::{Logger, LoggerProvider};
pub use noop::NoopLoggerProvider;
pub use record::{AnyValue, LogRecord, LogRecordBuilder, Severity, TraceContext};

/// Describe the result of operations in log SDK.
pub type LogResult<T> = Result<T, LogError>;

#[derive(Error, Debug)]
#[non_exhaustive]
/// Errors returned by the log SDK.
pub enum LogError {
    /// Export failed with the error returned by the exporter.
    #[error("Exporter {} encountered the following errors: {0}", .0.exporter_name())]
    ExportFailed(Box<dyn ExportError>),

    /// Export failed to finish after certain period and processor stopped the export.
    #[error("Exporter timed out after {} seconds", .0.as_secs())]
    ExportTimedOut(Duration),

    /// Other errors propagated from log SDK that weren't covered above.
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl<T> From<T> for LogError
where
    T: ExportError,
{
    fn from(err: T) -> Self {
        LogError::ExportFailed(Box::new(err))
    }
}

impl<T> From<TrySendError<T>> for LogError {
    fn from(err: TrySendError<T>) -> Self {
        LogError::Other(Box::new(err.into_send_error()))
    }
}

impl From<Canceled> for LogError {
    fn from(err: Canceled) -> Self {
        LogError::Other(Box::new(err))
    }
}

impl From<String> for LogError {
    fn from(err_msg: String) -> Self {
        LogError::Other(Box::new(Custom(err_msg)))
    }
}

impl From<&'static str> for LogError {
    fn from(err_msg: &'static str) -> Self {
        LogError::Other(Box::new(Custom(err_msg.into())))
    }
}

/// Wrap type for string
#[derive(Error, Debug)]
#[error("{0}")]
struct Custom(String);
