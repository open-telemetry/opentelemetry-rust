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
use std::fmt::Debug;

/// A batch of log records to be exported by a `LogExporter`.
///
/// The `LogBatch` struct holds a collection of log records along with their associated
/// instrumentation libraries. This structure is used to group log records together for efficient
/// export operations.
///
/// # Type Parameters
/// - `'a`: The lifetime of the references to the log records and instrumentation libraries.
///
#[derive(Debug)]
pub struct LogBatch<'a> {
    /// The data field contains a slice of tuples, where each tuple consists of a reference to
    /// a `LogRecord` and a reference to an `InstrumentationLibrary`.
    data: &'a [(&'a LogRecord<'a>, &'a InstrumentationLibrary)],
}

impl<'a> LogBatch<'a> {
    /// Creates a new instance of `LogBatch`.
    ///
    /// # Arguments
    ///
    /// * `data` - A slice of tuples, where each tuple consists of a reference to a `LogRecord`
    ///   and a reference to an `InstrumentationLibrary`. These tuples represent the log records
    ///   and their associated instrumentation libraries to be exported.
    ///
    /// # Returns
    ///
    /// A `LogBatch` instance containing the provided log records and instrumentation libraries.
    ///
    /// Note - this is not a public function, and should not be used directly. This would be
    /// made private in the future.

    pub fn new(data: &'a [(&'a LogRecord<'a>, &'a InstrumentationLibrary)]) -> LogBatch<'a> {
        LogBatch { data }
    }
}

impl<'a> LogBatch<'a> {
    /// Returns an iterator over the log records and instrumentation libraries in the batch.
    ///
    /// Each item yielded by the iterator is a tuple containing references to a `LogRecord`
    /// and an `InstrumentationLibrary`.
    ///
    /// # Returns
    ///
    /// An iterator that yields references to the `LogRecord` and `InstrumentationLibrary` in the batch.
    ///
    pub fn iter(&self) -> impl Iterator<Item = (&LogRecord<'a>, &InstrumentationLibrary)> {
        self.data
            .iter()
            .map(|(record, library)| (*record, *library))
    }
}

/// `LogExporter` defines the interface that log exporters should implement.
#[async_trait]
pub trait LogExporter: Send + Sync + Debug {
    /// Exports a batch of log records and their associated instrumentation libraries.
    ///
    /// The `export` method is responsible for sending a batch of log records to an external
    /// destination. It takes a `LogBatch` as an argument, which contains references to the
    /// log records and their corresponding instrumentation libraries. The method returns
    /// a `LogResult` indicating the success or failure of the export operation.
    ///
    /// # Arguments
    ///
    /// * `batch` - A `LogBatch` containing the log records and instrumentation libraries
    ///   to be exported.
    ///
    /// # Returns
    ///
    /// A `LogResult<()>`, which is a result type indicating either a successful export (with
    /// `Ok(())`) or an error (`Err(LogError)`) if the export operation failed.
    ///
    async fn export(&mut self, batch: LogBatch<'_>) -> LogResult<()>;
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

/// Describes the result of an export.
pub type ExportResult = Result<(), LogError>;
