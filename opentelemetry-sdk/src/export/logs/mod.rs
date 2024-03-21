//! Log exporters
use crate::Resource;
use async_trait::async_trait;
#[cfg(feature = "logs_level_enabled")]
use opentelemetry::logs::Severity;
use opentelemetry::{
    logs::{LogError, LogRecord, LogResult},
    InstrumentationLibrary,
};
use std::{borrow::Cow, fmt::Debug};

/// `LogExporter` defines the interface that log exporters should implement.
#[async_trait]
pub trait LogExporter: Send + Sync + Debug {
    /// Exports a batch of [`LogData`].
    async fn export(&mut self, batch: Vec<LogEvent>) -> LogResult<()>;
    /// Shuts down the exporter.
    fn shutdown(&mut self) {}
    #[cfg(feature = "logs_level_enabled")]
    /// Chek if logs are enabled.
    fn event_enabled(&self, _level: Severity, _target: &str, _name: &str) -> bool {
        true
    }
    /// Set the resource for the exporter.
    fn set_resource(&mut self, resource: &Resource);
}
/// `LogEvent` represents a single log event without resource context.
#[derive(Clone, Debug)]
pub struct LogEvent {
    /// Log record
    pub record: LogRecord,
    /// Instrumentation details for the emitter who produced this `LogEvent`.
    pub instrumentation: InstrumentationLibrary,
}

/// `LogData` associates a [`LogRecord`] with a [`Resource`] and
/// [`InstrumentationLibrary`].
#[derive(Clone, Debug)]
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
