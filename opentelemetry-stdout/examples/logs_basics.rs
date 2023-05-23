//! run with `$ cargo run --example basic --all-features

#[cfg(all(feature = "logs", feature = "trace"))]
use opentelemetry_api::{
    logs::LoggerProvider as _,
    logs::LogRecordBuilder,
    logs::Logger,
    logs::Severity,
    trace::{Span, Tracer, TracerProvider as _, mark_span_as_active},
    Context, KeyValue,
};
#[cfg(all(feature = "logs", feature = "trace"))]
use opentelemetry_sdk::{
    logs::LoggerProvider,
    runtime,
    trace::TracerProvider,
};

#[cfg(all(feature = "logs", feature = "trace"))]
fn init_trace() -> TracerProvider {
    let exporter = opentelemetry_stdout::SpanExporter::default();
    TracerProvider::builder()
        .with_simple_exporter(exporter)
        .build()
}

#[cfg(all(feature = "logs", feature = "trace"))]
fn init_logger() -> LoggerProvider {
    let exporter = opentelemetry_stdout::LogExporter::default();
    LoggerProvider::builder()
        .with_simple_exporter(exporter)
        .build()
}

#[tokio::main]
#[cfg(all(feature = "logs", feature = "trace"))]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use opentelemetry_api::logs::LogRecordBuilder;

    let logger_provider = init_logger();
    let tracer_provider = init_trace();
    let cx = Context::new();

    let tracer = tracer_provider.tracer("stdout-test");
    let logger = logger_provider.versioned_logger("stdout-test", None, None, None, true);
    let mut span = tracer.start("test_span");
    span.set_attribute(KeyValue::new("test_key", "test_value"));
    let span_active = mark_span_as_active(span);
    let log_record = LogRecordBuilder::new()
                        .with_body("test log".into())
                        .with_severity_number(Severity::Info)
                        .build();
    logger.emit(log_record);
    drop(span_active);
    Ok(())
}
#[cfg(not(all(feature = "logs", feature = "trace")))]
fn main() {}