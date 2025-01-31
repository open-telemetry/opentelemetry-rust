//! Wrapper for error from trace, logs and metrics part of open telemetry.

use std::{result::Result, time::Duration};

use thiserror::Error;

/// Trait for errors returned by exporters
pub trait ExportError: std::error::Error + Send + Sync + 'static {
    /// The name of exporter that returned this error
    fn exporter_name(&self) -> &'static str;
}

#[derive(Error, Debug)]
/// Errors that can occur during shutdown.
pub enum ShutdownError {
    /// Shutdown already invoked.
    #[error("Shutdown already invoked")]
    AlreadyShutdown,

    /// Shutdown timed out before completing.
    #[error("Shutdown timed out after {0:?}")]
    Timeout(Duration),

    /// Shutdown failed with an error.
    /// This error is returned when the shutdown process failed with an error.
    #[error("Shutdown failed: {0}")]
    InternalFailure(String),
}

/// A specialized `Result` type for Shutdown operations.
pub type ShutdownResult = Result<(), ShutdownError>;
