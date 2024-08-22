//! run with `$ cargo run --example basic

use once_cell::sync::Lazy;
use opentelemetry::{global, KeyValue};

#[cfg(feature = "trace")]
use opentelemetry::trace::{Span, Tracer};

#[cfg(feature = "metrics")]
use opentelemetry_sdk::runtime;

#[cfg(feature = "metrics")]
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};

use opentelemetry_sdk::trace::Config;
#[cfg(feature = "trace")]
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_sdk::Resource;

static RESOURCE: Lazy<Resource> = Lazy::new(|| {
    Resource::default().merge(&Resource::new(vec![KeyValue::new(
        opentelemetry_semantic_conventions::resource::SERVICE_NAME,
        "basic-stdout-example",
    )]))
});

#[cfg(feature = "trace")]
fn init_trace() {
    let exporter = opentelemetry_stdout::SpanExporter::default();
    let provider = TracerProvider::builder()
        .with_simple_exporter(exporter)
        .with_config(Config::default().with_resource(RESOURCE.clone()))
        .build();
    global::set_tracer_provider(provider);
}

#[cfg(feature = "metrics")]
fn init_metrics() -> opentelemetry_sdk::metrics::SdkMeterProvider {
    let exporter = opentelemetry_stdout::MetricsExporter::default();
    let reader = PeriodicReader::builder(exporter, runtime::Tokio).build();
    let provider = SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(RESOURCE.clone())
        .build();
    global::set_meter_provider(provider.clone());
    provider
}

#[cfg(feature = "logs")]
fn init_logs() -> opentelemetry_sdk::logs::LoggerProvider {
    use opentelemetry_appender_tracing::layer;
    use opentelemetry_sdk::logs::LoggerProvider;
    use tracing_subscriber::prelude::*;

    let exporter = opentelemetry_stdout::LogExporter::default();
    let provider: LoggerProvider = LoggerProvider::builder()
        .with_simple_exporter(exporter)
        .with_resource(RESOURCE.clone())
        .build();
    let layer = layer::OpenTelemetryTracingBridge::new(&provider);
    tracing_subscriber::registry().with(layer).init();
    provider
}

#[cfg(feature = "trace")]
fn emit_span() {
    use opentelemetry::trace::{
        SpanContext, SpanId, TraceFlags, TraceId, TraceState, TracerProvider,
    };

    let tracer = global::tracer_provider()
        .tracer_builder("stdout-example")
        .with_version("v1")
        .with_schema_url("schema_url")
        .with_attributes(vec![KeyValue::new("scope_key", "scope_value")])
        .build();
    let mut span = tracer.start("example-span");
    span.set_attribute(KeyValue::new("attribute_key1", "attribute_value1"));
    span.set_attribute(KeyValue::new("attribute_key2", "attribute_value2"));
    span.add_event(
        "example-event-name",
        vec![KeyValue::new("event_attribute1", "event_value1")],
    );
    span.add_link(
        SpanContext::new(
            TraceId::from_hex("58406520a006649127e371903a2de979").expect("invalid"),
            SpanId::from_hex("b6d7d7f6d7d6d7f6").expect("invalid"),
            TraceFlags::default(),
            false,
            TraceState::NONE,
        ),
        vec![
            KeyValue::new("link_attribute1", "link_value1"),
            KeyValue::new("link_attribute2", "link_value2"),
        ],
    );

    span.add_link(
        SpanContext::new(
            TraceId::from_hex("23401120a001249127e371903f2de971").expect("invalid"),
            SpanId::from_hex("cd37d765d743d7f6").expect("invalid"),
            TraceFlags::default(),
            false,
            TraceState::NONE,
        ),
        vec![
            KeyValue::new("link_attribute1", "link_value1"),
            KeyValue::new("link_attribute2", "link_value2"),
        ],
    );
    span.end();
}

#[cfg(feature = "metrics")]
fn emit_metrics() {
    let meter = global::meter("stdout-example");
    let c = meter.u64_counter("example_counter").init();
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
    global::shutdown_tracer_provider();

    #[cfg(feature = "metrics")]
    meter_provider.shutdown()?;

    #[cfg(feature = "logs")]
    logger_provider.shutdown()?;

    Ok(())
}
