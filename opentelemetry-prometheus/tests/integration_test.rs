use std::env;

use opentelemetry::sdk::Resource;
use opentelemetry::{
    metrics::{BatchObserverResult, MeterProvider, ObserverResult},
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
fn batch() {
    let exporter = opentelemetry_prometheus::exporter()
        .with_resource(Resource::new(vec![KeyValue::new("R", "V")]))
        .init();
    let meter = exporter.provider().unwrap().meter("test", None);
    let mut expected = Vec::new();

    meter.batch_observer(|batch| {
        let uint_observer = batch.u64_value_observer("uint_observer").init();
        let float_observer = batch.f64_value_observer("float_observer").init();

        move |result: BatchObserverResult| {
            result.observe(
                &[KeyValue::new("A", "B")],
                &[
                    uint_observer.observation(2),
                    float_observer.observation(3.1),
                ],
            );
        }
    });

    expected.push(r#"uint_observer{A="B",R="V"} 2"#);
    expected.push(r#"float_observer{A="B",R="V"} 3.1"#);
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

#[test]
fn test_sanitization() {
    let exporter = opentelemetry_prometheus::exporter()
        .with_default_histogram_boundaries(vec![-0.5, 1.0])
        .with_resource(Resource::new(vec![KeyValue::new(
            "service.name",
            "Test Service",
        )]))
        .init();
    let meter = exporter.provider().unwrap().meter("test", None);

    let value_recorder = meter.f64_value_recorder("http.server.duration").init();
    let labels = vec![
        KeyValue::new("http.method", "GET"),
        KeyValue::new("http.host", "server"),
    ];
    value_recorder.record(-0.6, &labels);
    value_recorder.record(-0.4, &labels);
    value_recorder.record(0.6, &labels);
    value_recorder.record(20.0, &labels);

    let expected = vec![
        r#"http_server_duration_bucket{http_host="server",http_method="GET",service_name="Test Service",le="+Inf"} 4"#,
        r#"http_server_duration_bucket{http_host="server",http_method="GET",service_name="Test Service",le="-0.5"} 1"#,
        r#"http_server_duration_bucket{http_host="server",http_method="GET",service_name="Test Service",le="1"} 3"#,
        r#"http_server_duration_count{http_host="server",http_method="GET",service_name="Test Service"} 4"#,
        r#"http_server_duration_sum{http_host="server",http_method="GET",service_name="Test Service"} 19.6"#,
    ];
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

#[test]
fn test_from_env() {
    let otel_exporter_prometheus_host = "OTEL_EXPORTER_PROMETHEUS_HOST";
    let otel_exporter_prometheus_port = "OTEL_EXPORTER_PROMETHEUS_PORT";

    // environment variables do not exist
    env::remove_var(otel_exporter_prometheus_host);
    env::remove_var(otel_exporter_prometheus_port);
    let exporter = opentelemetry_prometheus::ExporterBuilder::from_env().init();
    assert_eq!(exporter.host(), "0.0.0.0");
    assert_eq!(exporter.port(), "9464");

    // environment variables are available and non-empty strings
    env::set_var(otel_exporter_prometheus_host, "prometheus-test");
    env::set_var(otel_exporter_prometheus_port, "9000");

    let exporter = opentelemetry_prometheus::ExporterBuilder::from_env().init();
    assert_eq!(exporter.host(), "prometheus-test");
    assert_eq!(exporter.port(), "9000");

    // environment variables are available and empty
    env::set_var(otel_exporter_prometheus_host, "");
    env::set_var(otel_exporter_prometheus_port, "");
    let exporter = opentelemetry_prometheus::ExporterBuilder::from_env().init();
    assert_eq!(exporter.host(), "0.0.0.0");
    assert_eq!(exporter.port(), "9464");
}
