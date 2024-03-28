use crate::export::logs::{LogData, LogExporter};
use crate::Resource;
use async_trait::async_trait;
use opentelemetry::logs::{LogError, LogRecord, LogResult};
use opentelemetry::InstrumentationLibrary;
use std::borrow::Cow;
use std::sync::{Arc, Mutex};

/// An in-memory logs exporter that stores logs data in memory..
///
/// This exporter is useful for testing and debugging purposes.
/// It stores logs in a `Vec<LogData>`. Logs can be retrieved using
/// `get_emitted_logs` method.
///
/// # Example
/// ```no_run
///# use opentelemetry_sdk::logs::{BatchLogProcessor, LoggerProvider};
///# use opentelemetry_sdk::runtime;
///# use opentelemetry_sdk::testing::logs::InMemoryLogsExporter;
///
///# #[tokio::main]
///# async fn main() {
///    // Create an InMemoryLogsExporter
///    let exporter: InMemoryLogsExporter = InMemoryLogsExporter::default();
///    //Create a LoggerProvider and register the exporter
///    let logger_provider = LoggerProvider::builder()
///        .with_log_processor(BatchLogProcessor::builder(exporter.clone(), runtime::Tokio).build())
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
pub struct InMemoryLogsExporter {
    logs: Arc<Mutex<Vec<LogData>>>,
    resource: Arc<Mutex<Resource>>,
}

impl Default for InMemoryLogsExporter {
    fn default() -> Self {
        InMemoryLogsExporterBuilder::new().build()
    }
}

/// `LogDataWithResource` associates a [`LogRecord`] with a [`Resource`] and
/// [`InstrumentationLibrary`].
#[derive(Clone, Debug)]
pub struct LogDataWithResource {
    /// Log record
    pub record: LogRecord,
    /// Instrumentation details for the emitter who produced this `LogData`.
    pub instrumentation: InstrumentationLibrary,
    /// Resource for the emitter who produced this `LogData`.
    pub resource: Cow<'static, Resource>,
}

///Builder for ['InMemoryLogsExporter'].
/// # Example
///
/// ```no_run
///# use opentelemetry_sdk::testing::logs::{InMemoryLogsExporter, InMemoryLogsExporterBuilder};
///# use opentelemetry_sdk::logs::{BatchLogProcessor, LoggerProvider};
///# use opentelemetry_sdk::runtime;
///
///# #[tokio::main]
///# async fn main() {
///    //Create an InMemoryLogsExporter
///    let exporter: InMemoryLogsExporter = InMemoryLogsExporterBuilder::default().build();
///    //Create a LoggerProvider and register the exporter
///    let logger_provider = LoggerProvider::builder()
///        .with_log_processor(BatchLogProcessor::builder(exporter.clone(), runtime::Tokio).build())
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
pub struct InMemoryLogsExporterBuilder {}

impl Default for InMemoryLogsExporterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryLogsExporterBuilder {
    /// Creates a new instance of `InMemoryLogsExporter`.
    ///
    pub fn new() -> Self {
        Self {}
    }

    /// Creates a new instance of `InMemoryLogsExporter`.
    ///
    pub fn build(&self) -> InMemoryLogsExporter {
        InMemoryLogsExporter {
            logs: Arc::new(Mutex::new(Vec::new())),
            resource: Arc::new(Mutex::new(Resource::default())),
        }
    }
}

impl InMemoryLogsExporter {
    /// Returns the logs emitted via Logger as a vector of `LogData`.
    ///
    /// # Example
    ///
    /// ```
    /// use opentelemetry_sdk::testing::logs::{InMemoryLogsExporter, InMemoryLogsExporterBuilder};
    ///
    /// let exporter = InMemoryLogsExporterBuilder::default().build();
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
    /// use opentelemetry_sdk::testing::logs::{InMemoryLogsExporter, InMemoryLogsExporterBuilder};
    ///
    /// let exporter = InMemoryLogsExporterBuilder::default().build();
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

#[async_trait]
impl LogExporter for InMemoryLogsExporter {
    async fn export(&mut self, batch: Vec<LogData>) -> LogResult<()> {
        self.logs
            .lock()
            .map(|mut logs_guard| logs_guard.append(&mut batch.clone()))
            .map_err(LogError::from)
    }
    fn shutdown(&mut self) {
        self.reset();
    }

    fn set_resource(&mut self, resource: &Resource) {
        let mut res_guard = self.resource.lock().expect("Resource lock poisoned");
        *res_guard = resource.clone();
    }
}
