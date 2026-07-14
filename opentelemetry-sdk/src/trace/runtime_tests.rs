// Note that all tests here should be marked as ignore so that it won't be picked up by default We
// need to run those tests one by one as the GlobalTracerProvider is a shared object between
// threads Use cargo test -- --ignored --test-threads=1 to run those tests.
#[cfg(feature = "rt-tokio")]
use crate::runtime::RuntimeChannel;
#[cfg(feature = "rt-tokio")]
use crate::trace::SpanExporter;
#[cfg(feature = "rt-tokio")]
use crate::{error::OTelSdkResult, runtime};
#[cfg(feature = "rt-tokio")]
use opentelemetry::global::*;
#[cfg(feature = "rt-tokio")]
use opentelemetry::trace::Tracer;
#[cfg(feature = "rt-tokio")]
use std::fmt::Debug;
#[cfg(feature = "rt-tokio")]
use std::sync::atomic::{AtomicUsize, Ordering};
#[cfg(feature = "rt-tokio")]
use std::sync::Arc;
#[derive(Debug)]
#[cfg(feature = "rt-tokio")]
struct SpanCountExporter {
    span_count: Arc<AtomicUsize>,
}

#[cfg(feature = "rt-tokio")]
impl SpanExporter for SpanCountExporter {
    async fn export(&self, batch: Vec<crate::trace::SpanData>) -> OTelSdkResult {
        self.span_count.fetch_add(batch.len(), Ordering::SeqCst);
        Ok(())
    }
}

#[cfg(feature = "rt-tokio")]
impl SpanCountExporter {
    fn new() -> SpanCountExporter {
        SpanCountExporter {
            span_count: Arc::new(AtomicUsize::new(0)),
        }
    }
}

#[cfg(feature = "rt-tokio")]
fn build_batch_tracer_provider<R: RuntimeChannel>(
    exporter: SpanCountExporter,
    runtime: R,
) -> crate::trace::SdkTracerProvider {
    use crate::trace::SdkTracerProvider;
    let processor = crate::trace::span_processor_with_async_runtime::BatchSpanProcessor::builder(
        exporter, runtime,
    )
    .build();
    SdkTracerProvider::builder()
        .with_span_processor(processor)
        .build()
}

#[cfg(feature = "rt-tokio")]
fn build_simple_tracer_provider(exporter: SpanCountExporter) -> crate::trace::SdkTracerProvider {
    use crate::trace::SdkTracerProvider;
    SdkTracerProvider::builder()
        .with_simple_exporter(exporter)
        .build()
}

#[cfg(feature = "rt-tokio")]
async fn test_set_provider_in_tokio<R: RuntimeChannel>(
    runtime: R,
) -> (Arc<AtomicUsize>, crate::trace::SdkTracerProvider) {
    let exporter = SpanCountExporter::new();
    let span_count = exporter.span_count.clone();
    let tracer_provider = build_batch_tracer_provider(exporter, runtime);
    set_tracer_provider(tracer_provider.clone());
    let tracer = tracer("opentelemetery");

    tracer.in_span("test", |_cx| {});

    (span_count, tracer_provider)
}

// When using `tokio::spawn` to spawn the worker task in batch processor
//
// multiple -> no shut down -> not export
// multiple -> shut down -> export
// single -> no shutdown -> not export
// single -> shutdown -> hang forever

// When using |fut| tokio::task::spawn_blocking(|| futures::executor::block_on(fut))
// to spawn the worker task in batch processor
//
// multiple -> no shutdown -> hang forever
// multiple -> shut down -> export
// single -> shut down -> export
// single -> no shutdown -> hang forever

// Test if the multiple thread tokio runtime could exit successfully when not force flushing spans
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires --test-threads=1"]
#[cfg(feature = "rt-tokio")]
async fn test_set_provider_multiple_thread_tokio() {
    let (span_count, _) = test_set_provider_in_tokio(runtime::Tokio).await;
    assert_eq!(span_count.load(Ordering::SeqCst), 0);
}

// Test if the multiple thread tokio runtime could exit successfully when force flushing spans
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires --test-threads=1"]
#[cfg(feature = "rt-tokio")]
async fn test_set_provider_multiple_thread_tokio_shutdown() {
    let (span_count, tracer_provider) = test_set_provider_in_tokio(runtime::Tokio).await;
    tracer_provider
        .shutdown()
        .expect("TracerProvider should shutdown properly");
    assert!(span_count.load(Ordering::SeqCst) > 0);
}

// Test use simple processor in single thread tokio runtime.
// Expected to see the spans being exported to buffer
#[tokio::test]
#[ignore = "requires --test-threads=1"]
#[cfg(feature = "rt-tokio")]
async fn test_set_provider_single_thread_tokio_with_simple_processor() {
    let exporter = SpanCountExporter::new();
    let span_count = exporter.span_count.clone();
    let tracer_provider = build_simple_tracer_provider(exporter);
    set_tracer_provider(tracer_provider.clone());
    let tracer = tracer("opentelemetry");

    tracer.in_span("test", |_cx| {});

    tracer_provider
        .shutdown()
        .expect("TracerProvider should shutdown properly");

    assert!(span_count.load(Ordering::SeqCst) > 0);
}

// Test if the single thread tokio runtime could exit successfully when not force flushing spans.
#[tokio::test]
#[ignore = "requires --test-threads=1"]
#[cfg(feature = "rt-tokio")]
async fn test_set_provider_single_thread_tokio() {
    let (span_count, _) = test_set_provider_in_tokio(runtime::Tokio).await;
    assert_eq!(span_count.load(Ordering::SeqCst), 0)
}

// Test if the single thread tokio runtime could exit successfully when force flushing spans.
#[tokio::test]
#[ignore = "requires --test-threads=1"]
#[cfg(feature = "rt-tokio")]
async fn test_set_provider_single_thread_tokio_shutdown() {
    let (span_count, tracer_provider) = test_set_provider_in_tokio(runtime::Tokio).await;
    tracer_provider
        .shutdown()
        .expect("TracerProvider should shutdown properly");
    assert!(span_count.load(Ordering::SeqCst) > 0)
}
