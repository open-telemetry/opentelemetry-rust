#![cfg(unix)]

use anyhow::Result;
use ctor::dtor;
use integration_test_runner::test_utils;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::LogExporter;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::{logs as sdklogs, Resource};
use std::fs::File;
use std::io::Read;
use std::time::Duration;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use uuid::Uuid;

fn init_logs(is_simple: bool) -> Result<sdklogs::SdkLoggerProvider> {
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

    let mut logger_provider_builder = SdkLoggerProvider::builder();
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

async fn logs_tokio_helper(
    is_simple: bool,
    log_send_outside_rt: bool,
    current_thread: bool,
) -> Result<()> {
    use crate::{assert_logs_results_contains, init_logs};
    test_utils::start_collector_container().await?;

    let logger_provider = init_logs(is_simple).unwrap();
    let layer = OpenTelemetryTracingBridge::new(&logger_provider);
    // generate a random uuid and store it to expected guid
    let expected_uuid = std::sync::Arc::new(Uuid::new_v4().to_string());
    {
        let clone_uuid = expected_uuid.clone();
        if log_send_outside_rt {
            std::thread::spawn(move || {
                let subscriber = tracing_subscriber::registry().with(layer);
                let _guard = tracing::subscriber::set_default(subscriber);
                info!(
                    target: "my-target",
                    uuid = clone_uuid.as_str(),
                    "hello from {}. My price is {}.",
                    "banana",
                    2.99
                );
            })
            .join()
            .unwrap();
        } else {
            let subscriber = tracing_subscriber::registry().with(layer);
            let _guard = tracing::subscriber::set_default(subscriber);
            info!(target: "my-target",  uuid = expected_uuid.as_str(), "hello from {}. My price is {}.", "banana", 2.99);
        }
    }
    if current_thread {
        let _res = tokio::runtime::Handle::current()
            .spawn_blocking(move || logger_provider.shutdown())
            .await
            .unwrap();
    } else {
        assert!(logger_provider.shutdown().is_ok());
    }
    tokio::time::sleep(Duration::from_secs(5)).await;
    assert_logs_results_contains(test_utils::LOGS_FILE, expected_uuid.as_str())?;
    Ok(())
}

fn logs_non_tokio_helper(is_simple: bool, init_logs_inside_rt: bool) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    let logger_provider = if init_logs_inside_rt {
        // Initialize the logger provider inside the Tokio runtime
        rt.block_on(async {
            // Setup the collector container inside Tokio runtime
            test_utils::start_collector_container().await?;
            init_logs(is_simple)
        })?
    } else {
        // Initialize the logger provider outside the Tokio runtime
        rt.block_on(async {
            let _ = test_utils::start_collector_container().await;
        });
        init_logs(is_simple)?
    };

    let layer = OpenTelemetryTracingBridge::new(&logger_provider);
    let subscriber = tracing_subscriber::registry().with(layer);

    // Generate a random UUID and store it to expected guid
    let expected_uuid = Uuid::new_v4().to_string();
    {
        let _guard = tracing::subscriber::set_default(subscriber);
        info!(
            target: "my-target",
            uuid = expected_uuid,
            "hello from {}. My price is {}.",
            "banana",
            2.99
        );
    }

    assert!(logger_provider.shutdown().is_ok());
    std::thread::sleep(Duration::from_secs(5));
    assert_logs_results_contains(test_utils::LOGS_FILE, expected_uuid.as_str())?;
    Ok(())
}

fn assert_logs_results_contains(result: &str, expected_content: &str) -> Result<()> {
    let file = File::open(result)?;
    let mut contents = String::new();
    let mut reader = std::io::BufReader::new(&file);
    reader.read_to_string(&mut contents)?;
    assert!(contents.contains(expected_content));
    Ok(())
}

#[cfg(test)]
mod logtests {
    // The tests in this mod works like below: Emit a log with a UUID,
    // then read the logs from the file and check if the UUID is present in the
    // logs. This makes it easy to validate with a single collector and its
    // output. This is a very simple test but good enough to validate that OTLP
    // Exporter did work!

    use super::*;
    use integration_test_runner::logs_asserter::{read_logs_from_json, LogsAsserter};
    use std::fs::File;

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

    // Batch Processor

    // logger initialization - Inside RT
    // log emission - Inside RT
    // Client - Tonic, Reqwest-blocking
    // Worker threads - 4
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[cfg(any(feature = "tonic-client", feature = "reqwest-blocking-client"))]
    pub async fn logs_batch_tokio_multi_thread() -> Result<()> {
        logs_tokio_helper(false, false, false).await
    }

    // logger initialization - Inside RT
    // log emission - Inside RT
    // Client - Tonic, Reqwest-blocking
    // Worker threads - 1
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    #[cfg(any(feature = "tonic-client", feature = "reqwest-blocking-client"))]
    pub async fn logs_batch_tokio_multi_with_one_worker() -> Result<()> {
        logs_tokio_helper(false, false, false).await
    }

    // logger initialization - Inside RT
    // log emission - Inside RT
    // Client - Tonic, Reqwest-blocking
    // current thread
    #[tokio::test(flavor = "current_thread")]
    #[cfg(any(feature = "tonic-client", feature = "reqwest-blocking-client"))]
    pub async fn logs_batch_tokio_current() -> Result<()> {
        logs_tokio_helper(false, false, true).await
    }

    // logger initialization - Inside RT
    // Log emission - Outside RT
    // Client - Tonic, Reqwest-blocking
    // Worker threads - 4
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[cfg(any(feature = "tonic-client", feature = "reqwest-blocking-client"))]
    pub async fn logs_batch_tokio_log_outside_rt_multi_thread() -> Result<()> {
        logs_tokio_helper(false, true, false).await
    }

    // logger initialization - Inside RT
    // log emission - Outside RT
    // Client - Tonic, Reqwest-blocking
    // Worker threads - 1
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    #[cfg(any(feature = "tonic-client", feature = "reqwest-blocking-client"))]
    pub async fn logs_batch_tokio_log_outside_rt_multi_with_one_worker() -> Result<()> {
        logs_tokio_helper(false, true, false).await
    }

    // logger initialization - Inside RT
    // log emission - Outside RT
    // Client - Tonic, Reqwest-blocking
    // current thread
    #[tokio::test(flavor = "current_thread")]
    #[cfg(any(feature = "tonic-client", feature = "reqwest-blocking-client"))]
    pub async fn logs_batch_tokio_log_outside_rt_current_thread() -> Result<()> {
        logs_tokio_helper(false, true, true).await
    }

    // logger initialization  - Inside RT
    // Log emission - Inside RT
    // Client - Tonic, Reqwest-blocking
    // current thread
    #[test]
    #[cfg(any(feature = "tonic-client", feature = "reqwest-blocking-client"))]
    pub fn logs_batch_non_tokio_main_init_logs_inside_rt() -> Result<()> {
        logs_non_tokio_helper(false, true)
    }

    // logger initialization - Outside RT
    // log emission - Outside RT
    // Client - Tonic, Reqwest-blocking
    // current thread
    #[test]
    #[cfg(feature = "reqwest-blocking-client")]
    pub fn logs_batch_non_tokio_main_with_init_logs_outside_rt() -> Result<()> {
        logs_non_tokio_helper(false, false)
    }

    // logger initialization - Inside RT
    // log emission - Outside RT
    // Client - Tonic, Reqwest-blocking
    // current thread
    #[test]
    #[cfg(feature = "reqwest-blocking-client")]
    pub fn logs_batch_non_tokio_main_with_init_logs_inside_rt() -> Result<()> {
        logs_non_tokio_helper(false, true)
    }

    // **Simple Processor**

    // logger initialization - Inside RT
    // log emission - Outside RT
    // Client - Tonic, Reqwest-blocking
    #[test]
    #[cfg(any(feature = "tonic-client", feature = "reqwest-blocking-client"))]
    pub fn logs_simple_non_tokio_main_with_init_logs_inside_rt() -> Result<()> {
        logs_non_tokio_helper(true, true)
    }

    // logger initialization - Inside RT
    // log emission - Outside RT
    // Client - reqwest, hyper
    #[ignore] // request and hyper client does not work without tokio runtime
    #[test]
    #[cfg(any(feature = "reqwest-client", feature = "hyper-client"))]
    pub fn logs_simple_non_tokio_main_with_init_logs_inside_rt() -> Result<()> {
        logs_non_tokio_helper(true, true)
    }

    // logger initialization - Outside RT
    // log emission - Outside RT
    // Client - Reqwest-blocking
    #[test]
    #[cfg(feature = "reqwest-blocking-client")]
    pub fn logs_simple_non_tokio_main_with_init_logs_outsie_rt() -> Result<()> {
        logs_non_tokio_helper(true, false)
    }

    // logger initialization - Outside RT
    // log emission - Outside RT
    // Client - hyper, tonic, reqwest
    #[ignore] // request, tonic and hyper client does not work without tokio runtime
    #[test]
    #[cfg(any(
        feature = "hyper-client",
        feature = "tonic-client",
        feature = "reqwest-client"
    ))]
    pub fn logs_simple_non_tokio_main_with_init_logs_outsie_rt() -> Result<()> {
        logs_non_tokio_helper(true, false)
    }

    // logger initialization - Inside RT
    // log emission - Inside RT
    // Client - reqwest-blocking
    // Worker threads - 4
    #[ignore] // request-blocking client does not work with tokio
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[cfg(feature = "reqwest-blocking-client")]
    pub async fn logs_simple_tokio_multi_thread() -> Result<()> {
        logs_tokio_helper(true, false, false).await
    }

    // logger initialization - Inside RT
    // log emission - Inside RT
    // Client - Tonic, Reqwest, hyper
    // Worker threads - 4
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[cfg(any(
        feature = "tonic-client",
        feature = "reqwest-client",
        feature = "hyper-client"
    ))]
    pub async fn logs_simple_tokio_multi_thread() -> Result<()> {
        logs_tokio_helper(true, false, false).await
    }

    // logger initialization - Inside RT
    // log emission - Inside RT
    // Client - Tonic, Reqwest, hyper
    // Worker threads - 1
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    #[cfg(any(
        feature = "tonic-client",
        feature = "reqwest-client",
        feature = "hyper-client"
    ))]
    pub async fn logs_simple_tokio_multi_with_one_worker() -> Result<()> {
        logs_tokio_helper(true, false, false).await
    }

    // logger initialization - Inside RT
    // log emission - Inside RT
    // Client - Tonic, Reqwest, hyper
    // Current thread
    #[ignore] // https://github.com/open-telemetry/opentelemetry-rust/issues/2539
    #[tokio::test(flavor = "current_thread")]
    #[cfg(any(
        feature = "tonic-client",
        feature = "reqwest-client",
        feature = "hyper-client"
    ))]
    pub async fn logs_simple_tokio_current() -> Result<()> {
        logs_tokio_helper(true, false, false).await
    }
}
///
/// Make sure we stop the collector container, otherwise it will sit around hogging our
/// ports and subsequent test runs will fail.
///
#[dtor]
fn shutdown() {
    test_utils::stop_collector_container();
}
