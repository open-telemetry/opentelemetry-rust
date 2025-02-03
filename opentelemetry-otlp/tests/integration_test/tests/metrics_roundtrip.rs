//! OTLP integration tests for metrics. These tests cover the breadth of Metric
//! API by testing all instrument types and ensuring that the data is correctly
//! exported to the collector by validating the exported data against the
//! expected results.
//! Note: these are all expressed using Serde types for the deserialized metrics records.
//! We might consider changing this once we have fixed the issue identified in the #[ignore]d test
//! `test_roundtrip_example_data` - as the roundtripping is currently broken for metrics.
//!
#![cfg(unix)]

use anyhow::{Ok, Result};
use ctor::dtor;
use integration_test_runner::test_utils;
use opentelemetry::KeyValue;
use opentelemetry_proto::tonic::metrics::v1::MetricsData;
use serde_json::Value;

#[cfg(test)]
#[cfg(any(feature = "tonic-client", feature = "reqwest-blocking-client"))]
mod metrictests_roundtrip {
    use integration_test_runner::metric_helpers::{
        self, validate_metrics_against_results, SLEEP_DURATION,
    };

    use super::*;

    ///
    /// Validate JSON/Protobuf models roundtrip correctly.
    ///
    /// TODO - this test fails currently. Fields disappear, such as the actual value of a given metric.
    /// This appears to be on the _deserialization_ side.
    /// Issue: https://github.com/open-telemetry/opentelemetry-rust/issues/2434
    ///
    #[tokio::test]
    #[ignore]
    async fn test_roundtrip_example_data() -> Result<()> {
        let metrics_in = include_str!("../expected/metrics/test_u64_counter_meter.json");
        let metrics: MetricsData = serde_json::from_str(metrics_in)?;
        let metrics_out = serde_json::to_string(&metrics)?;

        println!("{:}", metrics_out);

        let metrics_in_json: Value = serde_json::from_str(metrics_in)?;
        let metrics_out_json: Value = serde_json::from_str(&metrics_out)?;

        assert_eq!(metrics_in_json, metrics_out_json);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_u64_counter() -> Result<()> {
        let meter_provider = metric_helpers::setup_metrics_tokio().await;
        const METER_NAME: &str = "test_u64_counter_meter";

        // Add data to u64_counter
        let meter = opentelemetry::global::meter_provider().meter(METER_NAME);

        let counter = meter.u64_counter("counter_u64").build();
        counter.add(
            10,
            &[
                KeyValue::new("mykey1", "myvalue1"),
                KeyValue::new("mykey2", "myvalue2"),
            ],
        );

        meter_provider.shutdown()?;
        tokio::time::sleep(SLEEP_DURATION).await;

        // Validate metrics against results file
        validate_metrics_against_results(METER_NAME)?;

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_histogram() -> Result<()> {
        let meter_provider = metric_helpers::setup_metrics_tokio().await;
        const METER_NAME: &str = "test_histogram_meter";

        // Add data to histogram
        let meter = opentelemetry::global::meter_provider().meter(METER_NAME);
        let histogram = meter.u64_histogram("example_histogram").build();
        histogram.record(42, &[KeyValue::new("mykey3", "myvalue4")]);

        meter_provider.shutdown()?;
        tokio::time::sleep(SLEEP_DURATION).await;

        validate_metrics_against_results(METER_NAME)?;

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_up_down_counter() -> Result<()> {
        let meter_provider = metric_helpers::setup_metrics_tokio().await;
        const METER_NAME: &str = "test_up_down_meter";

        // Add data to up_down_counter
        let meter = opentelemetry::global::meter_provider().meter(METER_NAME);
        let up_down_counter = meter.i64_up_down_counter("example_up_down_counter").build();
        up_down_counter.add(-1, &[KeyValue::new("mykey5", "myvalue5")]);

        meter_provider.shutdown()?;
        tokio::time::sleep(SLEEP_DURATION).await;

        validate_metrics_against_results(METER_NAME)?;

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
