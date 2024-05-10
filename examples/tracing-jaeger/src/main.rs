use opentelemetry::global::shutdown_tracer_provider;
use opentelemetry::{
    global,
    trace::{TraceContextExt, TraceError, Tracer},
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{runtime, trace as sdktrace, Resource};
use opentelemetry_semantic_conventions::resource::SERVICE_NAME;

use std::error::Error;

fn init_tracer() -> Result<opentelemetry_sdk::trace::Tracer, TraceError> {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .with_trace_config(
            sdktrace::config().with_resource(Resource::new(vec![KeyValue::new(
                SERVICE_NAME,
                "tracing-jaeger",
            )])),
        )
        .install_batch(runtime::Tokio)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let _tracer = init_tracer().expect("Failed to initialize tracer.");

    let tracer = global::tracer("tracing-jaeger");
    tracer.in_span("main-operation", |cx| {
        let span = cx.span();
        span.set_attribute(KeyValue::new("my-attribute", "my-value"));
        span.add_event(
            "Main span event".to_string(),
            vec![KeyValue::new("foo", "1")],
        );
        tracer.in_span("child-operation...", |cx| {
            let span = cx.span();
            span.add_event("Sub span event", vec![KeyValue::new("bar", "1")]);
        });
    });

    shutdown_tracer_provider();
    Ok(())
}
