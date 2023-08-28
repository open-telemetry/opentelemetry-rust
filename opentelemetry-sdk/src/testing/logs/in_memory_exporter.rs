use crate::export::logs::{LogData, LogExporter};
use async_trait::async_trait;
use opentelemetry_api::logs::{LogError, LogResult};
use std::sync::{Arc, Mutex};

/// An in-memory logs exporter that stores logs data in memory..
///
/// This exporter is useful for testing and debugging purposes.
/// It stores logs in a `Vec<LogData>`. Logs can be retrieved using
/// `get_emitted_logs` method.
///
/// # Example
/// ```
/// print(dupa)
/// ```
///
#[derive(Clone, Debug)]
pub struct InMemoryLogsExporter {
    logs: Arc<Mutex<Vec<LogData>>>,
}

impl Default for InMemoryLogsExporter {
    fn default() -> Self {
        InMemoryLogsExporterBuilder::new().build()
    }
}

///Builder for ['InMemoryLogsExporter'].
/// # Example
///
/// ```
/// use opentelemetry_sdk::testing::logs::{InMemoryLogsExporter, InMemoryLogsExporterBuilder};
///
/// use log::{error, info, Level, warn};
/// use opentelemetry_appender_log::OpenTelemetryLogBridge;
/// use opentelemetry_sdk::logs::{BatchLogProcessor, LoggerProvider};
/// use opentelemetry_sdk::runtime;
/// use opentelemetry_sdk::testing::logs::InMemoryLogsExporter;
///
///
/// [tokio::main]
/// sync fn main() {
///    //Create an InMemoryLogsExporter
///    let exporter: InMemoryLogsExporter = InMemoryLogsExporter::default();
///    //Create a LoggerProvider and register the exporter
///    let logger_provider = LoggerProvider::builder()
///        .with_log_processor(BatchLogProcessor::builder(exporter.clone(), runtime::Tokio).build())
///        .build();

///    // Setup Log Appender for the log crate.
///    let otel_log_appender = OpenTelemetryLogBridge::new(&logger_provider);
///    log::set_boxed_logger(Box::new(otel_log_appender)).unwrap();
///    log::set_max_level(Level::Info.to_level_filter());

///    // Emit logs using macros from the log crate.
///    error!("hello from {}. My price is {}", "apple", 2.99);
///    warn!("warn!");
///    info!("test log!");

///    logger_provider.force_flush();

///    let emitted_logs = exporter.get_emitted_logs().unwrap();
///    for log in emitted_logs {
///        println!("{:?}", log);
///    }
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
        }
    }
}

impl InMemoryLogsExporter {
    /// Returns the logs emitted via Logger as a vector of `LogData`.
    ///
    /// # Errors
    ///
    /// Returns a `LogError` if the internal lock cannot be acquired.
    ///
    /// # Example
    ///
    /// ```
    /// use opentelemetry_sdk::testing::logs::{InMemoryLogsExporter};
    ///
    /// let exporter = InMemoryLogsExporterBuilder::default();
    /// let emitted_logs = exporter.get_emitted_logs().unwrap();
    /// ```
    ///
    pub fn get_emitted_logs(&self) -> LogResult<Vec<LogData>> {
        self.logs
            .lock()
            .map(|logs_guard| logs_guard.iter().cloned().collect())
            .map_err(LogError::from)
    }

    /// Clears the internal (in-memory) storage of logs.
    ///
    /// # Errors
    ///
    /// Returns a `LogError` if the internal lock cannot be acquired.
    ///
    /// # Example
    ///
    /// ```
    /// use opentelemetry_sdk::testing::logs::{InMemoryLogsExporter};
    ///
    /// let exporter = InMemoryLogsExporterBuilder::default();
    /// exporter.reset();
    /// ```
    ///
    pub fn reset(&mut self) {
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
}
