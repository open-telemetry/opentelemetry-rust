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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "trace")]
    init_trace();

    #[cfg(feature = "metrics")]
    init_metrics();

    #[cfg(feature = "trace")]
    emit_span();

    #[cfg(feature = "metrics")]
    emit_metrics();

    #[cfg(feature = "trace")]
    global::shutdown_tracer_provider();

    #[cfg(feature = "metrics")]
    global::shutdown_meter_provider();

    Ok(())
}
