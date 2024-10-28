use once_cell::sync::Lazy;
use opentelemetry::{
    global,
    metrics::MetricError,
    trace::{TraceContextExt, TraceError, Tracer},
    InstrumentationScope, KeyValue,
};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_otlp::{LogExporter, MetricExporter, Protocol, SpanExporter};
use opentelemetry_sdk::{
    logs::LoggerProvider,
    metrics::{PeriodicReader, SdkMeterProvider},
    runtime,
    trace::{self as sdktrace, Config, TracerProvider},
};
use opentelemetry_sdk::{
    logs::{self as sdklogs},
    Resource,
};
use std::error::Error;
use tracing::info;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

#[cfg(feature = "hyper")]
use opentelemetry_otlp::WithHttpConfig;

#[cfg(feature = "hyper")]
mod hyper;

static RESOURCE: Lazy<Resource> = Lazy::new(|| {
    Resource::new(vec![KeyValue::new(
        opentelemetry_semantic_conventions::resource::SERVICE_NAME,
        "basic-otlp-example",
    )])
});

fn init_logs() -> Result<sdklogs::LoggerProvider, opentelemetry::logs::LogError> {
    let exporter_builder = LogExporter::builder()
        .with_http()
        .with_endpoint("http://localhost:4318/v1/logs")
        .with_protocol(Protocol::HttpBinary);

    #[cfg(feature = "hyper")]
    let exporter_builder = exporter_builder.with_http_client(hyper::HyperClient::default());

    let exporter = exporter_builder.build()?;

    Ok(LoggerProvider::builder()
        .with_batch_exporter(exporter, runtime::Tokio)
        .with_resource(RESOURCE.clone())
        .build())
}

fn init_tracer_provider() -> Result<sdktrace::TracerProvider, TraceError> {
    let exporter = SpanExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary) //can be changed to `Protocol::HttpJson` to export in JSON format
        .with_endpoint("http://localhost:4318/v1/traces")
        .build()?;
    Ok(TracerProvider::builder()
        .with_batch_exporter(exporter, runtime::Tokio)
        .with_config(Config::default().with_resource(RESOURCE.clone()))
        .build())
}

fn init_metrics() -> Result<opentelemetry_sdk::metrics::SdkMeterProvider, MetricError> {
    let exporter = MetricExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary) //can be changed to `Protocol::HttpJson` to export in JSON format
        .with_endpoint("http://localhost:4318/v1/metrics")
        .build()?;

    Ok(SdkMeterProvider::builder()
        .with_reader(PeriodicReader::builder(exporter, runtime::Tokio).build())
        .with_resource(RESOURCE.clone())
        .build())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
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

    // Opentelemetry will not provide a global API to manage the logger
    // provider. Application users must manage the lifecycle of the logger
    // provider on their own. Dropping logger providers will disable log
    // emitting.
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
    counter.add(1, &[KeyValue::new("test_key", "test_value")]);

    tracer.in_span("Main operation", |cx| {
        let span = cx.span();
        span.add_event(
            "Nice operation!".to_string(),
            vec![KeyValue::new("bogons", 100)],
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

    global::shutdown_tracer_provider();
    logger_provider.shutdown()?;
    meter_provider.shutdown()?;

    Ok(())
}
