use opentelemetry::{
    global::{self},
    trace::{Span, TraceError, Tracer},
    InstrumentationScope, KeyValue,
};
use opentelemetry_sdk::{trace::SdkTracerProvider, Resource};
use opentelemetry_zipkin::ZipkinExporter;
use std::thread;
use std::time::Duration;

fn bar() {
    let tracer = global::tracer("component-bar");
    let mut span = tracer.start("bar");
    thread::sleep(Duration::from_millis(6));
    span.end()
}

fn init_traces() -> Result<SdkTracerProvider, TraceError> {
    let exporter = ZipkinExporter::builder().build()?;

    Ok(SdkTracerProvider::builder()
        .with_simple_exporter(exporter)
        .with_resource(
            Resource::builder_empty()
                .with_service_name("trace-demo")
                .build(),
        )
        .build())
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let provider = init_traces()?;
    global::set_tracer_provider(provider.clone());

    let common_scope_attributes = vec![KeyValue::new("scope-key", "scope-value")];
    let scope = InstrumentationScope::builder("opentelemetry-zipkin")
        .with_version(env!("CARGO_PKG_VERSION"))
        .with_attributes(common_scope_attributes)
        .build();
    let tracer = global::tracer_with_scope(scope.clone());

    tracer.in_span("foo", |_cx| {
        thread::sleep(Duration::from_millis(6));
        bar();
        thread::sleep(Duration::from_millis(6));
    });

    provider.shutdown()?;
    Ok(())
}
