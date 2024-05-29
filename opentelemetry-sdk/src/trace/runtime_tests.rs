// Note that all tests here should be marked as ignore so that it won't be picked up by default We
// need to run those tests one by one as the GlobalTracerProvider is a shared object between
// threads Use cargo test -- --ignored --test-threads=1 to run those tests.
use crate::export::trace::{ExportResult, SpanExporter};
#[cfg(any(feature = "rt-tokio", feature = "rt-tokio-current-thread"))]
use crate::runtime;
#[cfg(any(feature = "rt-tokio", feature = "rt-tokio-current-thread"))]
use crate::runtime::RuntimeChannel;
use futures_util::future::BoxFuture;
#[cfg(any(feature = "rt-tokio", feature = "rt-tokio-current-thread"))]
use opentelemetry::global::*;
#[cfg(any(feature = "rt-tokio", feature = "rt-tokio-current-thread"))]
use opentelemetry::trace::Tracer;
use std::fmt::Debug;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Debug)]
struct SpanCountExporter {
    span_count: Arc<AtomicUsize>,
}

impl SpanExporter for SpanCountExporter {
    fn export(
        &mut self,
        batch: Vec<crate::export::trace::SpanData>,
    ) -> BoxFuture<'static, ExportResult> {
        self.span_count.fetch_add(batch.len(), Ordering::SeqCst);
        Box::pin(async { Ok(()) })
    }
}

#[cfg(any(feature = "rt-tokio", feature = "rt-tokio-current-thread"))]
impl SpanCountExporter {
    fn new() -> SpanCountExporter {
        SpanCountExporter {
            span_count: Arc::new(AtomicUsize::new(0)),
        }
    }
}

#[cfg(any(feature = "rt-tokio", feature = "rt-tokio-current-thread"))]
fn build_batch_tracer_provider<R: RuntimeChannel>(
    exporter: SpanCountExporter,
    runtime: R,
) -> crate::trace::TracerProvider {
    use crate::trace::TracerProvider;
    TracerProvider::builder()
        .with_batch_exporter(exporter, runtime)
        .build()
}

#[cfg(any(feature = "rt-tokio", feature = "rt-tokio-current-thread"))]
fn build_simple_tracer_provider(exporter: SpanCountExporter) -> crate::trace::TracerProvider {
    use crate::trace::TracerProvider;
    TracerProvider::builder()
        .with_simple_exporter(exporter)
        .build()
}

#[cfg(any(feature = "rt-tokio", feature = "rt-tokio-current-thread"))]
async fn test_set_provider_in_tokio<R: RuntimeChannel>(runtime: R) -> Arc<AtomicUsize> {
    let exporter = SpanCountExporter::new();
    let span_count = exporter.span_count.clone();
    let _ = set_tracer_provider(build_batch_tracer_provider(exporter, runtime));
    let tracer = tracer("opentelemetery");

    tracer.in_span("test", |_cx| {});

    span_count
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
    let span_count = test_set_provider_in_tokio(runtime::Tokio).await;
    assert_eq!(span_count.load(Ordering::SeqCst), 0);
}

// Test if the multiple thread tokio runtime could exit successfully when force flushing spans
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires --test-threads=1"]
#[cfg(feature = "rt-tokio")]
async fn test_set_provider_multiple_thread_tokio_shutdown() {
    let span_count = test_set_provider_in_tokio(runtime::Tokio).await;
    shutdown_tracer_provider();
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
    let _ = set_tracer_provider(build_simple_tracer_provider(exporter));
    let tracer = tracer("opentelemetry");

    tracer.in_span("test", |_cx| {});

    shutdown_tracer_provider();

    assert!(span_count.load(Ordering::SeqCst) > 0);
}

// Test if the single thread tokio runtime could exit successfully when not force flushing spans
#[tokio::test]
#[ignore = "requires --test-threads=1"]
#[cfg(feature = "rt-tokio-current-thread")]
async fn test_set_provider_single_thread_tokio() {
    let span_count = test_set_provider_in_tokio(runtime::TokioCurrentThread).await;
    assert_eq!(span_count.load(Ordering::SeqCst), 0)
}

// Test if the single thread tokio runtime could exit successfully when force flushing spans.
#[tokio::test]
#[ignore = "requires --test-threads=1"]
#[cfg(feature = "rt-tokio-current-thread")]
async fn test_set_provider_single_thread_tokio_shutdown() {
    let span_count = test_set_provider_in_tokio(runtime::TokioCurrentThread).await;
    shutdown_tracer_provider();
    assert!(span_count.load(Ordering::SeqCst) > 0)
}
