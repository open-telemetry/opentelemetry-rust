use once_cell::sync::Lazy;
use opentelemetry::{
    global, metrics,
    trace::{TraceContextExt, TraceError, Tracer},
    Key, KeyValue,
};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::logs as sdklogs;
use opentelemetry_sdk::metrics as sdkmetrics;
use opentelemetry_sdk::resource;
use opentelemetry_sdk::trace as sdktrace;

use std::error::Error;
use tracing::info;
use tracing_subscriber::prelude::*;

fn init_logs() -> Result<sdklogs::Logger, opentelemetry::logs::LogError> {
    let service_name = std::env::var("CARGO_BIN_NAME")
        .unwrap_or_else(|_| String::from("basic-otlp-http-logging-example"));
    opentelemetry_otlp::new_pipeline()
        .logging()
        .with_log_config(
            sdklogs::Config::default().with_resource(resource::Resource::new(vec![KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                service_name,
            )])),
        )
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint("http://localhost:4318"),
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
        .install_batch(opentelemetry_sdk::runtime::Tokio)
}

fn init_metrics() -> metrics::Result<sdkmetrics::MeterProvider> {
    let export_config = opentelemetry_otlp::ExportConfig {
        endpoint: "http://localhost:4318/v1/metrics".to_string(),
        ..opentelemetry_otlp::ExportConfig::default()
    };
    opentelemetry_otlp::new_pipeline()
        .metrics(opentelemetry_sdk::runtime::Tokio)
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_export_config(export_config),
        )
        .build()
}

const LEMONS_KEY: Key = Key::from_static_str("ex.com/lemons");
const ANOTHER_KEY: Key = Key::from_static_str("ex.com/another");

static COMMON_ATTRIBUTES: Lazy<[KeyValue; 4]> = Lazy::new(|| {
    [
        LEMONS_KEY.i64(10),
        KeyValue::new("A", "1"),
        KeyValue::new("B", "2"),
        KeyValue::new("C", "3"),
    ]
});

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let _ = init_tracer()?;
    let meter_provider = init_metrics()?;
    let _ = init_logs();

    let tracer = global::tracer("ex.com/basic");
    let meter = global::meter("ex.com/basic");

    // configure the global logger to use our opentelemetry logger
    let logger_provider = opentelemetry::global::logger_provider();
    let layer = OpenTelemetryTracingBridge::new(&logger_provider);
    tracing_subscriber::registry().with(layer).init();

    tracer.in_span("operation", |cx| {
        let span = cx.span();
        span.add_event(
            "Nice operation!".to_string(),
            vec![Key::new("bogons").i64(100)],
        );
        span.set_attribute(ANOTHER_KEY.string("yes"));

        tracer.in_span("Sub operation...", |cx| {
            let span = cx.span();
            span.set_attribute(LEMONS_KEY.string("five"));

            span.add_event("Sub span event", vec![]);
        });
        info!(target: "my-target", "hello from {}. My price is {}. I am also inside a Span!", "banana", 2.99);
    });
    info!(target: "my-target", "hello from {}. My price is {}", "apple", 1.99);

    let histogram = meter.f64_histogram("ex.com.two").init();
    histogram.record(5.5, COMMON_ATTRIBUTES.as_ref());

    global::shutdown_tracer_provider();
    global::shutdown_logger_provider();
    meter_provider.shutdown()?;

    Ok(())
}
