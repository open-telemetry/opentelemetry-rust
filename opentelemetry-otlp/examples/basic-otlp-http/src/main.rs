use once_cell::sync::Lazy;
use opentelemetry::{
    global,
    metrics::{MetricsError, Unit},
    trace::{TraceContextExt, TraceError, Tracer, TracerProvider as _},
    Key, KeyValue,
};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace as sdktrace;
use opentelemetry_sdk::{
    logs::{self as sdklogs, Config},
    Resource,
};
use tracing::info;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

use std::error::Error;

static RESOURCE: Lazy<Resource> = Lazy::new(|| {
    Resource::new(vec![KeyValue::new(
        opentelemetry_semantic_conventions::resource::SERVICE_NAME,
        "basic-otlp-example",
    )])
});

fn init_logs() -> Result<sdklogs::LoggerProvider, opentelemetry::logs::LogError> {
    opentelemetry_otlp::new_pipeline()
        .logging()
        .with_log_config(Config::default().with_resource(RESOURCE.clone()))
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint("http://localhost:4318/v1/logs"),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
}

fn init_tracer() -> Result<sdktrace::Tracer, TraceError> {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint("http://localhost:4318/v1/traces"),
        )
        .with_trace_config(sdktrace::config().with_resource(RESOURCE.clone()))
        .install_batch(opentelemetry_sdk::runtime::Tokio)
}

fn init_metrics() -> Result<opentelemetry_sdk::metrics::SdkMeterProvider, MetricsError> {
    let provider = opentelemetry_otlp::new_pipeline()
        .metrics(opentelemetry_sdk::runtime::Tokio)
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint("http://localhost:4318/v1/metrics"),
        )
        .with_resource(RESOURCE.clone())
        .build();
    match provider {
        Ok(provider) => Ok(provider),
        Err(err) => Err(err),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let result = init_tracer();
    assert!(
        result.is_ok(),
        "Init tracer failed with error: {:?}",
        result.err()
    );

    let result = init_metrics();
    assert!(
        result.is_ok(),
        "Init metrics failed with error: {:?}",
        result.err()
    );

    let meter_provider = result.unwrap();

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
    let tracer = global::tracer_provider()
        .tracer_builder("basic")
        .with_attributes(common_scope_attributes.clone())
        .build();
    let meter = global::meter_with_version(
        "basic",
        Some("v1.0"),
        Some("schema_url"),
        Some(common_scope_attributes.clone()),
    );

    let counter = meter
        .u64_counter("test_counter")
        .with_description("a simple counter for demo purposes.")
        .with_unit(Unit::new("my_unit"))
        .init();
    for _ in 0..10 {
        counter.add(1, &[KeyValue::new("test_key", "test_value")]);
    }
    counter.add(1, &[KeyValue::new("test_key", "test_value")]);

    tracer.in_span("Main operation", |cx| {
        let span = cx.span();
        span.add_event(
            "Nice operation!".to_string(),
            vec![Key::new("bogons").i64(100)],
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
