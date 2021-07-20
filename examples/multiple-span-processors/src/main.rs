use opentelemetry::global::{self, shutdown_tracer_provider};
use opentelemetry::sdk::export::trace::stdout::Exporter as StdoutExporter;
use opentelemetry::sdk::trace::{BatchSpanProcessor, TracerProvider};
use opentelemetry::trace::{mark_span_as_active, TraceError, Tracer};
use opentelemetry::KeyValue;
use std::io::stdout;
use std::time::Duration;

fn init_tracer() -> Result<(), TraceError> {
    // build a jaeger batch span processor
    let jaeger_processor = BatchSpanProcessor::builder(
        opentelemetry_jaeger::new_pipeline()
            .with_service_name("trace-demo")
            .with_tags(vec![KeyValue::new("exporter", "jaeger")])
            .init_exporter(opentelemetry::runtime::Tokio)?,
        opentelemetry::runtime::Tokio,
    )
    .build();

    // build a zipkin exporter
    let zipkin_exporter = opentelemetry_zipkin::new_pipeline()
        .with_service_name("trace-demo")
        .init_exporter()?;

    let provider = TracerProvider::builder()
        // We can build a span processor and pass it into provider.
        .with_span_processor(jaeger_processor)
        // For batch span processor, we can also provide the exporter and runtime and use this
        // helper function to build a batch span processor
        .with_batch_exporter(zipkin_exporter, opentelemetry::runtime::Tokio)
        // Same helper function is also available to build a simple span processor.
        .with_simple_exporter(StdoutExporter::new(stdout(), true))
        .build();

    let _ = global::set_tracer_provider(provider);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    init_tracer()?;

    let tracer = global::tracer("jaeger-and-zipkin");

    {
        let span = tracer.start("first span");
        let _guard = mark_span_as_active(span);
        {
            let _inner = tracer.start("first sub span");
            tokio::time::sleep(Duration::from_millis(15)).await;
        }
        {
            let _inner = tracer.start("second sub span");
            tokio::time::sleep(Duration::from_millis(15)).await;
        }
    }

    tokio::time::sleep(Duration::from_millis(15)).await;

    shutdown_tracer_provider();

    Ok(())
}
