//! run with `$ cargo run --example basic

use opentelemetry::{global, KeyValue};

#[cfg(feature = "trace")]
use opentelemetry::trace::{Span, Tracer};

#[cfg(feature = "metrics")]
use opentelemetry_sdk::runtime;

#[cfg(feature = "metrics")]
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};

#[cfg(feature = "trace")]
use opentelemetry_sdk::trace::TracerProvider;

#[cfg(feature = "trace")]
fn init_trace() {
    let exporter = opentelemetry_stdout::SpanExporter::default();
    let provider = TracerProvider::builder()
        .with_simple_exporter(exporter)
        .build();
    global::set_tracer_provider(provider);
}

#[cfg(feature = "metrics")]
fn init_metrics() {
    let exporter = opentelemetry_stdout::MetricsExporter::default();
    let reader = PeriodicReader::builder(exporter, runtime::Tokio).build();
    let provider = SdkMeterProvider::builder().with_reader(reader).build();
    global::set_meter_provider(provider);
}

#[cfg(feature = "logs")]
fn init_logs() -> opentelemetry_sdk::logs::LoggerProvider {
    use opentelemetry_appender_tracing::layer;
    use opentelemetry_sdk::logs::LoggerProvider;
    use tracing_subscriber::prelude::*;

    let exporter = opentelemetry_stdout::LogExporter::default();
    let provider: LoggerProvider = LoggerProvider::builder()
        .with_simple_exporter(exporter)
        .build();
    let layer = layer::OpenTelemetryTracingBridge::new(&provider);
    tracing_subscriber::registry().with(layer).init();
    provider
}

#[cfg(feature = "trace")]
fn emit_span() {
    let tracer = global::tracer("stdout-test");
    let mut span = tracer.start("test_span");
    span.set_attribute(KeyValue::new("test_key", "test_value"));
    span.add_event(
        "test_event",
        vec![KeyValue::new("test_event_key", "test_event_value")],
    );
    span.end();
}

#[cfg(feature = "metrics")]
fn emit_metrics() {
    let meter = global::meter("stdout-test");
    let c = meter.u64_counter("test_counter").init();
    c.add(1, &[KeyValue::new("test_key", "test_value")]);
}

#[cfg(feature = "logs")]
fn emit_log() {
    use tracing::error;
    error!(name: "my-event-name", target: "my-system", event_id = 20, user_name = "otel", user_email = "otel@opentelemetry.io");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "trace")]
    init_trace();

    #[cfg(feature = "metrics")]
    init_metrics();

    #[cfg(feature = "logs")]
    let logger_provider = init_logs();

    #[cfg(feature = "logs")]
    emit_log();

    println!(
        "======================================================================================"
    );

    #[cfg(feature = "trace")]
    emit_span();

    println!(
        "======================================================================================"
    );

    #[cfg(feature = "metrics")]
    emit_metrics();

    #[cfg(feature = "trace")]
    global::shutdown_tracer_provider();

    #[cfg(feature = "metrics")]
    global::shutdown_meter_provider();

    #[cfg(feature = "logs")]
    drop(logger_provider);

    Ok(())
}
