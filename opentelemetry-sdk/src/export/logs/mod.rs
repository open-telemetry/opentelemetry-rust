//! Log exporters
use crate::{logs::LogRecord, Resource};
use async_trait::async_trait;
use opentelemetry_api::logs::LogError;
use opentelemetry_api::{logs::LogResult, InstrumentationLibrary};
use std::{fmt::Debug, sync::Arc};

pub mod stdout;

/// `LogExporter` defines the interface that log exporters should implement.
#[async_trait]
pub trait LogExporter: Send + Debug {
    /// Exports a batch of `ResourceLogs`.
    async fn export(&mut self, batch: Vec<LogData>) -> LogResult<()>;
    /// Shuts down the expoter.
    fn shutdown(&mut self) {}
}

/// `LogData` associates a [`LogRecord`] with a [`Resource`] and
/// [`InstrumentationLibrary`].
#[derive(Debug)]
#[non_exhaustive]
pub struct LogData {
    /// Log record
    pub record: LogRecord,
    /// Resource for the emitter who produced this `LogData`.
    pub resource: Option<Arc<Resource>>,
    /// Instrumentation details for the emitter who produced this `LogData`.
    pub instrumentation: InstrumentationLibrary,
}

/// Describes the result of an export.
pub type ExportResult = Result<(), LogError>;
