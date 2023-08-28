use log::{error, info, warn, Level};
use opentelemetry_appender_log::OpenTelemetryLogBridge;
use opentelemetry_sdk::logs::{BatchLogProcessor, LoggerProvider};
use opentelemetry_sdk::runtime;
use opentelemetry_sdk::testing::logs::InMemoryLogsExporter;

#[tokio::main]
async fn main() {
    //Create an InMemoryLogsExporter
    let exporter: InMemoryLogsExporter = InMemoryLogsExporter::default();
    //Create a LoggerProvider and register the exporter
    let logger_provider = LoggerProvider::builder()
        .with_log_processor(BatchLogProcessor::builder(exporter.clone(), runtime::Tokio).build())
        .build();

    // Setup Log Appender for the log crate.
    let otel_log_appender = OpenTelemetryLogBridge::new(&logger_provider);
    log::set_boxed_logger(Box::new(otel_log_appender)).unwrap();
    log::set_max_level(Level::Info.to_level_filter());

    // Emit logs using macros from the log crate.
    error!("hello from {}. My price is {}", "apple", 2.99);
    warn!("warn!");
    info!("test log!");

    logger_provider.force_flush();

    let finished_logs = exporter.get_finished_logs().unwrap();
    for log in finished_logs {
        println!("{:?}", log);
    }
}
