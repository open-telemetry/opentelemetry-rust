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
    /// Exports a batch of readable logs. Protocol exporters that will
    /// implement this function are typically expected to serialize and transmit
    /// the data to the destination.
    ///
    /// This function will never be called concurrently for the same exporter
    /// instance. It can be called again only after the current call returns.
    ///
    /// This function must not block indefinitely, there must be a reasonable
    /// upper limit after which the call must time out with an error result.
    ///
    /// Any retry logic that is required by the exporter is the responsibility
    /// of the exporter.
    async fn export<'a>(&mut self, batch: Vec<Cow<'a, LogData>>) -> LogResult<()>;
    /// Shuts down the exporter. Called when SDK is shut down. This is an
    /// opportunity for exporter to do any cleanup required.
    ///
    /// This function should be called only once for each `LogExporter`
    /// instance. After the call to `shutdown`, subsequent calls to `export` are
    /// not allowed and should return an error.
    ///
    /// This function should not block indefinitely (e.g. if it attempts to
    /// flush the data and the destination is unavailable). SDK authors
    /// can decide if they want to make the shutdown timeout
    /// configurable.
    fn shutdown(&mut self) {}
    #[cfg(feature = "logs_level_enabled")]
    /// Chek if logs are enabled.
    fn event_enabled(&self, _level: Severity, _target: &str, _name: &str) -> bool {
        true
    }
    /// Set the resource for the exporter.
    /// This function SHOULD only be called once during the initialization of the exporter.
    /// This function SHOULD complete or abort within some timeout. This function SHOULD be
    /// implemented as a blocking API
    fn set_resource(&mut self, _resource: &Resource) -> LogResult<()> {
        Ok(())
    }
}

/// `LogData` represents a single log event without resource context.
#[derive(Clone, Debug)]
pub struct LogData {
    /// Log record
    pub record: LogRecord,
    /// Instrumentation details for the emitter who produced this `LogEvent`.
    pub instrumentation: InstrumentationLibrary,
}

/// Describes the result of an export.
pub type ExportResult = Result<(), LogError>;
