//! run with `$ cargo run --example logs-basic`

/// This example shows how to use in_memory_exporter for logs. This uses opentelemetry-appender-log crate, which is a
/// [logging appender](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/glossary.md#log-appender--bridge) that bridges logs from the [log crate](https://docs.rs/log/latest/log/) to OpenTelemetry.
/// The example setups a LoggerProvider with a in-memory exporter, so emitted logs are stored in memory.
///
use log::{error, info, warn, Level};
use opentelemetry_appender_log::OpenTelemetryLogBridge;
use opentelemetry_sdk::logs::{BatchLogProcessor, LoggerProvider};
use opentelemetry_sdk::runtime;
use opentelemetry_stdout::LogExporter;

#[tokio::main]
async fn main() {
    //Create an exporter that writes to stdout
    let exporter = LogExporter::default();
    //Create a LoggerProvider and register the exporter
    let logger_provider = LoggerProvider::builder()
        .with_log_processor(BatchLogProcessor::builder(exporter, runtime::Tokio).build())
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

    logger_provider.force_flush();
}
