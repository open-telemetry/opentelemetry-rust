#![cfg(unix)]

use anyhow::Result;
use ctor::dtor;
use integration_test_runner::test_utils;
use opentelemetry_otlp::LogExporter;
use opentelemetry_sdk::logs::LoggerProvider;
use opentelemetry_sdk::{logs as sdklogs, Resource};
use std::fs::File;
use std::io::Read;

fn init_logs(is_simple: bool) -> Result<sdklogs::LoggerProvider> {
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

    let mut logger_provider_builder = LoggerProvider::builder();
    if is_simple {
        logger_provider_builder = logger_provider_builder.with_simple_exporter(exporter)
    } else {
        logger_provider_builder = logger_provider_builder.with_batch_exporter(exporter)
    };

    let logger_provider = logger_provider_builder
        .with_resource(
            Resource::builder_empty()
                .with_service_name("logs-integration-test")
                .build(),
        )
        .build();

    Ok(logger_provider)
}

#[cfg(test)]
mod logtests {
    // TODO: The tests in this mod works like below: Emit a log with a UUID,
    // then read the logs from the file and check if the UUID is present in the
    // logs. This makes it easy to validate with a single collector and its
    // output. This is a very simple test but good enough to validate that OTLP
    // Exporter did work! A more comprehensive test would be to validate the
    // entire Payload. The infra for it already exists (logs_asserter.rs), the
    // TODO here is to write a test that validates the entire payload.

    use super::*;
    use integration_test_runner::logs_asserter::{read_logs_from_json, LogsAsserter};
    use integration_test_runner::test_utils;
    use opentelemetry_appender_tracing::layer;
    use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
    use std::{fs::File, time::Duration};
    use tracing::info;
    use tracing_subscriber::layer::SubscriberExt;
    use uuid::Uuid;

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
    #[cfg(any(feature = "tonic-client", feature = "reqwest-blocking-client"))]
    pub async fn logs_batch_tokio_multi_thread() -> Result<()> {
        logs_batch_tokio_helper().await
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    #[cfg(any(feature = "tonic-client", feature = "reqwest-blocking-client"))]
    pub async fn logs_batch_tokio_multi_with_one_worker() -> Result<()> {
        logs_batch_tokio_helper().await
    }

    #[tokio::test(flavor = "current_thread")]
    #[cfg(any(feature = "tonic-client", feature = "reqwest-blocking-client"))]
    pub async fn logs_batch_tokio_current() -> Result<()> {
        logs_batch_tokio_helper().await
    }

    async fn logs_batch_tokio_helper() -> Result<()> {
        use crate::{assert_logs_results, init_logs};
        test_utils::start_collector_container().await?;

        let logger_provider = init_logs(false).unwrap();
        let layer = OpenTelemetryTracingBridge::new(&logger_provider);
        let subscriber = tracing_subscriber::registry().with(layer);
        // generate a random uuid and store it to expected guid
        let expected_uuid = Uuid::new_v4().to_string();
        {
            let _guard = tracing::subscriber::set_default(subscriber);
            info!(target: "my-target",  uuid = expected_uuid, "hello from {}. My price is {}.", "banana", 2.99);
        }

        let _ = logger_provider.shutdown();
        tokio::time::sleep(Duration::from_secs(5)).await;
        assert_logs_results(test_utils::LOGS_FILE, expected_uuid.as_str())?;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[cfg(any(feature = "tonic-client", feature = "reqwest-client"))]
    pub async fn logs_simple_tokio_multi_thread() -> Result<()> {
        logs_simple_tokio_helper().await
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    #[cfg(any(feature = "tonic-client", feature = "reqwest-client"))]
    pub async fn logs_simple_tokio_multi_with_one_worker() -> Result<()> {
        logs_simple_tokio_helper().await
    }

    // Ignored, to be investigated
    #[ignore]
    #[tokio::test(flavor = "current_thread")]
    #[cfg(any(feature = "tonic-client", feature = "reqwest-client"))]
    pub async fn logs_simple_tokio_current() -> Result<()> {
        logs_simple_tokio_helper().await
    }

    async fn logs_simple_tokio_helper() -> Result<()> {
        use crate::{assert_logs_results, init_logs};
        test_utils::start_collector_container().await?;

        let logger_provider = init_logs(true).unwrap();
        let layer = OpenTelemetryTracingBridge::new(&logger_provider);
        let subscriber = tracing_subscriber::registry().with(layer);
        info!("Tracing initialized");
        // generate a random uuid and store it to expected guid
        let expected_uuid = Uuid::new_v4().to_string();
        {
            let _guard = tracing::subscriber::set_default(subscriber);
            info!(target: "my-target",  uuid = expected_uuid, "hello from {}. My price is {}.", "banana", 2.99);
        }

        let _ = logger_provider.shutdown();
        tokio::time::sleep(Duration::from_secs(5)).await;
        assert_logs_results(test_utils::LOGS_FILE, expected_uuid.as_str())?;
        Ok(())
    }

    #[test]
    #[cfg(any(feature = "tonic-client", feature = "reqwest-blocking-client"))]
    pub fn logs_batch_non_tokio_main() -> Result<()> {
        logs_batch_non_tokio_helper()
    }

    fn logs_batch_non_tokio_helper() -> Result<()> {
        // Initialize the logger provider inside a tokio runtime
        // as this allows tonic client to capture the runtime,
        // but actual export occurs from the dedicated std::thread
        // created by BatchLogProcessor.
        let rt = tokio::runtime::Runtime::new()?;
        let logger_provider = rt.block_on(async {
            // While we're here setup our collector container too, as this needs tokio to run
            test_utils::start_collector_container().await?;
            init_logs(false)
        })?;
        let layer = layer::OpenTelemetryTracingBridge::new(&logger_provider);
        let subscriber = tracing_subscriber::registry().with(layer);
        // generate a random uuid and store it to expected guid
        let expected_uuid = Uuid::new_v4().to_string();
        {
            let _guard = tracing::subscriber::set_default(subscriber);
            info!(target: "my-target",  uuid = expected_uuid, "hello from {}. My price is {}.", "banana", 2.99);
        }

        let _ = logger_provider.shutdown();
        std::thread::sleep(Duration::from_secs(5));
        assert_logs_results(test_utils::LOGS_FILE, expected_uuid.as_str())?;
        Ok(())
    }

    #[test]
    #[cfg(any(feature = "tonic-client", feature = "reqwest-blocking-client"))]
    pub fn logs_simple_non_tokio_main() -> Result<()> {
        logs_simple_non_tokio_helper()
    }

    fn logs_simple_non_tokio_helper() -> Result<()> {
        // Initialize the logger provider inside a tokio runtime
        // as this allows tonic client to capture the runtime,
        // but actual export occurs from the main non-tokio thread.
        let rt = tokio::runtime::Runtime::new()?;
        let logger_provider = rt.block_on(async {
            // While we're here setup our collector container too, as this needs tokio to run
            test_utils::start_collector_container().await?;
            init_logs(true)
        })?;
        let layer = layer::OpenTelemetryTracingBridge::new(&logger_provider);
        let subscriber = tracing_subscriber::registry().with(layer);
        // generate a random uuid and store it to expected guid
        let expected_uuid = Uuid::new_v4().to_string();
        {
            let _guard = tracing::subscriber::set_default(subscriber);
            info!(target: "my-target",  uuid = expected_uuid, "hello from {}. My price is {}.", "banana", 2.99);
        }

        let _ = logger_provider.shutdown();
        std::thread::sleep(Duration::from_secs(5));
        assert_logs_results(test_utils::LOGS_FILE, expected_uuid.as_str())?;
        Ok(())
    }
}

pub fn assert_logs_results(result: &str, expected_content: &str) -> Result<()> {
    let file = File::open(result)?;
    let mut contents = String::new();
    let mut reader = std::io::BufReader::new(&file);
    reader.read_to_string(&mut contents)?;
    assert!(contents.contains(expected_content));
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
