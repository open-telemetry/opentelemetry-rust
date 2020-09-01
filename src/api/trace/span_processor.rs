//! # OpenTelemetry Span Processor Interface
//!
//! Span processor is an interface which allows hooks for span start and end method
//! invocations. The span processors are invoked only when
//! [`is_recording`] is true.
//!
//! Built-in span processors are responsible for batching and conversion of spans to
//! exportable representation and passing batches to exporters.
//!
//! Span processors can be registered directly on SDK [`TracerProvider`] and they are
//! invoked in the same order as they were registered.
//!
//! All `Tracer` instances created by a `TracerProvider` share the same span processors.
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
//! [`is_recording`]: ../span/trait.Span.html#method.is_recording
//! [`TracerProvider`]: ../provider/trait.TracerProvider.html

use crate::exporter;
use std::sync::Arc;

/// `SpanProcessor`s allow finished spans to be processed.
pub trait SpanProcessor: Send + Sync + std::fmt::Debug {
    /// `on_start` method is invoked when a `Span` is started.
    fn on_start(&self, span: Arc<exporter::trace::SpanData>);
    /// `on_end` method is invoked when a `Span` is ended.
    fn on_end(&self, span: Arc<exporter::trace::SpanData>);
    /// Shutdown is invoked when SDK shuts down. Use this call to cleanup any
    /// processor data. No calls to `on_start` and `on_end` method is invoked
    /// after `shutdown` call is made.
    fn shutdown(&self);
}
