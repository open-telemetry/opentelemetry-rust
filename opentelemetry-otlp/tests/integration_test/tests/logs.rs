#![cfg(unix)]

use integration_test_runner::logs_asserter::{read_logs_from_json, LogsAsserter};
use log::{info, Level};
use opentelemetry::KeyValue;
use opentelemetry_appender_log::OpenTelemetryLogBridge;
use opentelemetry_otlp::LogExporter;
use opentelemetry_sdk::logs::{LogError, LoggerProvider};
use opentelemetry_sdk::{logs as sdklogs, Resource};
use std::error::Error;
use std::fs::File;
use std::os::unix::fs::MetadataExt;

fn init_logs() -> Result<sdklogs::LoggerProvider, LogError> {
    let exporter_builder = LogExporter::builder();
    #[cfg(feature = "tonic-client")]
    let exporter_builder = exporter_builder.with_tonic();
    #[cfg(not(feature = "tonic-client"))]
    #[cfg(any(
        feature = "hyper-client",
        feature = "reqwest-client",
        feature = "reqwest-blocking-client"
    ))]
    let exporter_builder = exporter_builder.with_http();

    let exporter = exporter_builder.build()?;

    Ok(LoggerProvider::builder()
        .with_batch_exporter(exporter, runtime::Tokio)
        .with_resource(Resource::new(vec![KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
            "logs-integration-test",
        )]))
        .build())
}

pub async fn logs() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let logger_provider = init_logs().unwrap();
    let otel_log_appender = OpenTelemetryLogBridge::new(&logger_provider);
    log::set_boxed_logger(Box::new(otel_log_appender))?;
    log::set_max_level(Level::Info.to_level_filter());

    info!(target: "my-target", "hello from {}. My price is {}.", "banana", 2.99);
    let _ = logger_provider.shutdown();
    Ok(())
}

pub fn assert_logs_results(result: &str, expected: &str) {
    let left = read_logs_from_json(File::open(expected).unwrap());
    let right = read_logs_from_json(File::open(result).unwrap());

    LogsAsserter::new(left, right).assert();

    assert!(File::open(result).unwrap().metadata().unwrap().size() > 0)
}

#[test]
#[should_panic(expected = "assertion `left == right` failed: body does not match")]
pub fn test_assert_logs_eq_failure() {
    let left = read_logs_from_json(File::open("./expected/logs.json").unwrap());
    let right = read_logs_from_json(File::open("./expected/failed_logs.json").unwrap());
    LogsAsserter::new(right, left).assert();
}

#[test]
pub fn test_assert_logs_eq() {
    let logs = read_logs_from_json(File::open("./expected/logs.json").unwrap());
    LogsAsserter::new(logs.clone(), logs).assert();
}
