//! # Span Processor
//!
//! ### Span processor
//!
//! Span processor is an interface which allows hooks for span start and end method
//! invocations. The span processors are invoked only when
//! [`is_recording`] is true.
//!
//! Built-in span processors are responsible for batching and conversion of spans to
//! exportable representation and passing batches to exporters.
//!
//! Span processors can be registered directly on SDK [`Provider`] and they are
//! invoked in the same order as they were registered.
//!
//! All `Tracer` instances created by a `Provider` share the same span processors.
//! Changes to this collection reflect in all `Tracer` instances.
//!
//! The following diagram shows `SpanProcessor`'s relationship to other components
//! in the SDK:
//!
//! ```ascii
//!   +-----+--------------+   +-----------------------+   +-------------------+
//!   |     |              |   |                       |   |                   |
//!   |     |              |   | (Batch)SpanProcessor  |   |    SpanExporter   |
//!   |     |              +---> (Simple)SpanProcessor +--->  (JaegerExporter) |
//!   |     |              |   |                       |   |                   |
//!   | SDK | Tracer.span()|   +-----------------------+   +-------------------+
//!   |     | Span.end()   |
//!   |     |              |   +---------------------+
//!   |     |              |   |                     |
//!   |     |              +---> ZPagesProcessor     |
//!   |     |              |   |                     |
//!   +-----+--------------+   +---------------------+
//! ```
//!
//! [`is_recording`]: trait.Span.html#is_recording
//! [`Provider`]: trait.Provider.html
use crate::{api, exporter};
use std::sync::Arc;

/// `SimpleSpanProcessor` is used by exporters to receive SpanData
/// synchronously when span is finished.
#[derive(Debug)]
pub struct SimpleSpanProcessor {
    exporter: Box<dyn exporter::trace::SpanExporter>,
}

impl SimpleSpanProcessor {
    pub(crate) fn new(exporter: Box<dyn exporter::trace::SpanExporter>) -> Self {
        SimpleSpanProcessor { exporter }
    }
}

impl api::SpanProcessor for SimpleSpanProcessor {
    fn on_start(&self, _span: Arc<exporter::trace::SpanData>) {
        // Ignored
    }

    fn on_end(&self, span: Arc<exporter::trace::SpanData>) {
        if span.context.is_sampled() {
            self.exporter.export(vec![span]);
        }
    }

    fn shutdown(&self) {
        // Ignored
    }
}
