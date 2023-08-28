<<<<<<< HEAD
<<<<<<< HEAD
//! run with `$ cargo run --example logs-basic-in-memory
=======
<<<<<<<< HEAD:opentelemetry-appender-log/examples/logs-basic-in-memory/src/main.rs
//! run with `$ cargo run --example logs-basic-in-memory
========
//! run with `$ cargo run --example logs-basic-in-memory 
>>>>>>>> 1ee6b54 (remove proj example in favor of single file):opentelemetry-appender-log/examples/logs-basic-in-memory.rs
>>>>>>> 1ee6b54 (remove proj example in favor of single file)
=======
//! run with `$ cargo run --example logs-basic-in-memory
>>>>>>> 12faf74 (corrected dependency, removed repeated code)

/// This example shows how to use the opentelemetry-appender-log crate, which is a
/// [logging appender](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/glossary.md#log-appender--bridge) that bridges logs from the [log crate](https://docs.rs/log/latest/log/) to OpenTelemetry.
/// The example setups a LoggerProvider with a in-memory exporter, so emitted logs are stored in memory.
///
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

    let emitted_logs = exporter.get_emitted_logs().unwrap();
    for log in emitted_logs {
        println!("{:?}", log);
    }
}
