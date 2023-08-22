use crate::export::trace::{ExportResult, SpanData, SpanExporter};
use futures_util::future::BoxFuture;
use opentelemetry_api::trace::{TraceError, TraceResult};
use std::sync::{Arc, Mutex};

///
/// ```
///# use opentelemetry::sdk::trace::{BatchSpanProcessor, TracerProvider};
///# use opentelemetry::testing::span::InMemorySpanExporterBuilder;
///# use opentelemetry::trace::{SpanKind, TraceContextExt};
///# use opentelemetry::{global, sdk::propagation::TraceContextPropagator, trace::Tracer};
///# use opentelemetry::{runtime, Context};
///
///# #[tokio::main]
///# async fn main() {
///     global::set_text_map_propagator(TraceContextPropagator::new());
///     let exporter = InMemorySpanExporterBuilder::new().build();
///     let provider = TracerProvider::builder()
///         .with_span_processor(BatchSpanProcessor::builder(exporter.clone(), runtime::Tokio).build())
///         .build();
///
///     global::set_tracer_provider(provider.clone());
///
///     let tracer = global::tracer("example/in_memory_exporter");
///     let span = tracer
///         .span_builder("say hello")
///         .with_kind(SpanKind::Server)
///         .start(&tracer);
///
///     let cx = Context::current_with_span(span);
///     cx.span().add_event("handling this...", Vec::new());
///     cx.span().end();
///
///     let results = provider.force_flush();
///     for result in results {
///         if let Err(e) = result {
///             println!("{:?}", e)
///         }
///     }
///     let spans = exporter.get_finished_spans().unwrap();
///     for span in spans {
///         println!("{:?}", span)
///     }
///# }
/// ```
#[derive(Clone, Debug)]
pub struct InMemorySpanExporter {
    spans: Arc<Mutex<Vec<SpanData>>>,
}
#[derive(Clone, Debug)]
pub struct InMemorySpanExporterBuilder {}

impl InMemorySpanExporterBuilder {
    pub fn new() -> Self {
        Self {}
    }
    pub fn build(&self) -> InMemorySpanExporter {
        InMemorySpanExporter {
            spans: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl InMemorySpanExporter {
    pub fn new() -> Self {
        InMemorySpanExporterBuilder::new().build()
    }

    pub fn get_finished_spans(&self) -> TraceResult<Vec<SpanData>> {
        self.spans
            .lock()
            .map(|spans_guard| spans_guard.iter().map(Self::clone_span).collect())
            .map_err(TraceError::from)
    }

    pub fn reset(&self) {
        let _ = self.spans.lock().map(|mut spans_guard| spans_guard.clear());
    }

    fn clone_span(span: &SpanData) -> SpanData {
        span.clone()
    }
}

impl SpanExporter for InMemorySpanExporter {
    fn export(&mut self, batch: Vec<SpanData>) -> BoxFuture<'static, ExportResult> {
        if let Err(err) = self
            .spans
            .lock()
            .map(|mut spans_guard| spans_guard.append(&mut batch.clone()))
            .map_err(TraceError::from)
        {
            return Box::pin(std::future::ready(Err(Into::into(err))));
        }
        Box::pin(std::future::ready(Ok(())))
    }

    fn shutdown(&mut self) {
        self.reset()
    }
}
