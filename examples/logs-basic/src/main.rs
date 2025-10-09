use log::{error, info, warn, Level};
use opentelemetry_appender_log::OpenTelemetryLogBridge;
use opentelemetry_sdk::logs::{BatchLogProcessor, SdkLoggerProvider};
use opentelemetry_stdout::LogExporter;

#[tokio::main]
async fn main() {
    //Create an exporter that writes to stdout
    let exporter = LogExporter::default();
    //Create a LoggerProvider and register the exporter
    let logger_provider = SdkLoggerProvider::builder()
        .with_log_processor(BatchLogProcessor::builder(exporter).build())
        .build();

    // Setup Log Appender for the log crate.
    let otel_log_appender = OpenTelemetryLogBridge::new(&logger_provider);
    log::set_boxed_logger(Box::new(otel_log_appender)).unwrap();
    log::set_max_level(Level::Info.to_level_filter());

    // Emit logs using macros from the log crate.
    let fruit = "apple";
    let price = 2.99;

    error!(fruit, price; "hello from {fruit}. My price is {price}");
    warn!("warn!");
    info!("test log!");

    let _ = logger_provider.shutdown();
}
