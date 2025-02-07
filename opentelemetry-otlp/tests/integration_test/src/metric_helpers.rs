#![cfg(unix)]
use crate::test_utils;
use anyhow::Result;
use anyhow::{Context, Ok};
use opentelemetry_otlp::MetricExporter;
use opentelemetry_sdk::metrics::{MeterProviderBuilder, PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::Resource;
use serde_json::Value;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::time::Duration;
use tracing::info;

static RESULT_PATH: &str = "actual/metrics.json";
pub const SLEEP_DURATION: Duration = Duration::from_secs(5);

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

/// Initializes the OpenTelemetry metrics pipeline
fn init_meter_provider() -> SdkMeterProvider {
    let exporter = create_exporter();
    let resource = Resource::builder_empty()
        .with_service_name("metrics-integration-test")
        .build();
    let meter_provider = MeterProviderBuilder::default()
        .with_resource(resource)
        .with_periodic_exporter(exporter)
        .build();
    opentelemetry::global::set_meter_provider(meter_provider.clone());
    meter_provider
}

///
/// Performs setup for metrics tests using the Tokio runtime.
///
pub async fn setup_metrics_tokio() -> SdkMeterProvider {
    let _ = test_utils::start_collector_container().await;
    // Truncate results
    _ = File::create(RESULT_PATH).expect("it's good");
    info!("Truncated metrics file");

    init_meter_provider()
}

///
/// Performs setup for metrics tests.
///
pub fn setup_metrics_non_tokio(
    initialize_metric_in_tokio: bool,
) -> (SdkMeterProvider, tokio::runtime::Runtime) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let meter_provider: SdkMeterProvider = if initialize_metric_in_tokio {
        // Initialize the logger provider inside the Tokio runtime
        rt.block_on(async {
            // Setup the collector container inside Tokio runtime
            let _ = test_utils::start_collector_container().await;
            init_meter_provider()
        })
    } else {
        rt.block_on(async {
            let _ = test_utils::start_collector_container().await;
        });

        // Initialize the logger provider outside the Tokio runtime
        init_meter_provider()
    };

    (meter_provider, rt)
}

///
/// Check that the results contain the given string.
///
pub fn assert_metrics_results_contains(expected_content: &str) -> Result<()> {
    // let contents = fs::read_to_string(test_utils::METRICS_FILE)?;
    let file = File::open(test_utils::METRICS_FILE)?;
    let mut contents = String::new();
    let mut reader = std::io::BufReader::new(&file);
    reader.read_to_string(&mut contents)?;
    assert!(
        contents.contains(expected_content),
        "Expected content {} not found in actual content {}",
        expected_content,
        contents
    );
    Ok(())
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

pub fn read_metrics_from_json(file: File) -> Result<Value> {
    // Create a buffered reader for the file
    let mut reader = BufReader::new(file);
    let mut contents = String::new();

    // Read the file contents into a string
    reader
        .read_to_string(&mut contents)
        .expect("Failed to read json file");

    // Parse the contents into a JSON Value
    let metrics_data: Value = serde_json::from_str(&contents)?;
    Ok(metrics_data)
}

pub struct MetricsAsserter {
    results: Value,
    expected: Value,
}

impl MetricsAsserter {
    pub fn new(results: Value, expected: Value) -> Self {
        MetricsAsserter { results, expected }
    }

    pub fn assert(mut self) {
        // Normalize JSON by cleaning out timestamps
        Self::zero_out_timestamps(&mut self.results);
        Self::zero_out_timestamps(&mut self.expected);

        // Perform the assertion
        assert_eq!(
            self.results, self.expected,
            "Metrics did not match. Results: {:#?}, Expected: {:#?}",
            self.results, self.expected
        );
    }

    /// Recursively removes or zeros out timestamp fields in the JSON
    fn zero_out_timestamps(value: &mut Value) {
        match value {
            Value::Object(map) => {
                for (key, val) in map.iter_mut() {
                    if key == "startTimeUnixNano" || key == "timeUnixNano" {
                        *val = Value::String("0".to_string());
                    } else {
                        Self::zero_out_timestamps(val);
                    }
                }
            }
            Value::Array(array) => {
                for item in array.iter_mut() {
                    Self::zero_out_timestamps(item);
                }
            }
            _ => {}
        }
    }
}
