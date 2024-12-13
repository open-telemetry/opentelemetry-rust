use integration_test_runner::logs_asserter::{read_logs_from_json, LogsAsserter};
use integration_test_runner::metrics_asserter::{
    read_metrics_from_json, read_metrics_from_json_string, MetricsAsserter,
};
use opentelemetry::metrics::MeterProvider;
use opentelemetry::trace::FutureExt;
use opentelemetry::KeyValue;
use opentelemetry_otlp::MetricExporter;
use opentelemetry_proto::tonic::metrics::v1::MetricsData;
use opentelemetry_sdk::metrics::{
    MeterProviderBuilder, MetricError, PeriodicReader, SdkMeterProvider,
};
use opentelemetry_sdk::{runtime, Resource};
use std::error::Error;
use std::io::{BufRead, BufReader, Read};
use std::os::unix::fs::MetadataExt;
use std::time::Duration;
use std::{fs::File, io::Write};

fn init_metrics() -> SdkMeterProvider {
    // Create the OTLP exporter
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

    let exporter = exporter_builder
        .build()
        .expect("Failed to build MetricExporter");

    // Wrap the exporter in a MetricReader
    // Create a periodic reader with desired interval and timeout
    let reader = PeriodicReader::builder(exporter)
        .with_interval(Duration::from_millis(100)) // Adjust the interval as needed
        .with_timeout(Duration::from_secs(1)) // Adjust the timeout as needed
        .build();

    // Add resource information for this meter provider
    let resource = Resource::new(vec![KeyValue::new(
        opentelemetry_semantic_conventions::resource::SERVICE_NAME,
        "metrics-integration-test",
    )]);

    // Build the SdkMeterProvider
    let meter_provider = MeterProviderBuilder::default()
        .with_resource(resource)
        .with_reader(reader)
        .build();

    // Set the meter provider globally
    opentelemetry::global::set_meter_provider(meter_provider.clone());

    meter_provider
}

pub async fn metrics() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let meter_provider = init_metrics();

    let meter = meter_provider.meter("meter");
    let counter = meter.u64_counter("counter_u64").build();
    counter.add(
        10,
        &[
            KeyValue::new("mykey1", "myvalue1"),
            KeyValue::new("mykey2", "myvalue2"),
        ],
    );

    let histogram = meter.u64_histogram("example_histogram").build();
    histogram.record(42, &[KeyValue::new("mykey3", "myvalue4")]);

    let up_down_counter = meter.i64_up_down_counter("example_up_down_counter").build();
    up_down_counter.add(-1, &[KeyValue::new("mykey5", "myvalue5")]);

    Ok(())
}

pub fn assert_metrics_results(result: &str, expected: &str) {
    let left = read_metrics_from_json(File::open(expected).unwrap());

    // For the results file for metrics - what the OTLP collector outputs -
    // we get a line per unit of time. So let's grab the last line, and compare that
    // to our expected data.
    let last_line = BufReader::new(File::open(result).expect("can open metrics results file"))
        .lines()
        .last()
        .expect("metrics results has a final line")
        .unwrap();
    let right = read_metrics_from_json_string(&last_line);

    MetricsAsserter::new(left, right).assert();

    assert!(File::open(result).unwrap().metadata().unwrap().size() > 0)
}

#[test]
fn test_serde() {
    let metrics = read_metrics_from_json(File::open("./expected/metrics.json").unwrap());

    let json = serde_json::to_string_pretty(&MetricsData {
        resource_metrics: metrics,
    })
    .expect("Failed to serialize metrics");

    // Write to file.
    let mut file = File::create("./expected/serialized_metrics.json").unwrap();
    file.write_all(json.as_bytes()).unwrap();

    let left = read_metrics_from_json(File::open("./expected/metrics.json").unwrap());
    let right = read_metrics_from_json(File::open("./expected/serialized_metrics.json").unwrap());

    MetricsAsserter::new(left, right).assert();
}

#[test]
#[should_panic(expected = "assertion `left == right` failed")]
pub fn test_assert_metrics_eq_failure() {
    let left = read_metrics_from_json(File::open("./expected/metrics.json").unwrap());
    let right = read_metrics_from_json(File::open("./expected/different_metrics.json").unwrap());

    MetricsAsserter::new(left, right).assert();
}

///
/// Make sure that metrics that are the same are equal(...)
///
#[test]
pub fn test_assert_metrics_eq() {
    let metrics = read_metrics_from_json(File::open("./expected/metrics.json").unwrap());
    MetricsAsserter::new(metrics.clone(), metrics).assert();
}
