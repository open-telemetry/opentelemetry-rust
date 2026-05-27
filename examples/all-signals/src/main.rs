use opentelemetry::global;
use opentelemetry::trace::{TraceContextExt, Tracer};
use opentelemetry::KeyValue;
use opentelemetry_otlp::SpanExporter;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;
use std::sync::OnceLock;
use std::time::Duration;

fn get_resource() -> Resource {
    static RESOURCE: OnceLock<Resource> = OnceLock::new();
    RESOURCE
        .get_or_init(|| Resource::builder().with_service_name("DemoApp").build())
        .clone()
}

fn init_traces() -> SdkTracerProvider {
    let exporter = SpanExporter::builder()
        .with_tonic()
        .build()
        .expect("Failed to create span exporter");
    SdkTracerProvider::builder()
        .with_resource(get_resource())
        .with_batch_exporter(exporter)
        .build()
}

#[tokio::main]
async fn main() {
    let tracer_provider = init_traces();
    global::set_tracer_provider(tracer_provider.clone());
    let tracer = global::tracer("my-application");

    tracer.in_span("Main operation", |cx| {
        let span = cx.span();
        span.set_attribute(KeyValue::new("operation.name", "demo"));

        // Simulate some work
        std::thread::sleep(Duration::from_millis(200));

        tracer.in_span("Sub operation", |cx| {
            let span = cx.span();
            span.set_attribute(KeyValue::new("operation.type", "processing"));

            // Simulate sub-operation work
            std::thread::sleep(Duration::from_millis(50));

            span.add_event("Processing completed", vec![]);
        });
    });

    tracer_provider
        .shutdown()
        .expect("Failed to shutdown tracer provider");
}
