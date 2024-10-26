use once_cell::sync::Lazy;
use opentelemetry::logs::LogError;
use opentelemetry::metrics::MetricError;
use opentelemetry::trace::{TraceContextExt, TraceError, Tracer};
use opentelemetry::KeyValue;
use opentelemetry::{global, InstrumentationScope};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, MetricsExporter, SpanExporter, WithExportConfig};
use opentelemetry_sdk::logs::LoggerProvider;
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::trace::Config;
use opentelemetry_sdk::{runtime, trace as sdktrace, Resource};
use std::error::Error;
use tracing::info;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

static RESOURCE: Lazy<Resource> = Lazy::new(|| {
    Resource::new(vec![KeyValue::new(
        opentelemetry_semantic_conventions::resource::SERVICE_NAME,
        "basic-otlp-example",
    )])
});

fn init_tracer_provider() -> Result<sdktrace::TracerProvider, TraceError> {
    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint("http://localhost:4317")
        .build()?;
    Ok(sdktrace::TracerProvider::builder()
        .with_config(Config::default().with_resource(RESOURCE.clone()))
        .with_batch_exporter(exporter, runtime::Tokio)
        .build())
}

fn init_metrics() -> Result<opentelemetry_sdk::metrics::SdkMeterProvider, MetricError> {
    let exporter = MetricsExporter::builder().with_tonic().build()?;

    let reader = PeriodicReader::builder(exporter, runtime::Tokio).build();

    Ok(SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(RESOURCE.clone())
        .build())
}

fn init_logs() -> Result<opentelemetry_sdk::logs::LoggerProvider, LogError> {
    let exporter = LogExporter::builder()
        .with_tonic()
        .with_endpoint("http://localhost:4317")
        .build()?;

    Ok(LoggerProvider::builder()
        .with_resource(RESOURCE.clone())
        .with_batch_exporter(exporter, runtime::Tokio)
        .build())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    // By binding the result to an unused variable, the lifetime of the variable
    // matches the containing block, reporting traces and metrics during the whole
    // execution.

    let result = init_tracer_provider();
    assert!(
        result.is_ok(),
        "Init tracer failed with error: {:?}",
        result.err()
    );
    let tracer_provider = result.unwrap();
    global::set_tracer_provider(tracer_provider.clone());

    let result = init_metrics();
    assert!(
        result.is_ok(),
        "Init metrics failed with error: {:?}",
        result.err()
    );
    let meter_provider = result.unwrap();
    global::set_meter_provider(meter_provider.clone());

    // Initialize logs and save the logger_provider.
    let logger_provider = init_logs().unwrap();

    // Create a new OpenTelemetryTracingBridge using the above LoggerProvider.
    let layer = OpenTelemetryTracingBridge::new(&logger_provider);

    // Add a tracing filter to filter events from crates used by opentelemetry-otlp.
    // The filter levels are set as follows:
    // - Allow `info` level and above by default.
    // - Restrict `hyper`, `tonic`, and `reqwest` to `error` level logs only.
    // This ensures events generated from these crates within the OTLP Exporter are not looped back,
    // thus preventing infinite event generation.
    // Note: This will also drop events from these crates used outside the OTLP Exporter.
    // For more details, see: https://github.com/open-telemetry/opentelemetry-rust/issues/761
    let filter = EnvFilter::new("info")
        .add_directive("hyper=error".parse().unwrap())
        .add_directive("tonic=error".parse().unwrap())
        .add_directive("reqwest=error".parse().unwrap());

    tracing_subscriber::registry()
        .with(filter)
        .with(layer)
        .init();

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

    tracer.in_span("Main operation", |cx| {
        let span = cx.span();
        span.add_event(
            "Nice operation!".to_string(),
            vec![KeyValue::new("bogons", 100)],
        );
        span.set_attribute(KeyValue::new("another.key", "yes"));

        info!(name: "my-event-inside-span", target: "my-target", "hello from {}. My price is {}. I am also inside a Span!", "banana", 2.99);

        tracer.in_span("Sub operation...", |cx| {
            let span = cx.span();
            span.set_attribute(KeyValue::new("another.key", "yes"));
            span.add_event("Sub span event", vec![]);
        });
    });

    info!(name: "my-event", target: "my-target", "hello from {}. My price is {}", "apple", 1.99);

    global::shutdown_tracer_provider();
    meter_provider.shutdown()?;
    logger_provider.shutdown()?;

    Ok(())
}
