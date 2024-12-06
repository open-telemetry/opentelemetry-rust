use log::{error, Level};
use opentelemetry::KeyValue;
use opentelemetry_appender_log::OpenTelemetryLogBridge;
use opentelemetry_sdk::logs::LoggerProvider;
use opentelemetry_sdk::thread_runtime::CustomThreadRuntime;
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions::resource::SERVICE_NAME;

fn main() {
    // Setup LoggerProvider with a stdout exporter
    let exporter = opentelemetry_stdout::LogExporter::default();
    let runtime = CustomThreadRuntime::new(2, 5); // 2 worker thread with queue size of 5
    let logger_provider = LoggerProvider::builder()
        .with_resource(Resource::new([KeyValue::new(
            SERVICE_NAME,
            "logs-basic-example",
        )]))
        .with_batch_exporter(exporter, runtime)
        .build();

    // Setup Log Appender for the log crate.
    let otel_log_appender = OpenTelemetryLogBridge::new(&logger_provider);
    log::set_boxed_logger(Box::new(otel_log_appender)).unwrap();
    log::set_max_level(Level::Error.to_level_filter());

    // Emit logs using macros from the log crate.
    // These logs gets piped through OpenTelemetry bridge and gets exported to stdout.
    // 10 error events
    for i in 0..10000 {
        error!(target: "my-target", "hello from {}. My price is {} at itr {}", "apple", 2.99, i);
        //sleep 1 sec every 100 secs
        if i % 10 == 0 {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
    println!("Flushing logs explicitly before exiting..");
    logger_provider.force_flush();
}
