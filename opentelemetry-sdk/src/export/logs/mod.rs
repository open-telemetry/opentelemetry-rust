//! Log exporters
use crate::logs::LogRecord;
use crate::logs::{LogError, LogResult};
use crate::Resource;
use async_trait::async_trait;
use futures_util::future::BoxFuture;
#[cfg(feature = "logs_level_enabled")]
use opentelemetry::logs::Severity;
use opentelemetry::InstrumentationScope;
use std::fmt::Debug;

/// A batch of log records to be exported by a `LogExporter`.
///
/// The `LogBatch` struct holds a collection of log records along with their associated
/// instrumentation scopes. This structure is used to group log records together for efficient
/// export operations.
///
/// # Type Parameters
/// - `'a`: The lifetime of the references to the log records and instrumentation scopes.
///
#[derive(Debug)]
pub struct LogBatch<'a> {
    /// The data field contains a slice of tuples, where each tuple consists of a reference to
    /// a `LogRecord` and a reference to an `InstrumentationScope`.
    data: &'a [(&'a LogRecord, &'a InstrumentationScope)],
}

impl<'a> LogBatch<'a> {
    /// Creates a new instance of `LogBatch`.
    ///
    /// # Arguments
    ///
    /// * `data` - A slice of tuples, where each tuple consists of a reference to a `LogRecord`
    ///   and a reference to an `InstrumentationScope`. These tuples represent the log records
    ///   and their associated instrumentation scopes to be exported.
    ///
    /// # Returns
    ///
    /// A `LogBatch` instance containing the provided log records and instrumentation scopes.
    ///
    /// Note - this is not a public function, and should not be used directly. This would be
    /// made private in the future.

    pub fn new(data: &'a [(&'a LogRecord, &'a InstrumentationScope)]) -> LogBatch<'a> {
        LogBatch { data }
    }
}

impl LogBatch<'_> {
    /// Returns an iterator over the log records and instrumentation scopes in the batch.
    ///
    /// Each item yielded by the iterator is a tuple containing references to a `LogRecord`
    /// and an `InstrumentationScope`.
    ///
    /// # Returns
    ///
    /// An iterator that yields references to the `LogRecord` and `InstrumentationScope` in the batch.
    ///
    pub fn iter(&self) -> impl Iterator<Item = (&LogRecord, &InstrumentationScope)> {
        self.data
            .iter()
            .map(|(record, library)| (*record, *library))
    }
}

/// `LogExporter` defines the interface that log exporters should implement.
#[async_trait]
pub trait LogExporter: Send + Sync + Debug {
    /// Exports a batch of log records and their associated instrumentation scopes.
    ///
    /// The `export` method is responsible for sending a batch of log records to an external
    /// destination. It takes a `LogBatch` as an argument, which contains references to the
    /// log records and their corresponding instrumentation scopes. The method returns
    /// a `LogResult` indicating the success or failure of the export operation.
    ///
    /// # Arguments
    ///
    /// * `batch` - A `LogBatch` containing the log records and instrumentation scopes
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

    /// This is a hint to ensure that the export of any Logs the exporter
    /// has received prior to the call to this function SHOULD be completed
    /// as soon as possible, preferably before returning from this method.
    ///
    /// This function SHOULD provide a way to let the caller know
    /// whether it succeeded, failed or timed out.
    ///
    /// This function SHOULD only be called in cases where it is absolutely necessary,
    /// such as when using some FaaS providers that may suspend the process after
    /// an invocation, but before the exporter exports the completed logs.
    ///
    /// This function SHOULD complete or abort within some timeout. This function can be
    /// implemented as a blocking API or an asynchronous API which notifies the caller via
    /// a callback or an event. OpenTelemetry client authors can decide if they want to
    /// make the flush timeout configurable.
    fn force_flush(&mut self) -> BoxFuture<'static, ExportResult> {
        Box::pin(async { Ok(()) })
    }
    /// Set the resource for the exporter.
    fn set_resource(&mut self, _resource: &Resource) {}
}

/// Describes the result of an export.
pub type ExportResult = Result<(), LogError>;
