use opentelemetry::sdk::Resource;
use opentelemetry::{
    metrics::{MeterProvider, ObserverResult},
    KeyValue,
};
use opentelemetry_prometheus::PrometheusExporter;
use prometheus::{Encoder, TextEncoder};

#[test]
fn free_unused_instruments() {
    let exporter = opentelemetry_prometheus::exporter()
        .with_default_histogram_boundaries(vec![-0.5, 1.0])
        .with_resource(Resource::new(vec![KeyValue::new("R", "V")]))
        .init();
    let mut expected = Vec::new();

    {
        let meter = exporter.provider().unwrap().meter("test", None);
        let counter = meter.f64_counter("counter").init();

        let labels = vec![KeyValue::new("A", "B"), KeyValue::new("C", "D")];

        counter.add(10.0, &labels);
        counter.add(5.3, &labels);

        expected.push(r#"counter{A="B",C="D",R="V"} 15.3"#);
    }
    // Standard export
    compare_export(&exporter, expected.clone());
    // Final export before instrument dropped
    compare_export(&exporter, expected.clone());
    // Instrument dropped, but last value kept by prom exporter
    compare_export(&exporter, expected);
}

#[test]
fn test_add() {
    let exporter = opentelemetry_prometheus::exporter()
        .with_default_histogram_boundaries(vec![-0.5, 1.0])
        .with_resource(Resource::new(vec![KeyValue::new("R", "V")]))
        .init();

    let meter = exporter.provider().unwrap().meter("test", None);

    let up_down_counter = meter.f64_up_down_counter("updowncounter").init();
    let counter = meter.f64_counter("counter").init();
    let value_recorder = meter.f64_value_recorder("value_recorder").init();

    let labels = vec![KeyValue::new("A", "B"), KeyValue::new("C", "D")];

    let mut expected = Vec::new();

    counter.add(10.0, &labels);
    counter.add(5.3, &labels);

    expected.push(r#"counter{A="B",C="D",R="V"} 15.3"#);

    let cb_labels = labels.clone();
    let _observer = meter
        .i64_value_observer("intobserver", move |result: ObserverResult<i64>| {
            result.observe(1, cb_labels.as_ref())
        })
        .init();

    expected.push(r#"intobserver{A="B",C="D",R="V"} 1"#);

    value_recorder.record(-0.6, &labels);
    value_recorder.record(-0.4, &labels);
    value_recorder.record(0.6, &labels);
    value_recorder.record(20.0, &labels);

    expected.push(r#"value_recorder_bucket{A="B",C="D",R="V",le="+Inf"} 4"#);
    expected.push(r#"value_recorder_bucket{A="B",C="D",R="V",le="-0.5"} 1"#);
    expected.push(r#"value_recorder_bucket{A="B",C="D",R="V",le="1"} 3"#);
    expected.push(r#"value_recorder_count{A="B",C="D",R="V"} 4"#);
    expected.push(r#"value_recorder_sum{A="B",C="D",R="V"} 19.6"#);

    up_down_counter.add(10.0, &labels);
    up_down_counter.add(-3.2, &labels);

    expected.push(r#"updowncounter{A="B",C="D",R="V"} 6.8"#);

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
