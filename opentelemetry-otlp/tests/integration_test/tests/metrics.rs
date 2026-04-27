//! OTLP integration tests for metrics. These tests cover various OTel Metric
//! SDK/OTLP Exporter scenarios in particular focusing on ensuring that various
//! async runtimes works well. This also includes validating that shutdown,
//! force_flush are working as expected. Validation is simple in the sense it
//! merely checks the presence of a UUID in the exported metrics, which is good
//! enough to confirm that metrics have been accepted by OTel Collector.
//!
#![cfg(unix)]

use anyhow::{Ok, Result};
use ctor::dtor;
use integration_test_runner::test_utils;
use opentelemetry::KeyValue;
use opentelemetry::metrics::MeterProvider as _;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use std::net::TcpStream;
use std::time::Duration;

const SLEEP_DURATION: Duration = Duration::from_secs(5);
const FLUSH_RETRY_SLEEP: Duration = Duration::from_millis(250);
const FLUSH_MAX_RETRIES: usize = 60;

#[cfg(test)]
#[cfg(any(feature = "tonic-client", feature = "reqwest-blocking-client"))]
mod metrictests {
    use super::*;
    use integration_test_runner::metric_helpers::{
        assert_metrics_results_contains, setup_metrics_non_tokio, setup_metrics_tokio,
    };
    use tokio::runtime::Handle;
    use uuid::Uuid;

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn counter_tokio_multi_thread() -> Result<()> {
        metric_helper_tokio().await
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn counter_tokio_multi_thread_one_worker() -> Result<()> {
        metric_helper_tokio().await
    }

    #[tokio::test(flavor = "current_thread")]
    async fn counter_tokio_current() -> Result<()> {
        metric_helper_tokio_current().await
    }

    #[test]
    fn counter_non_tokio() -> Result<()> {
        metric_helper_non_tokio()
    }

    async fn metric_helper_tokio() -> Result<()> {
        let meter_provider = setup_metrics_tokio().await;
        wait_for_collector_grpc();
        emit_and_validate_metrics(meter_provider)
    }

    async fn metric_helper_tokio_current() -> Result<()> {
        let meter_provider = setup_metrics_tokio().await;
        wait_for_collector_grpc();
        let expected_uuid = emit_metrics_with_provider(&meter_provider);
        force_flush_with_retry(&meter_provider);

        // In tokio::current_thread flavor, shutdown must be done in a separate thread
        let shutdown_result = Handle::current()
            .spawn_blocking(move || meter_provider.shutdown())
            .await
            .unwrap();
        assert!(shutdown_result.is_ok());
        // We still need to sleep, to give otel-collector a chance to flush to disk
        std::thread::sleep(SLEEP_DURATION);

        // Validate metrics against results file This is not the extensive
        // validation of output, but good enough to confirm that metrics have
        // been accepted by OTel Collector.
        assert_metrics_results_contains(&expected_uuid)?;

        Ok(())
    }

    fn metric_helper_non_tokio() -> Result<()> {
        let (meter_provider, _rt) = setup_metrics_non_tokio(true);
        wait_for_collector_grpc();
        let expected_uuid = emit_metrics_with_provider(&meter_provider);
        force_flush_with_retry(&meter_provider);

        let shutdown_result = meter_provider.shutdown();
        assert!(shutdown_result.is_ok());
        // We still need to sleep, to give otel-collector a chance to flush to disk
        std::thread::sleep(SLEEP_DURATION);

        // Validate metrics against results file This is not the extensive
        // validation of output, but good enough to confirm that metrics have
        // been accepted by OTel Collector.
        assert_metrics_results_contains(&expected_uuid)
    }

    fn emit_and_validate_metrics(meter_provider: SdkMeterProvider) -> Result<()> {
        let expected_uuid = emit_metrics_with_provider(&meter_provider);
        force_flush_with_retry(&meter_provider);

        let shutdown_result = meter_provider.shutdown();
        assert!(shutdown_result.is_ok());
        // We still need to sleep, to give otel-collector a chance to flush to disk
        std::thread::sleep(SLEEP_DURATION);

        // Validate metrics against results file This is not the extensive
        // validation of output, but good enough to confirm that metrics have
        // been accepted by OTel Collector.
        assert_metrics_results_contains(&expected_uuid)?;

        Ok(())
    }

    fn force_flush_with_retry(meter_provider: &SdkMeterProvider) {
        for attempt in 1..=FLUSH_MAX_RETRIES {
            if meter_provider.force_flush().is_ok() {
                return;
            }
            if attempt < FLUSH_MAX_RETRIES {
                std::thread::sleep(FLUSH_RETRY_SLEEP);
            }
        }
        panic!(
            "force_flush failed after {} attempts",
            FLUSH_MAX_RETRIES
        );
    }

    fn wait_for_collector_grpc() {
        for _ in 1..=FLUSH_MAX_RETRIES {
            if TcpStream::connect("127.0.0.1:4317").is_ok() {
                return;
            }
            std::thread::sleep(FLUSH_RETRY_SLEEP);
        }
        panic!("collector gRPC endpoint is not reachable on 127.0.0.1:4317");
    }

    fn emit_metrics_with_provider(meter_provider: &SdkMeterProvider) -> String {
        const METER_NAME: &str = "test_meter";
        const INSTRUMENT_NAME: &str = "test_counter";

        // Use the test-local provider directly to avoid cross-test global races.
        let meter = meter_provider.meter(METER_NAME);
        let expected_uuid = Uuid::new_v4().to_string();
        let counter = meter.u64_counter(INSTRUMENT_NAME).build();
        counter.add(
            10,
            &[
                KeyValue::new("mykey1", expected_uuid.clone()),
                KeyValue::new("mykey2", "myvalue2"),
            ],
        );
        expected_uuid
    }
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
