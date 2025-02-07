//! run with `$ cargo run --example basic

use once_cell::sync::Lazy;
use opentelemetry::{global, KeyValue};

#[cfg(feature = "trace")]
use opentelemetry::trace::Tracer;

#[cfg(feature = "metrics")]
use opentelemetry_sdk::metrics::SdkMeterProvider;

#[cfg(feature = "trace")]
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;

static RESOURCE: Lazy<Resource> = Lazy::new(|| {
    Resource::builder()
        .with_service_name("basic-stdout-example")
        .build()
});

#[cfg(feature = "trace")]
fn init_trace() -> SdkTracerProvider {
    let exporter = opentelemetry_stdout::SpanExporter::default();
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(exporter)
        .with_resource(RESOURCE.clone())
        .build();
    global::set_tracer_provider(provider.clone());
    provider
}

#[cfg(feature = "metrics")]
fn init_metrics() -> opentelemetry_sdk::metrics::SdkMeterProvider {
    let exporter = opentelemetry_stdout::MetricExporter::default();
    let provider = SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .with_resource(RESOURCE.clone())
        .build();
    global::set_meter_provider(provider.clone());
    provider
}

#[cfg(feature = "logs")]
fn init_logs() -> opentelemetry_sdk::logs::SdkLoggerProvider {
    use opentelemetry_appender_tracing::layer;
    use opentelemetry_sdk::logs::SdkLoggerProvider;
    use tracing_subscriber::prelude::*;

    let exporter = opentelemetry_stdout::LogExporter::default();
    let provider: SdkLoggerProvider = SdkLoggerProvider::builder()
        .with_simple_exporter(exporter)
        .with_resource(RESOURCE.clone())
        .build();
    let layer = layer::OpenTelemetryTracingBridge::new(&provider);
    tracing_subscriber::registry().with(layer).init();
    provider
}

#[cfg(feature = "trace")]
fn emit_span() {
    use opentelemetry::{trace::TraceContextExt, InstrumentationScope};

    let scope = InstrumentationScope::builder("stdout-example")
        .with_version("v1")
        .with_attributes([KeyValue::new("scope_key", "scope_value")])
        .build();

    let tracer = global::tracer_with_scope(scope);
    tracer.in_span("example-span", |cx| {
        let span = cx.span();
        span.set_attribute(KeyValue::new("my-attribute", "my-value"));
        span.add_event(
            "example-event-name",
            vec![KeyValue::new("event_attribute1", "event_value1")],
        );
        emit_log();
    })
}

#[cfg(feature = "metrics")]
fn emit_metrics() {
    let meter = global::meter("stdout-example");
    let c = meter.u64_counter("example_counter").build();
    c.add(
        1,
        &[
            KeyValue::new("name", "apple"),
            KeyValue::new("color", "green"),
        ],
    );
    c.add(
        1,
        &[
            KeyValue::new("name", "apple"),
            KeyValue::new("color", "green"),
        ],
    );
    c.add(
        2,
        &[
            KeyValue::new("name", "apple"),
            KeyValue::new("color", "red"),
        ],
    );
    c.add(
        1,
        &[
            KeyValue::new("name", "banana"),
            KeyValue::new("color", "yellow"),
        ],
    );
    c.add(
        11,
        &[
            KeyValue::new("name", "banana"),
            KeyValue::new("color", "yellow"),
        ],
    );

    let h = meter.f64_histogram("example_histogram").build();
    h.record(
        1.0,
        &[
            KeyValue::new("name", "apple"),
            KeyValue::new("color", "green"),
        ],
    );
    h.record(
        1.0,
        &[
            KeyValue::new("name", "apple"),
            KeyValue::new("color", "green"),
        ],
    );
    h.record(
        2.0,
        &[
            KeyValue::new("name", "apple"),
            KeyValue::new("color", "red"),
        ],
    );
    h.record(
        1.0,
        &[
            KeyValue::new("name", "banana"),
            KeyValue::new("color", "yellow"),
        ],
    );
    h.record(
        11.0,
        &[
            KeyValue::new("name", "banana"),
            KeyValue::new("color", "yellow"),
        ],
    );
}

#[cfg(feature = "logs")]
fn emit_log() {
    use tracing::error;
    error!(name: "my-event-name", target: "my-system", event_id = 20, user_name = "otel", user_email = "otel@opentelemetry.io");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "trace")]
    let tracer_provider = init_trace();

    #[cfg(feature = "metrics")]
    let meter_provider = init_metrics();

    #[cfg(feature = "logs")]
    let logger_provider = init_logs();

    #[cfg(feature = "logs")]
    emit_log();

    #[cfg(feature = "trace")]
    emit_span();

    #[cfg(feature = "metrics")]
    emit_metrics();

    #[cfg(feature = "trace")]
    tracer_provider.shutdown()?;

    #[cfg(feature = "metrics")]
    meter_provider.shutdown()?;

    #[cfg(feature = "logs")]
    logger_provider.shutdown()?;

    Ok(())
}
