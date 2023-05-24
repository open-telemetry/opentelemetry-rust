//! Log exporters
use crate::Resource;
use async_trait::async_trait;
use opentelemetry_api::{
    logs::{LogRecord, LogResult},
    InstrumentationLibrary,
};
use std::{borrow::Cow, fmt::Debug};

/// Describes the result of an export.
pub type ExportResult = LogResult<()>;

/// `LogExporter` defines the interface that log exporters should implement.
#[async_trait]
pub trait LogExporter: Send + Debug {
    /// Exports a batch of `ResourceLogs`.
    async fn export(&mut self, batch: Vec<LogData>) -> ExportResult;
    /// Shuts down the expoter.
    fn shutdown(&mut self) {}
}

/// `LogData` associates a [`LogRecord`] with a [`Resource`] and
/// [`InstrumentationLibrary`].
#[derive(Debug)]
pub struct LogData {
    /// Log record
    pub record: LogRecord,
    /// Resource for the emitter who produced this `LogData`.
    pub resource: Cow<'static, Resource>,
    /// Instrumentation details for the emitter who produced this `LogData`.
    pub instrumentation: InstrumentationLibrary,
}
