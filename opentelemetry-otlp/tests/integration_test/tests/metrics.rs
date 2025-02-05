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
use opentelemetry_sdk::metrics::SdkMeterProvider;
use std::time::Duration;

const SLEEP_DURATION: Duration = Duration::from_secs(5);

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
        emit_and_validate_metrics(meter_provider)
    }

    async fn metric_helper_tokio_current() -> Result<()> {
        let meter_provider = setup_metrics_tokio().await;

        const METER_NAME: &str = "test_meter";
        const INSTRUMENT_NAME: &str = "test_counter";

        let meter = opentelemetry::global::meter_provider().meter(METER_NAME);
        let expected_uuid = Uuid::new_v4().to_string();
        let counter = meter.u64_counter(INSTRUMENT_NAME).build();
        counter.add(
            10,
            &[
                KeyValue::new("mykey1", expected_uuid.clone()),
                KeyValue::new("mykey2", "myvalue2"),
            ],
        );

        // In tokio::current_thread flavor, shutdown must be done in a separate thread
        let shutdown_resut = Handle::current()
            .spawn_blocking(move || meter_provider.shutdown())
            .await
            .unwrap();
        assert!(shutdown_resut.is_ok());
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
        const METER_NAME: &str = "test_meter";
        const INSTRUMENT_NAME: &str = "test_counter";

        // Add data to u64_counter
        let meter = opentelemetry::global::meter_provider().meter(METER_NAME);
        let expected_uuid = Uuid::new_v4().to_string();
        let counter = meter.u64_counter(INSTRUMENT_NAME).build();
        counter.add(
            10,
            &[
                KeyValue::new("mykey1", expected_uuid.clone()),
                KeyValue::new("mykey2", "myvalue2"),
            ],
        );

        let shutdown_resut = meter_provider.shutdown();
        assert!(shutdown_resut.is_ok());
        // We still need to sleep, to give otel-collector a chance to flush to disk
        std::thread::sleep(SLEEP_DURATION);

        // Validate metrics against results file This is not the extensive
        // validation of output, but good enough to confirm that metrics have
        // been accepted by OTel Collector.
        assert_metrics_results_contains(&expected_uuid)
    }

    fn emit_and_validate_metrics(meter_provider: SdkMeterProvider) -> Result<()> {
        const METER_NAME: &str = "test_meter";
        const INSTRUMENT_NAME: &str = "test_counter";

        // Add data to u64_counter
        let meter = opentelemetry::global::meter_provider().meter(METER_NAME);
        let expected_uuid = Uuid::new_v4().to_string();
        let counter = meter.u64_counter(INSTRUMENT_NAME).build();
        counter.add(
            10,
            &[
                KeyValue::new("mykey1", expected_uuid.clone()),
                KeyValue::new("mykey2", "myvalue2"),
            ],
        );

        let shutdown_resut = meter_provider.shutdown();
        assert!(shutdown_resut.is_ok());
        // We still need to sleep, to give otel-collector a chance to flush to disk
        std::thread::sleep(SLEEP_DURATION);

        // Validate metrics against results file This is not the extensive
        // validation of output, but good enough to confirm that metrics have
        // been accepted by OTel Collector.
        assert_metrics_results_contains(&expected_uuid)?;

        Ok(())
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
