#![cfg(unix)]

use anyhow::Result;
use ctor::dtor;
use integration_test_runner::logs_asserter::{read_logs_from_json, LogsAsserter};
use integration_test_runner::test_utils;
use opentelemetry_appender_tracing::layer;
use opentelemetry_otlp::LogExporter;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::Resource;
use std::fs::File;
use std::os::unix::fs::MetadataExt;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[cfg(feature = "tonic-client")]
pub async fn test_logs() -> Result<()> {
    test_utils::start_collector_container().await?;
    test_utils::cleanup_file("./actual/logs.json"); // Ensure logs.json is empty before the test
    let exporter_builder = LogExporter::builder().with_tonic();
    let exporter = exporter_builder.build()?;
    let mut logger_provider_builder = SdkLoggerProvider::builder();
    logger_provider_builder = logger_provider_builder.with_batch_exporter(exporter);
    let logger_provider = logger_provider_builder
        .with_resource(
            Resource::builder_empty()
                .with_service_name("logs-integration-test")
                .build(),
        )
        .build();
    let layer = layer::OpenTelemetryTracingBridge::new(&logger_provider);
    let subscriber = tracing_subscriber::registry().with(layer);

    {
        let _guard = tracing::subscriber::set_default(subscriber);
        info!(target: "my-target", "hello from {}. My price is {}.", "banana", 2.99);
    }

    let _ = logger_provider.shutdown();
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    assert_logs_results(test_utils::LOGS_FILE, "expected/logs.json")?;
    Ok(())
}

fn assert_logs_results(result: &str, expected: &str) -> Result<()> {
    let left = read_logs_from_json(File::open(expected)?)?;
    let right = read_logs_from_json(File::open(result)?)?;

    LogsAsserter::new(left, right).assert();

    assert!(File::open(result).unwrap().metadata().unwrap().size() > 0);
    Ok(())
}

///
/// Make sure we stop the collector container, otherwise it will sit around hogging our
/// ports and subsequent test runs will fail.
///
#[dtor]
fn shutdown() {
    println!("metrics::shutdown");
    test_utils::stop_collector_container();
}
