use opentelemetry::api::{metrics::MeterProvider, KeyValue};
use opentelemetry::sdk::Resource;
use opentelemetry_prometheus::PrometheusExporter;
use prometheus::{Encoder, TextEncoder};

#[test]
fn test_add() {
    let exporter = opentelemetry_prometheus::exporter()
        .with_default_histogram_boundaries(vec![-0.5, 1.0])
        .with_resource(Resource::new(vec![KeyValue::new("R", "V")]))
        .init();

    let meter = exporter.provider().unwrap().meter("test");

    let counter = meter.f64_counter("counter").init();
    let value_recorder = meter.f64_value_recorder("value_recorder").init();

    let labels = vec![KeyValue::new("A", "B"), KeyValue::new("C", "D")];

    let mut expected = Vec::new();

    counter.add(10.0, &labels);
    counter.add(5.3, &labels);

    expected.push("counter{A=\"B\",C=\"D\",R=\"V\"} 15.3");

    value_recorder.record(-0.6, &labels);
    value_recorder.record(-0.4, &labels);
    value_recorder.record(0.6, &labels);
    value_recorder.record(20.0, &labels);

    expected.push("value_recorder_bucket{A=\"B\",C=\"D\",R=\"V\",le=\"+Inf\"} 4");
    expected.push("value_recorder_bucket{A=\"B\",C=\"D\",R=\"V\",le=\"-0.5\"} 1");
    expected.push("value_recorder_bucket{A=\"B\",C=\"D\",R=\"V\",le=\"1\"} 3");
    expected.push("value_recorder_count{A=\"B\",C=\"D\",R=\"V\"} 4");
    expected.push("value_recorder_sum{A=\"B\",C=\"D\",R=\"V\"} 19.6");

    compare_export(&exporter, expected)
}

fn compare_export(exporter: &PrometheusExporter, mut expected: Vec<&'static str>) {
    let mut output = Vec::new();
    let encoder = TextEncoder::new();
    let metric_families = exporter.registry().gather();
    encoder.encode(&metric_families, &mut output).unwrap();
    let output_string = String::from_utf8(output).unwrap();

    let mut metrics_only = output_string
        .split_terminator('\n')
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .collect::<Vec<_>>();

    metrics_only.sort_unstable();
    expected.sort_unstable();

    assert_eq!(expected.join("\n"), metrics_only.join("\n"))
}
