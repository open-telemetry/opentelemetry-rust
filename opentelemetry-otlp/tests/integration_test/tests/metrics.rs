use std::{fs::File, io::Write};

use integration_test_runner::metrics_asserter::{read_metrics_from_json, MetricsAsserter};
use opentelemetry_proto::tonic::metrics::v1::MetricsData;

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
