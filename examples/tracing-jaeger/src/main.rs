use opentelemetry::{
    global,
    trace::{TraceContextExt, TraceError, Tracer},
    Key, KeyValue,
};
use opentelemetry_sdk::{runtime, trace as sdktrace, Resource};
use opentelemetry_otlp::WithExportConfig;
use std::error::Error;
use opentelemetry::global::shutdown_tracer_provider;

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
                "service.name",
                "tracing-jaeger",
            )])),
        )
        .install_batch(runtime::Tokio)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let _tracer = init_tracer().expect("Failed to initialize tracer.");

    let tracer = global::tracer("ex.com/basic");
    tracer.in_span("operation", |cx| {
        let span = cx.span();
        span.add_event(
            "Nice operation!".to_string(),
            vec![Key::new("bogons").i64(100)],
        );
        tracer.in_span("Sub operation...", |cx| {
            let span = cx.span();
            span.add_event("Sub span event", vec![]);
        });
    });

    shutdown_tracer_provider();
    Ok(())
}
