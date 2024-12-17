#![cfg(unix)]

use anyhow::Result;
use ctor::dtor;
use integration_test_runner::logs_asserter::{read_logs_from_json, LogsAsserter};
use integration_test_runner::test_utils;
use log::{info, Level};
use opentelemetry_appender_log::OpenTelemetryLogBridge;
use opentelemetry_otlp::LogExporter;
use opentelemetry_sdk::logs::LoggerProvider;
use opentelemetry_sdk::{logs as sdklogs, runtime, Resource};
use std::fs::File;
use std::os::unix::fs::MetadataExt;
use std::time::Duration;

fn init_logs() -> Result<sdklogs::LoggerProvider> {
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
        .with_resource(
            Resource::builder_empty()
                .with_service_name("logs-integration-test")
                .build(),
        )
        .build())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
pub async fn test_logs() -> Result<()> {
    // Make sure the container is running
    test_utils::start_collector_container().await?;

    let logger_provider = init_logs().unwrap();
    let otel_log_appender = OpenTelemetryLogBridge::new(&logger_provider);
    log::set_boxed_logger(Box::new(otel_log_appender))?;
    log::set_max_level(Level::Info.to_level_filter());

    info!(target: "my-target", "hello from {}. My price is {}.", "banana", 2.99);
    let _ = logger_provider.shutdown();

    tokio::time::sleep(Duration::from_secs(10)).await;

    assert_logs_results(test_utils::LOGS_FILE, "expected/logs.json");

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

///
/// Make sure we stop the collector container, otherwise it will sit around hogging our
/// ports and subsequent test runs will fail.
///
#[dtor]
fn shutdown() {
    test_utils::stop_collector_container();
}
