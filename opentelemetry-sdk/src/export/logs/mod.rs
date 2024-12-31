//! Log exporters
use crate::logs::LogRecord;
use crate::logs::{LogError, LogResult};
use crate::Resource;
#[cfg(feature = "spec_unstable_logs_enabled")]
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
    data: LogBatchData<'a>,
}

/// The `LogBatchData` enum represents the data field of a `LogBatch`.
/// It can either be:
/// - A shared reference to a slice of boxed tuples, where each tuple consists of an owned `LogRecord` and an owned `InstrumentationScope`.
/// - Or it can be a shared reference to a slice of tuples, where each tuple consists of a reference to a `LogRecord` and a reference to an `InstrumentationScope`.
#[derive(Debug)]
enum LogBatchData<'a> {
    SliceOfOwnedData(&'a [Box<(LogRecord, InstrumentationScope)>]), // Used by BatchProcessor which clones the LogRecords for its own use.
    SliceOfBorrowedData(&'a [(&'a LogRecord, &'a InstrumentationScope)]),
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
        LogBatch {
            data: LogBatchData::SliceOfBorrowedData(data),
        }
    }

    pub(crate) fn new_with_owned_data(
        data: &'a [Box<(LogRecord, InstrumentationScope)>],
    ) -> LogBatch<'a> {
        LogBatch {
            data: LogBatchData::SliceOfOwnedData(data),
        }
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
        LogBatchDataIter {
            data: &self.data,
            index: 0,
        }
    }
}

struct LogBatchDataIter<'a> {
    data: &'a LogBatchData<'a>,
    index: usize,
}

impl<'a> Iterator for LogBatchDataIter<'a> {
    type Item = (&'a LogRecord, &'a InstrumentationScope);

    fn next(&mut self) -> Option<Self::Item> {
        match self.data {
            LogBatchData::SliceOfOwnedData(data) => {
                if self.index < data.len() {
                    let record = &*data[self.index];
                    self.index += 1;
                    Some((&record.0, &record.1))
                } else {
                    None
                }
            }
            LogBatchData::SliceOfBorrowedData(data) => {
                if self.index < data.len() {
                    let record = &data[self.index];
                    self.index += 1;
                    Some((record.0, record.1))
                } else {
                    None
                }
            }
        }
    }
}

/// `LogExporter` defines the interface that log exporters should implement.
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
    fn export(
        &self,
        batch: LogBatch<'_>,
    ) -> impl std::future::Future<Output = LogResult<()>> + Send;

    /// Shuts down the exporter.
    fn shutdown(&mut self) {}
    #[cfg(feature = "spec_unstable_logs_enabled")]
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
