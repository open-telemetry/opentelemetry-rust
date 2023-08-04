//! Log exporters
use crate::Resource;
use async_trait::async_trait;
#[cfg(feature = "logs_level_enabled")]
use opentelemetry_api::logs::Severity;
use opentelemetry_api::{
    logs::{LogError, LogRecord, LogResult},
    InstrumentationLibrary,
};
use std::{borrow::Cow, fmt::Debug};

/// `LogExporter` defines the interface that log exporters should implement.
#[async_trait]
pub trait LogExporter: Send + Debug {
    /// Exports a batch of [`LogData`].
    async fn export(&mut self, batch: Vec<LogData>) -> LogResult<()>;
    /// Shuts down the exporter.
    fn shutdown(&mut self) {}
    #[cfg(feature = "logs_level_enabled")]
    /// Chek if logs are enabled.
    fn event_enabled(&self, _level: Severity, _target: &str, _name: &str) -> bool {
        true
    }
}

/// `LogData` associates a [`LogRecord`] with a [`Resource`] and
/// [`InstrumentationLibrary`].
#[derive(Debug)]
#[non_exhaustive]
pub struct LogData {
    /// Log record
    pub record: LogRecord,
    /// Resource for the emitter who produced this `LogData`.
    pub resource: Cow<'static, Resource>,
    /// Instrumentation details for the emitter who produced this `LogData`.
    pub instrumentation: InstrumentationLibrary,
}

/// Describes the result of an export.
pub type ExportResult = Result<(), LogError>;
