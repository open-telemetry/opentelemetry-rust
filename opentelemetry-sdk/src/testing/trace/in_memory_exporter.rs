use crate::export::trace::{ExportResult, SpanData, SpanExporter};
use futures_util::future::BoxFuture;
use opentelemetry::trace::{TraceError, TraceResult};
use std::sync::{Arc, Mutex};

/// An in-memory span exporter that stores span data in memory.
///
/// This exporter is useful for testing and debugging purposes. It stores
/// metric data in a `Vec<SpanData>`. Metrics can be retrieved
/// using the `get_finished_spans` method.
/// # Example
/// ```
///# use opentelemetry::trace::{SpanKind, TraceContextExt};
///# use opentelemetry::{global, trace::Tracer, Context};
///# use opentelemetry_sdk::propagation::TraceContextPropagator;
///# use opentelemetry_sdk::runtime;
///# use opentelemetry_sdk::testing::trace::InMemorySpanExporterBuilder;
///# use opentelemetry_sdk::trace::{BatchSpanProcessor, TracerProvider};
///
///# #[tokio::main]
///# async fn main() {
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

impl Default for InMemorySpanExporter {
    fn default() -> Self {
        InMemorySpanExporterBuilder::new().build()
    }
}

/// Builder for [`InMemorySpanExporter`].
/// # Example
/// ```
///# use opentelemetry_sdk::testing::trace::InMemorySpanExporterBuilder;
///
/// let exporter = InMemorySpanExporterBuilder::new().build();
/// ```
#[derive(Clone, Debug)]
pub struct InMemorySpanExporterBuilder {}

impl Default for InMemorySpanExporterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemorySpanExporterBuilder {
    /// Creates a new instance of the `InMemorySpanExporterBuilder`.
    pub fn new() -> Self {
        Self {}
    }

    /// Creates a new instance of the `InMemorySpanExporter`.
    pub fn build(&self) -> InMemorySpanExporter {
        InMemorySpanExporter {
            spans: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl InMemorySpanExporter {
    /// Returns the finished span as a vector of `SpanData`.
    ///
    /// # Errors
    ///
    /// Returns a `TraceError` if the internal lock cannot be acquired.
    ///
    /// # Example
    ///
    /// ```
    /// # use opentelemetry_sdk::testing::trace::InMemorySpanExporter;
    ///
    /// let exporter = InMemorySpanExporter::default();
    /// let finished_spans = exporter.get_finished_spans().unwrap();
    /// ```
    pub fn get_finished_spans(&self) -> TraceResult<Vec<SpanData>> {
        self.spans
            .lock()
            .map(|spans_guard| spans_guard.iter().cloned().collect())
            .map_err(TraceError::from)
    }

    /// Clears the internal storage of finished spans.
    ///
    /// # Example
    ///
    /// ```
    /// # use opentelemetry_sdk::testing::trace::InMemorySpanExporter;
    ///
    /// let exporter = InMemorySpanExporter::default();
    /// exporter.reset();
    /// ```
    pub fn reset(&self) {
        let _ = self.spans.lock().map(|mut spans_guard| spans_guard.clear());
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
