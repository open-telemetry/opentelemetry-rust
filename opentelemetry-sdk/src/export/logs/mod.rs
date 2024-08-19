//! Log exporters
use crate::logs::LogRecord;
use crate::Resource;
use async_trait::async_trait;
#[cfg(feature = "logs_level_enabled")]
use opentelemetry::logs::Severity;
use opentelemetry::{
    logs::{LogError, LogResult},
    InstrumentationLibrary,
};
use std::borrow::Cow;
use std::fmt::Debug;

/// `LogExporter` defines the interface that log exporters should implement.
#[async_trait]
pub trait LogExporter: Send + Sync + Debug {
    /// Exports a batch of [`LogData`].
    async fn export<'a>(&mut self, batch: Vec<Cow<'a, LogData<'a>>>) -> LogResult<()>;
    /// Shuts down the exporter.
    fn shutdown(&mut self) {}
    #[cfg(feature = "logs_level_enabled")]
    /// Chek if logs are enabled.
    fn event_enabled(&self, _level: Severity, _target: &str, _name: &str) -> bool {
        // By default, all logs are enabled
        true
    }
    /// Set the resource for the exporter.
    fn set_resource(&mut self, _resource: &Resource) {}
}

/// `LogData` represents a single log event without resource context.
#[derive(Clone, Debug)]
pub struct LogData<'a> {
    /// Log record, which can be borrowed or owned.
    pub record: Cow<'a, LogRecord>,
    /// Instrumentation details for the emitter who produced this `LogEvent`.
    pub instrumentation: Cow<'a, InstrumentationLibrary>,
}

/// Describes the result of an export.
pub type ExportResult = Result<(), LogError>;
