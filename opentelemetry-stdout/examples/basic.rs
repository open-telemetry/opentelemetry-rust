//! run with `$ cargo run --example basic --all-features

#[cfg(all(feature = "metrics", feature = "trace"))]
use opentelemetry::{
    metrics::MeterProvider as _,
    trace::{Span, Tracer, TracerProvider as _},
    KeyValue,
};
#[cfg(all(feature = "metrics", feature = "trace"))]
use opentelemetry_sdk::{
    metrics::{PeriodicReader, SdkMeterProvider},
    runtime,
    trace::TracerProvider,
};

#[cfg(all(feature = "metrics", feature = "trace"))]
fn init_trace() -> TracerProvider {
    let exporter = opentelemetry_stdout::SpanExporter::default();
    TracerProvider::builder()
        .with_simple_exporter(exporter)
        .build()
}

#[cfg(all(feature = "metrics", feature = "trace"))]
fn init_metrics() -> SdkMeterProvider {
    let exporter = opentelemetry_stdout::MetricsExporter::default();
    let reader = PeriodicReader::builder(exporter, runtime::Tokio).build();
    SdkMeterProvider::builder().with_reader(reader).build()
}

#[tokio::main]
#[cfg(all(feature = "metrics", feature = "trace"))]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tracer_provider = init_trace();
    let meter_provider = init_metrics();

    let tracer = tracer_provider.tracer("stdout-test");
    let mut span = tracer.start("test_span");
    span.set_attribute(KeyValue::new("test_key", "test_value"));
    span.add_event(
        "test_event",
        vec![KeyValue::new("test_event_key", "test_event_value")],
    );
    span.end();

    let meter = meter_provider.meter("stdout-test");
    let c = meter.u64_counter("test_events").init();
    c.add(1, &[KeyValue::new("test_key", "test_value")]);

    meter_provider.shutdown()?;

    Ok(())
}
#[cfg(not(all(feature = "metrics", feature = "trace")))]
fn main() {}
