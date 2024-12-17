//! OTLP integration tests for metrics
//! Note: these are all expressed using Serde types for the deserialized metrics records.
//! We might consider changing this once we have fixed the issue identified in the #[ignore]d test
//! `test_roundtrip_example_data` - as the roundtripping is currently broken for metrics.
//!
#![cfg(unix)]

use anyhow::{Context, Result};
use ctor::dtor;
use integration_test_runner::metrics_asserter::{read_metrics_from_json, MetricsAsserter};
use integration_test_runner::test_utils;
use integration_test_runner::test_utils::start_collector_container;
use opentelemetry::KeyValue;
use opentelemetry_otlp::MetricExporter;
use opentelemetry_proto::tonic::metrics::v1::MetricsData;
use opentelemetry_sdk::metrics::{MeterProviderBuilder, PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::Resource;
use serde_json::Value;
use std::fs;
use std::fs::File;
use std::sync::Mutex;
use std::time::Duration;

static SETUP_DONE: Mutex<bool> = Mutex::new(false);

static RESULT_PATH: &str = "actual/metrics.json";

/// Initializes the OpenTelemetry metrics pipeline
async fn init_metrics() -> SdkMeterProvider {
    let exporter = create_exporter();

    let reader = PeriodicReader::builder(exporter)
        .with_interval(Duration::from_millis(100))
        .with_timeout(Duration::from_secs(1))
        .build();

    let resource = Resource::builder_empty()
        .with_service_name("metrics-integration-test")
        .build();

    let meter_provider = MeterProviderBuilder::default()
        .with_resource(resource)
        .with_reader(reader)
        .build();

    opentelemetry::global::set_meter_provider(meter_provider.clone());

    meter_provider
}

///
/// Creates an exporter using the appropriate HTTP or gRPC client based on
/// the configured features.
///
fn create_exporter() -> MetricExporter {
    let exporter_builder = MetricExporter::builder();

    #[cfg(feature = "tonic-client")]
    let exporter_builder = exporter_builder.with_tonic();
    #[cfg(not(feature = "tonic-client"))]
    #[cfg(any(
        feature = "hyper-client",
        feature = "reqwest-client",
        feature = "reqwest-blocking-client"
    ))]
    let exporter_builder = exporter_builder.with_http();

    exporter_builder
        .build()
        .expect("Failed to build MetricExporter")
}

///
/// Retrieves the latest metrics for the given scope. Each test should use
/// its own scope, so that we can easily pull the data for it out from the rest
/// of the data.
///
/// This will also retrieve the resource attached to the scope.
///
pub fn fetch_latest_metrics_for_scope(scope_name: &str) -> Result<Value> {
    // Open the file and fetch the contents
    let contents = fs::read_to_string(test_utils::METRICS_FILE)?;

    // Find the last parseable metrics line that contains the desired scope
    let json_line = contents
        .lines()
        .rev()
        .find_map(|line| {
            // Attempt to parse the line as JSON
            serde_json::from_str::<Value>(line)
                .ok()
                .and_then(|mut json_line| {
                    // Check if it contains the specified scope
                    if let Some(resource_metrics) = json_line
                        .get_mut("resourceMetrics")
                        .and_then(|v| v.as_array_mut())
                    {
                        resource_metrics.retain_mut(|resource| {
                            if let Some(scope_metrics) = resource
                                .get_mut("scopeMetrics")
                                .and_then(|v| v.as_array_mut())
                            {
                                scope_metrics.retain(|scope| {
                                    scope
                                        .get("scope")
                                        .and_then(|s| s.get("name"))
                                        .and_then(|name| name.as_str())
                                        .map_or(false, |n| n == scope_name)
                                });

                                // Keep the resource only if it has any matching `ScopeMetrics`
                                !scope_metrics.is_empty()
                            } else {
                                false
                            }
                        });

                        // If any resource metrics remain, return this line
                        if !resource_metrics.is_empty() {
                            return Some(json_line);
                        }
                    }

                    None
                })
        })
        .with_context(|| {
            format!(
                "No valid JSON line containing scope `{}` found.",
                scope_name
            )
        })?;

    Ok(json_line)
}

///
/// Performs setup for metrics tests
///
async fn setup_metrics_test() -> Result<()> {
    // Make sure the collector container is running
    start_collector_container().await?;

    let mut done = SETUP_DONE.lock().unwrap();
    if !*done {
        println!("Running setup before any tests...");
        *done = true; // Mark setup as done

        // Initialise the metrics subsystem
        _ = init_metrics().await;
    }

    // Truncate results
    _ = File::create(RESULT_PATH).expect("it's good");

    Ok(())
}

///
/// Check that the metrics for the given scope match what we expect. This
/// includes zeroing out timestamps, which we reasonably expect not to match.
///
pub fn validate_metrics_against_results(scope_name: &str) -> Result<()> {
    // Define the results file path
    let results_file_path = format!("./expected/metrics/{}.json", scope_name);

    // Fetch the actual metrics for the given scope
    let actual_metrics = fetch_latest_metrics_for_scope(scope_name)
        .context(format!("Failed to fetch metrics for scope: {}", scope_name))?;

    // Read the expected metrics from the results file
    let expected_metrics = {
        let file = File::open(&results_file_path).context(format!(
            "Failed to open results file: {}",
            results_file_path
        ))?;
        read_metrics_from_json(file)
    }?;

    // Compare the actual metrics with the expected metrics
    MetricsAsserter::new(actual_metrics, expected_metrics).assert();

    Ok(())
}

///
/// TODO - the HTTP metrics exporters do not seem to flush at the moment.
/// TODO - fix this asynchronously.
///
#[cfg(test)]
#[cfg(not(feature = "hyper-client"))]
#[cfg(not(feature = "reqwest-client"))]
#[cfg(not(feature = "reqwest-blocking-client"))]
mod tests {

    use super::*;
    use opentelemetry::metrics::MeterProvider;

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
        let _result_path = setup_metrics_test().await;
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

        tokio::time::sleep(Duration::from_secs(2)).await;

        // Validate metrics against results file
        validate_metrics_against_results(METER_NAME)?;

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    // #[ignore] // skip when running unit test
    async fn test_histogram() -> Result<()> {
        _ = setup_metrics_test().await;
        const METER_NAME: &str = "test_histogram_meter";

        // Add data to histogram
        let meter = opentelemetry::global::meter_provider().meter(METER_NAME);
        let histogram = meter.u64_histogram("example_histogram").build();
        histogram.record(42, &[KeyValue::new("mykey3", "myvalue4")]);
        tokio::time::sleep(Duration::from_secs(5)).await;

        validate_metrics_against_results(METER_NAME)?;

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    // #[ignore] // skip when running unit test
    async fn test_up_down_counter() -> Result<()> {
        _ = setup_metrics_test().await;
        const METER_NAME: &str = "test_up_down_meter";

        // Add data to up_down_counter
        let meter = opentelemetry::global::meter_provider().meter(METER_NAME);
        let up_down_counter = meter.i64_up_down_counter("example_up_down_counter").build();
        up_down_counter.add(-1, &[KeyValue::new("mykey5", "myvalue5")]);
        tokio::time::sleep(Duration::from_secs(5)).await;

        validate_metrics_against_results(METER_NAME)?;

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[ignore]
    async fn test_flush_on_shutdown() -> Result<()> {
        const METER_NAME: &str = "test_flush_on_shutdown";

        // Set everything up by hand, so that we can shutdown() the exporter
        // and make sure our data is flushed through.

        // Make sure the collector is running
        start_collector_container().await?;

        // Set up the exporter
        let exporter = create_exporter();
        let reader = PeriodicReader::builder(exporter)
            .with_interval(Duration::from_millis(100))
            .with_timeout(Duration::from_secs(1))
            .build();
        let resource = Resource::builder_empty()
            .with_service_name("metrics-integration-test")
            .build();
        let meter_provider = MeterProviderBuilder::default()
            .with_resource(resource)
            .with_reader(reader)
            .build();

        // Send something
        let meter = meter_provider.meter(METER_NAME);
        let counter = meter.u64_counter("counter_").build();
        counter.add(123, &[]);

        // Shutdown
        meter_provider.shutdown()?;

        // We still need to sleep, to give otel-collector a chance to flush to disk
        tokio::time::sleep(Duration::from_secs(2)).await;

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
