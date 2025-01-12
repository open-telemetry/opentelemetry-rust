#![cfg(unix)]

use anyhow::Result;
use ctor::dtor;
use integration_test_runner::logs_asserter::{read_logs_from_json, LogsAsserter};
use integration_test_runner::test_utils;
use opentelemetry_otlp::LogExporter;
use opentelemetry_sdk::logs::LoggerProvider;
use opentelemetry_sdk::{logs as sdklogs, Resource};
use std::fs::File;
use std::os::unix::fs::MetadataExt;

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
        .with_batch_exporter(exporter)
        .with_resource(
            Resource::builder_empty()
                .with_service_name("logs-integration-test")
                .build(),
        )
        .build())
}

#[cfg(test)]
mod logtests {
    use super::*;
    use integration_test_runner::logs_asserter::{read_logs_from_json, LogsAsserter};
    use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
    use std::{fs::File, time::Duration};
    use tracing::info;
    use tracing_subscriber::layer::SubscriberExt;

    #[test]
    #[should_panic(expected = "assertion `left == right` failed: body does not match")]
    pub fn test_assert_logs_eq_failure() {
        let left = read_logs_from_json(
            File::open("./expected/logs.json").expect("failed to open expected file"),
        )
        .expect("Failed to read logs from expected file");

        let right = read_logs_from_json(
            File::open("./expected/failed_logs.json")
                .expect("failed to open expected failed log file"),
        )
        .expect("Failed to read logs from expected failed log file");
        LogsAsserter::new(right, left).assert();
    }

    #[test]
    pub fn test_assert_logs_eq() -> Result<()> {
        let logs = read_logs_from_json(File::open("./expected/logs.json")?)?;
        LogsAsserter::new(logs.clone(), logs).assert();

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[cfg(not(feature = "hyper-client"))]
    #[cfg(not(feature = "reqwest-client"))]
    pub async fn test_logs() -> Result<()> {
        // Make sure the container is running

        use integration_test_runner::test_utils;
        use opentelemetry_appender_tracing::layer;
        use tracing::info;
        use tracing_subscriber::layer::SubscriberExt;

        use crate::{assert_logs_results, init_logs};
        test_utils::start_collector_container().await?;

        let logger_provider = init_logs().unwrap();
        let layer = layer::OpenTelemetryTracingBridge::new(&logger_provider);
        let subscriber = tracing_subscriber::registry().with(layer);
        {
            let _guard = tracing::subscriber::set_default(subscriber);
            info!(target: "my-target", "hello from {}. My price is {}.", "banana", 2.99);
        }
        // TODO: remove below wait before calling logger_provider.shutdown()
        // tokio::time::sleep(Duration::from_secs(10)).await;
        let _ = logger_provider.shutdown();

        tokio::time::sleep(Duration::from_secs(10)).await;

        assert_logs_results(test_utils::LOGS_FILE, "expected/logs.json")?;

        Ok(())
    }

    #[ignore = "TODO: [Fix Me] Failing on CI. Needs to be investigated and resolved."]
    #[test]
    #[cfg(any(feature = "tonic-client", feature = "reqwest-blocking-client"))]
    pub fn logs_batch_non_tokio_main() -> Result<()> {
        // Initialize the logger provider inside a tokio runtime
        // as this allows tonic client to capture the runtime,
        // but actual export occurs from the dedicated std::thread
        // created by BatchLogProcessor.
        let rt = tokio::runtime::Runtime::new()?;
        let logger_provider = rt.block_on(async {
            // While we're here setup our collector container too, as this needs tokio to run
            test_utils::start_collector_container().await?;
            init_logs()
        })?;

        info!("LoggerProvider created");
        let layer = OpenTelemetryTracingBridge::new(&logger_provider);
        let subscriber = tracing_subscriber::registry().with(layer);
        {
            let _guard = tracing::subscriber::set_default(subscriber);
            info!(target: "my-target", "hello from {}. My price is {}.", "banana", 2.99);
        }
        let _ = logger_provider.shutdown();
        // tokio::time::sleep(Duration::from_secs(10)).await;
        assert_logs_results(test_utils::LOGS_FILE, "expected/logs.json")?;

        Ok(())
    }
}

pub fn assert_logs_results(result: &str, expected: &str) -> Result<()> {
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
    test_utils::stop_collector_container();
}
