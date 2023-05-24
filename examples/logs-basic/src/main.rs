use log::{error, Level};
use opentelemetry_api::KeyValue;
use opentelemetry_appender_log::OpenTelemetryLogBridge;
use opentelemetry_sdk::logs::{Config, LoggerProvider};
use opentelemetry_sdk::Resource;

fn main() {
    // Setup LoggerProvider with a stdout exporter
    let exporter = opentelemetry_stdout::LogExporter::default();
    let logger_provider = LoggerProvider::builder()
        .with_config(
            Config::default().with_resource(Resource::new(vec![KeyValue::new(
                "service.name",
                "logs-basic-example",
            )])),
        )
        .with_simple_exporter(exporter)
        .build();

    // Setup Log Appender for the log crate.
    let otel_log_appender = OpenTelemetryLogBridge::new(Level::Info, &logger_provider);
    log::set_boxed_logger(Box::new(otel_log_appender)).unwrap();

    // Emit logs using macros from the log crate.
    // These logs gets piped through OpenTelemetry bridge and gets exported to stdout.
    error!(target: "my-target", "hello from {}. My price is {}", "apple", 2.99);
}
