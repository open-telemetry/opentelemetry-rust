//! Log exporters
use crate::{log::LogRecord, Resource};
use async_trait::async_trait;
use opentelemetry_api::log::LogError;
use opentelemetry_api::{log::LogResult, InstrumentationLibrary};
use std::{fmt::Debug, sync::Arc};

pub mod stdout;

/// `LogExporter` defines the interface that log exporters should implement.
#[async_trait]
pub trait LogExporter: Send + Debug {
    /// Exports a batch of `ResourceLogs`.
    async fn export(&mut self, batch: Vec<ResourceLog>) -> LogResult<()>;
    /// Shuts down the expoter.
    fn shutdown(&mut self) {}
}

/// `ResourceLog` associates a [`LogRecord`] with a [`Resource`] and
/// [`InstrumentationLibrary`].
#[derive(Debug)]
#[non_exhaustive]
pub struct ResourceLog {
    /// Log record
    pub record: LogRecord,
    /// Resource for the emitter who produced this `ResourceLog`.
    pub resource: Option<Arc<Resource>>,
    /// Instrumentation details for the emitter who produced this `ResourceLog`.
    pub instrumentation: InstrumentationLibrary,
}

/// Describes the result of an export.
pub type ExportResult = Result<(), LogError>;
