use opentelemetry::{metrics::MeterProvider, InstrumentationScope, KeyValue};
use opentelemetry_sdk::metrics::SdkMeterProvider;

#[test]
fn test_basic_exposition_format() {
    // Create the Prometheus exporter
    let exporter = opentelemetry_prometheus::exporter()
        .build()
        .expect("failed to build exporter");

    // Create a meter provider with the exporter
    let provider = SdkMeterProvider::builder()
        .with_reader(exporter.clone())
        .build();

    // Create an instrumentation scope with version and custom attributes
    let scope = InstrumentationScope::builder("minimal-example")
        .with_version("1.0.0")
        .with_attributes(vec![
            KeyValue::new("deployment.environment", "test"),
            KeyValue::new("service.instance.id", "instance-123"),
        ])
        .build();

    // Create a meter with the scope
    let meter = provider.meter_with_scope(scope);
    let counter = meter
        .u64_counter("requests")
        .with_description("Number of requests")
        .with_unit("1")
        .build();

    // Record some measurements
    counter.add(10, &[KeyValue::new("method", "GET")]);
    counter.add(5, &[KeyValue::new("method", "POST")]);

    // Scrape and print the metrics
    let output = exporter.export().expect("failed to export metrics");
    println!("{}", output);

    // Verify expected content is present
    let expected = "\
# HELP requests_ratio_total Number of requests
# TYPE requests_ratio_total counter
requests_ratio_total{method=\"GET\",otel_scope_name=\"minimal-example\",otel_scope_version=\"1.0.0\",otel_scope_deployment.environment=\"test\",otel_scope_service.instance.id=\"instance-123\"} 10
requests_ratio_total{method=\"POST\",otel_scope_name=\"minimal-example\",otel_scope_version=\"1.0.0\",otel_scope_deployment.environment=\"test\",otel_scope_service.instance.id=\"instance-123\"} 5
# HELP otel_scope_info Instrumentation Scope metadata
# TYPE otel_scope_info gauge
otel_scope_info{otel_scope_name=\"minimal-example\",otel_scope_version=\"1.0.0\",otel_scope_deployment.environment=\"test\",otel_scope_service.instance.id=\"instance-123\"} 1
# HELP target_info Target metadata
# TYPE target_info gauge
target_info{service_name=\"unknown_service\",telemetry_sdk_language=\"rust\",telemetry_sdk_name=\"opentelemetry\",telemetry_sdk_version=\"0.29.0\"} 1
";

    assert_eq!(output, expected);
}
