use opentelemetry::sdk::export::metrics::aggregation;
use opentelemetry::sdk::metrics::{controllers, processors, selectors};
use opentelemetry::sdk::Resource;
use opentelemetry::Context;
use opentelemetry::{metrics::MeterProvider, KeyValue};
use opentelemetry_prometheus::{ExporterConfig, PrometheusExporter};
use prometheus::{Encoder, TextEncoder};

#[test]
fn free_unused_instruments() {
    let cx = Context::new();
    let controller = controllers::basic(processors::factory(
        selectors::simple::histogram(vec![-0.5, 1.0]),
        aggregation::cumulative_temporality_selector(),
    ))
    .with_resource(Resource::new(vec![KeyValue::new("R", "V")]))
    .build();
    let exporter = opentelemetry_prometheus::exporter(controller).init();
    let mut expected = Vec::new();

    {
        let meter =
            exporter
                .meter_provider()
                .unwrap()
                .versioned_meter("test", Some("v0.1.0"), None);
        let counter = meter.f64_counter("counter").init();

        let attributes = vec![KeyValue::new("A", "B"), KeyValue::new("C", "D")];

        counter.add(&cx, 10.0, &attributes);
        counter.add(&cx, 5.3, &attributes);

        expected.push(r#"counter_total{A="B",C="D",R="V",otel_scope_name="test",otel_scope_version="v0.1.0"} 15.3"#);
        expected.push(r#"otel_scope_info{otel_scope_name="test",otel_scope_version="v0.1.0"} 1"#);
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
    let cx = Context::new();
    let controller = controllers::basic(processors::factory(
        selectors::simple::histogram(vec![-0.5, 1.0]),
        aggregation::cumulative_temporality_selector(),
    ))
    .with_resource(Resource::new(vec![KeyValue::new("R", "V")]))
    .build();
    let exporter = opentelemetry_prometheus::exporter(controller)
        .with_config(ExporterConfig::default().with_scope_info(false))
        .init();

    let meter = exporter
        .meter_provider()
        .unwrap()
        .versioned_meter("test", None, None);

    let up_down_counter = meter.f64_up_down_counter("updowncounter").init();
    let counter = meter.f64_counter("counter").init();
    let histogram = meter.f64_histogram("my.histogram").init();

    let attributes = vec![KeyValue::new("A", "B"), KeyValue::new("C", "D")];

    let mut expected = Vec::new();

    counter.add(&cx, 10.0, &attributes);
    counter.add(&cx, 5.3, &attributes);

    expected.push(r#"counter_total{A="B",C="D",R="V"} 15.3"#);

    let cb_attributes = attributes.clone();
    let gauge = meter.i64_observable_gauge("intgauge").init();
    meter
        .register_callback(move |cx| gauge.observe(cx, 1, cb_attributes.as_ref()))
        .unwrap();

    expected.push(r#"intgauge{A="B",C="D",R="V"} 1"#);

    histogram.record(&cx, -0.6, &attributes);
    histogram.record(&cx, -0.4, &attributes);
    histogram.record(&cx, 0.6, &attributes);
    histogram.record(&cx, 20.0, &attributes);

    expected.push(r#"my_histogram_bucket{A="B",C="D",R="V",le="+Inf"} 4"#);
    expected.push(r#"my_histogram_bucket{A="B",C="D",R="V",le="-0.5"} 1"#);
    expected.push(r#"my_histogram_bucket{A="B",C="D",R="V",le="1"} 3"#);
    expected.push(r#"my_histogram_count{A="B",C="D",R="V"} 4"#);
    expected.push(r#"my_histogram_sum{A="B",C="D",R="V"} 19.6"#);

    up_down_counter.add(&cx, 10.0, &attributes);
    up_down_counter.add(&cx, -3.2, &attributes);

    expected.push(r#"updowncounter{A="B",C="D",R="V"} 6.8"#);

    compare_export(&exporter, expected)
}

#[test]
fn test_sanitization() {
    let cx = Context::new();
    let controller = controllers::basic(processors::factory(
        selectors::simple::histogram(vec![-0.5, 1.0]),
        aggregation::cumulative_temporality_selector(),
    ))
    .with_resource(Resource::new(vec![KeyValue::new(
        "service.name",
        "Test Service",
    )]))
    .build();
    let exporter = opentelemetry_prometheus::exporter(controller)
        .with_config(ExporterConfig::default().with_scope_info(false))
        .init();
    let meter = exporter
        .meter_provider()
        .unwrap()
        .versioned_meter("test", None, None);

    let histogram = meter.f64_histogram("http.server.duration").init();
    let attributes = vec![
        KeyValue::new("http.method", "GET"),
        KeyValue::new("http.host", "server"),
    ];
    histogram.record(&cx, -0.6, &attributes);
    histogram.record(&cx, -0.4, &attributes);
    histogram.record(&cx, 0.6, &attributes);
    histogram.record(&cx, 20.0, &attributes);

    let expected = vec![
        r#"http_server_duration_bucket{http_host="server",http_method="GET",service_name="Test Service",le="+Inf"} 4"#,
        r#"http_server_duration_bucket{http_host="server",http_method="GET",service_name="Test Service",le="-0.5"} 1"#,
        r#"http_server_duration_bucket{http_host="server",http_method="GET",service_name="Test Service",le="1"} 3"#,
        r#"http_server_duration_count{http_host="server",http_method="GET",service_name="Test Service"} 4"#,
        r#"http_server_duration_sum{http_host="server",http_method="GET",service_name="Test Service"} 19.6"#,
    ];
    compare_export(&exporter, expected)
}

#[test]
fn test_scope_info() {
    let cx = Context::new();
    let controller = controllers::basic(processors::factory(
        selectors::simple::histogram(vec![-0.5, 1.0]),
        aggregation::cumulative_temporality_selector(),
    ))
    .with_resource(Resource::new(vec![KeyValue::new("R", "V")]))
    .build();
    let exporter = opentelemetry_prometheus::exporter(controller).init();

    let meter = exporter
        .meter_provider()
        .unwrap()
        .versioned_meter("test", Some("v0.1.0"), None);

    let up_down_counter = meter.f64_up_down_counter("updowncounter").init();
    let counter = meter.f64_counter("counter").init();
    let histogram = meter.f64_histogram("my.histogram").init();

    let attributes = vec![KeyValue::new("A", "B"), KeyValue::new("C", "D")];

    let mut expected = Vec::new();

    counter.add(&cx, 10.0, &attributes);
    counter.add(&cx, 5.3, &attributes);

    expected.push(r#"counter_total{A="B",C="D",R="V",otel_scope_name="test",otel_scope_version="v0.1.0"} 15.3"#);

    let cb_attributes = attributes.clone();
    let gauge = meter.i64_observable_gauge("intgauge").init();
    meter
        .register_callback(move |cx| gauge.observe(cx, 1, cb_attributes.as_ref()))
        .unwrap();

    expected.push(
        r#"intgauge{A="B",C="D",R="V",otel_scope_name="test",otel_scope_version="v0.1.0"} 1"#,
    );

    histogram.record(&cx, -0.6, &attributes);
    histogram.record(&cx, -0.4, &attributes);
    histogram.record(&cx, 0.6, &attributes);
    histogram.record(&cx, 20.0, &attributes);

    expected.push(r#"my_histogram_bucket{A="B",C="D",R="V",otel_scope_name="test",otel_scope_version="v0.1.0",le="+Inf"} 4"#);
    expected.push(r#"my_histogram_bucket{A="B",C="D",R="V",otel_scope_name="test",otel_scope_version="v0.1.0",le="-0.5"} 1"#);
    expected.push(r#"my_histogram_bucket{A="B",C="D",R="V",otel_scope_name="test",otel_scope_version="v0.1.0",le="1"} 3"#);
    expected.push(r#"my_histogram_count{A="B",C="D",R="V",otel_scope_name="test",otel_scope_version="v0.1.0"} 4"#);
    expected.push(r#"my_histogram_sum{A="B",C="D",R="V",otel_scope_name="test",otel_scope_version="v0.1.0"} 19.6"#);

    up_down_counter.add(&cx, 10.0, &attributes);
    up_down_counter.add(&cx, -3.2, &attributes);

    expected.push(r#"updowncounter{A="B",C="D",R="V",otel_scope_name="test",otel_scope_version="v0.1.0"} 6.8"#);
    expected.push(r#"otel_scope_info{otel_scope_name="test",otel_scope_version="v0.1.0"} 1"#);

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
