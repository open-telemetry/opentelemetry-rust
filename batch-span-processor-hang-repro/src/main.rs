use futures_util::future;
use opentelemetry::trace::{Span, Tracer, TracerProvider};
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::runtime;
use opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessor;
use opentelemetry_sdk::trace::{SdkTracerProvider, SpanData, SpanExporter};

#[derive(Debug)]
struct ReadyExporter;

impl SpanExporter for ReadyExporter {
    fn export(
        &self,
        _batch: Vec<SpanData>,
    ) -> impl std::future::Future<Output = OTelSdkResult> + Send {
        future::ready(Ok(()))
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Build a provider that uses the async-runtime batch span processor with the Tokio runtime.
    let batch_processor = BatchSpanProcessor::builder(ReadyExporter, runtime::Tokio).build();

    let provider = SdkTracerProvider::builder()
        .with_span_processor(batch_processor)
        .build();

    // End a span so the processor has some work queued.
    provider.tracer("repro").start("blocking-call").end();

    println!("Calling force_flush... this blocks forever");

    // This never returns because force_flush uses futures_executor::block_on,
    // which cannot make progress while we're on Tokio's current-thread scheduler.
    provider.force_flush().expect("force_flush should succeed");

    println!("force_flush returned");
}
