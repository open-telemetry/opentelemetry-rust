use log::{info, Level};
use opentelemetry::{
    global,
    trace::{TraceContextExt, Tracer},
    InstrumentationScope, KeyValue,
};
use opentelemetry_appender_log::OpenTelemetryLogBridge;
// use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_otlp::{LogExporter, MetricExporter, Protocol, SpanExporter};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::{
    logs::SdkLoggerProvider, metrics::SdkMeterProvider, trace::SdkTracerProvider,
};
use std::{error::Error, sync::OnceLock};

fn get_resource() -> Resource {
    static RESOURCE: OnceLock<Resource> = OnceLock::new();
    RESOURCE
        .get_or_init(|| {
            Resource::builder()
                .with_service_name("basic-otlp-example-grpc")
                .build()
        })
        .clone()
}

fn init_logs() -> SdkLoggerProvider {
    let exporter = LogExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .build()
        .expect("Failed to create log exporter");

    SdkLoggerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(get_resource())
        .build()
}

fn init_traces() -> SdkTracerProvider {
    let exporter = SpanExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary) //can be changed to `Protocol::HttpJson` to export in JSON format
        .build()
        .expect("Failed to create trace exporter");

    SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(get_resource())
        .build()
}

fn init_metrics() -> SdkMeterProvider {
    let exporter = MetricExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary) //can be changed to `Protocol::HttpJson` to export in JSON format
        .build()
        .expect("Failed to create metric exporter");

    SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .with_resource(get_resource())
        .build()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let logger_provider = init_logs();

    let otel_log_appender = OpenTelemetryLogBridge::new(&logger_provider);
    log::set_boxed_logger(Box::new(otel_log_appender)).unwrap();
    log::set_max_level(Level::Info.to_level_filter());

    let tracer_provider = init_traces();
    // Set the global tracer provider using a clone of the tracer_provider.
    // Setting global tracer provider is required if other parts of the application
    // uses global::tracer() or global::tracer_with_version() to get a tracer.
    // Cloning simply creates a new reference to the same tracer provider. It is
    // important to hold on to the tracer_provider here, so as to invoke
    // shutdown on it when application ends.
    global::set_tracer_provider(tracer_provider.clone());

    let meter_provider = init_metrics();
    // Set the global meter provider using a clone of the meter_provider.
    // Setting global meter provider is required if other parts of the application
    // uses global::meter() or global::meter_with_version() to get a meter.
    // Cloning simply creates a new reference to the same meter provider. It is
    // important to hold on to the meter_provider here, so as to invoke
    // shutdown on it when application ends.
    global::set_meter_provider(meter_provider.clone());

    let common_scope_attributes = vec![KeyValue::new("scope-key", "scope-value")];
    let scope = InstrumentationScope::builder("basic")
        .with_version("1.0")
        .with_attributes(common_scope_attributes)
        .build();

    let tracer = global::tracer_with_scope(scope.clone());
    let meter = global::meter_with_scope(scope);

    let counter = meter
        .u64_counter("test_counter")
        .with_description("a simple counter for demo purposes.")
        .with_unit("my_unit")
        .build();
    for _ in 0..10 {
        counter.add(1, &[KeyValue::new("test_key", "test_value")]);
    }
    counter.add(1, &[KeyValue::new("test_key", "test_value")]);

    tracer.in_span("Main operation", |cx| {
        let span = cx.span();
        span.add_event(
            "Nice operation!".to_string(),
            vec![KeyValue::new("some.key", 100)],
        );
        span.set_attribute(KeyValue::new("another.key", "yes"));

        info!(target: "my-target", "hello from {}. My price is {}. I am also inside a Span!", "banana", 2.99);

        tracer.in_span("Sub operation...", |cx| {
            let span = cx.span();
            span.set_attribute(KeyValue::new("another.key", "yes"));
            span.add_event("Sub span event", vec![]);
        });
    });

    info!(target: "my-target", "hello from {}. My price is {}", "apple", 1.99);

    // Collect all shutdown errors
    let mut shutdown_errors = Vec::new();
    if let Err(e) = tracer_provider.shutdown() {
        shutdown_errors.push(format!("tracer provider: {e}"));
    }

    if let Err(e) = meter_provider.shutdown() {
        shutdown_errors.push(format!("meter provider: {e}"));
    }

    if let Err(e) = logger_provider.shutdown() {
        shutdown_errors.push(format!("logger provider: {e}"));
    }

    // Return an error if any shutdown failed
    if !shutdown_errors.is_empty() {
        return Err(format!(
            "Failed to shutdown providers:{}",
            shutdown_errors.join("\n")
        )
        .into());
    }
    Ok(())
}
