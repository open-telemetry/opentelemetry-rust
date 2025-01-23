use crate::logs::LogRecord;
use crate::logs::{LogBatch, LogError, LogExporter, LogResult};
use crate::Resource;
use opentelemetry::InstrumentationScope;
use std::borrow::Cow;
use std::sync::{Arc, Mutex};

/// An in-memory logs exporter that stores logs data in memory..
///
/// This exporter is useful for testing and debugging purposes.
/// It stores logs in a `Vec<OwnedLogData>`. Logs can be retrieved using
/// `get_emitted_logs` method.
///
/// # Example
/// ```no_run
///# use opentelemetry_sdk::logs::{BatchLogProcessor, LoggerProvider};
///# use opentelemetry_sdk::runtime;
///# use opentelemetry_sdk::logs::InMemoryLogExporter;
///
///# #[tokio::main]
///# async fn main() {
///    // Create an InMemoryLogExporter
///    let exporter: InMemoryLogExporter = InMemoryLogExporter::default();
///    //Create a LoggerProvider and register the exporter
///    let logger_provider = LoggerProvider::builder()
///        .with_log_processor(BatchLogProcessor::builder(exporter.clone()).build())
///        .build();
///    // Setup Log Appenders and emit logs. (Not shown here)
///    logger_provider.force_flush();
///    let emitted_logs = exporter.get_emitted_logs().unwrap();
///    for log in emitted_logs {
///        println!("{:?}", log);
///    }
///# }
/// ```
///
#[derive(Clone, Debug)]
pub struct InMemoryLogExporter {
    logs: Arc<Mutex<Vec<OwnedLogData>>>,
    resource: Arc<Mutex<Resource>>,
    should_reset_on_shutdown: bool,
}

impl Default for InMemoryLogExporter {
    fn default() -> Self {
        InMemoryLogExporterBuilder::new().build()
    }
}

/// `OwnedLogData` represents a single log event without resource context.
#[derive(Debug, Clone)]
pub struct OwnedLogData {
    /// Log record, which can be borrowed or owned.
    pub record: LogRecord,
    /// Instrumentation details for the emitter who produced this `LogEvent`.
    pub instrumentation: InstrumentationScope,
}

/// `LogDataWithResource` associates a [`LogRecord`] with a [`Resource`] and
/// [`InstrumentationScope`].
#[derive(Clone, Debug)]
pub struct LogDataWithResource {
    /// Log record
    pub record: LogRecord,
    /// Instrumentation details for the emitter who produced this `LogRecord`.
    pub instrumentation: InstrumentationScope,
    /// Resource for the emitter who produced this `LogRecord`.
    pub resource: Cow<'static, Resource>,
}

///Builder for ['InMemoryLogExporter'].
/// # Example
///
/// ```no_run
///# use opentelemetry_sdk::logs::{InMemoryLogExporter, InMemoryLogExporterBuilder};
///# use opentelemetry_sdk::logs::{BatchLogProcessor, LoggerProvider};
///# use opentelemetry_sdk::runtime;
///
///# #[tokio::main]
///# async fn main() {
///    //Create an InMemoryLogExporter
///    let exporter: InMemoryLogExporter = InMemoryLogExporterBuilder::default().build();
///    //Create a LoggerProvider and register the exporter
///    let logger_provider = LoggerProvider::builder()
///        .with_log_processor(BatchLogProcessor::builder(exporter.clone()).build())
///        .build();
///    // Setup Log Appenders and emit logs. (Not shown here)
///    logger_provider.force_flush();
///    let emitted_logs = exporter.get_emitted_logs().unwrap();
///    for log in emitted_logs {
///        println!("{:?}", log);
///    }
///# }
///
/// ```
///
#[derive(Debug, Clone)]
pub struct InMemoryLogExporterBuilder {
    reset_on_shutdown: bool,
}

impl Default for InMemoryLogExporterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryLogExporterBuilder {
    /// Creates a new instance of `InMemoryLogExporter`.
    ///
    pub fn new() -> Self {
        Self {
            reset_on_shutdown: true,
        }
    }

    /// Creates a new instance of `InMemoryLogExporter`.
    ///
    pub fn build(&self) -> InMemoryLogExporter {
        InMemoryLogExporter {
            logs: Arc::new(Mutex::new(Vec::new())),
            resource: Arc::new(Mutex::new(Resource::builder().build())),
            should_reset_on_shutdown: self.reset_on_shutdown,
        }
    }

    /// If set, the records will not be [`InMemoryLogExporter::reset`] on shutdown.
    #[cfg(test)]
    pub(crate) fn keep_records_on_shutdown(self) -> Self {
        Self {
            reset_on_shutdown: false,
        }
    }
}

impl InMemoryLogExporter {
    /// Returns the logs emitted via Logger as a vector of `LogDataWithResource`.
    ///
    /// # Example
    ///
    /// ```
    /// use opentelemetry_sdk::logs::{InMemoryLogExporter, InMemoryLogExporterBuilder};
    ///
    /// let exporter = InMemoryLogExporterBuilder::default().build();
    /// let emitted_logs = exporter.get_emitted_logs().unwrap();
    /// ```
    ///
    pub fn get_emitted_logs(&self) -> LogResult<Vec<LogDataWithResource>> {
        let logs_guard = self.logs.lock().map_err(LogError::from)?;
        let resource_guard = self.resource.lock().map_err(LogError::from)?;
        let logs: Vec<LogDataWithResource> = logs_guard
            .iter()
            .map(|log_data| LogDataWithResource {
                record: log_data.record.clone(),
                resource: Cow::Owned(resource_guard.clone()),
                instrumentation: log_data.instrumentation.clone(),
            })
            .collect();

        Ok(logs)
    }
    /// Clears the internal (in-memory) storage of logs.
    ///
    /// # Example
    ///
    /// ```
    /// use opentelemetry_sdk::logs::{InMemoryLogExporter, InMemoryLogExporterBuilder};
    ///
    /// let exporter = InMemoryLogExporterBuilder::default().build();
    /// exporter.reset();
    /// ```
    ///
    pub fn reset(&self) {
        let _ = self
            .logs
            .lock()
            .map(|mut logs_guard| logs_guard.clear())
            .map_err(LogError::from);
    }
}

impl LogExporter for InMemoryLogExporter {
    #[allow(clippy::manual_async_fn)]
    fn export(
        &self,
        batch: LogBatch<'_>,
    ) -> impl std::future::Future<Output = LogResult<()>> + Send {
        async move {
            let mut logs_guard = self.logs.lock().map_err(LogError::from)?;
            for (log_record, instrumentation) in batch.iter() {
                let owned_log = OwnedLogData {
                    record: (*log_record).clone(),
                    instrumentation: (*instrumentation).clone(),
                };
                logs_guard.push(owned_log);
            }
            Ok(())
        }
    }

    fn shutdown(&mut self) {
        if self.should_reset_on_shutdown {
            self.reset();
        }
    }

    fn set_resource(&mut self, resource: &Resource) {
        let mut res_guard = self.resource.lock().expect("Resource lock poisoned");
        *res_guard = resource.clone();
    }
}
