//! # No-op OpenTelemetry Trace Implementation
//!
//! This implementation is returned as the global tracer if no `Tracer`
//! has been set. It is also useful for testing purposes as it is intended
//! to have minimal resource utilization and runtime impact.
use crate::{api, exporter};
use std::sync::Arc;
use std::time::SystemTime;

/// A no-op instance of a `Provider`.
#[derive(Debug)]
pub struct NoopProvider {}

impl api::Provider for NoopProvider {
    type Tracer = NoopTracer;

    /// Returns a new `NoopTracer` instance.
    fn get_tracer(&self, _name: &'static str) -> Self::Tracer {
        NoopTracer {}
    }
}

/// A no-op instance of a `Span`.
#[derive(Debug)]
pub struct NoopSpan {
    span_context: api::SpanContext,
}

impl Default for NoopSpan {
    fn default() -> Self {
        NoopSpan::new()
    }
}

impl NoopSpan {
    /// Creates a new `NoopSpan` instance.
    pub fn new() -> Self {
        NoopSpan {
            span_context: api::SpanContext::new(
                api::TraceId::invalid(),
                api::SpanId::invalid(),
                0,
                false,
            ),
        }
    }
}

impl api::Span for NoopSpan {
    /// Ignores all events
    fn add_event(&self, _name: String, _attributes: Vec<api::KeyValue>) {
        // Ignore
    }

    /// Ignores all events with timestamps
    fn add_event_with_timestamp(
        &self,
        _name: String,
        _timestamp: SystemTime,
        _attributes: Vec<api::KeyValue>,
    ) {
        // Ignored
    }

    /// Returns an invalid `SpanContext`.
    fn span_context(&self) -> api::SpanContext {
        self.span_context.clone()
    }

    /// Returns false, signifying that this span is never recording.
    fn is_recording(&self) -> bool {
        false
    }

    /// Ignores all attributes
    fn set_attribute(&self, _attribute: api::KeyValue) {
        // Ignored
    }

    /// Ignores status
    fn set_status(&self, _code: api::StatusCode, _message: String) {
        // Ignored
    }

    /// Ignores name updates
    fn update_name(&self, _new_name: String) {
        // Ignored
    }

    /// Ignores `Span` endings.
    fn end(&self) {
        // Ignored
    }

    /// Ignores `Span` endings
    fn end_with_timestamp(&self, _timestamp: SystemTime) {
        // Ignored
    }
}

/// A no-op instance of a `Tracer`.
#[derive(Debug)]
pub struct NoopTracer {}

impl api::Tracer for NoopTracer {
    type Span = api::NoopSpan;

    /// Returns a `NoopSpan` as they are always invalid.
    fn invalid(&self) -> Self::Span {
        api::NoopSpan::new()
    }

    /// Starts a new `NoopSpan`.
    fn start_from_context(&self, _name: &str, _context: &api::Context) -> Self::Span {
        self.invalid()
    }

    /// Starts a SpanBuilder
    fn span_builder(&self, name: &str) -> api::SpanBuilder {
        api::SpanBuilder::from_name(name.to_string())
    }

    /// Builds a `NoopSpan` from a `SpanBuilder`
    fn build_with_context(&self, _builder: api::SpanBuilder, _cx: &api::Context) -> Self::Span {
        self.invalid()
    }
}

/// A no-op instance of an [`SpanExporter`].
///
/// [`SpanExporter`]: ../../../exporter/trace/trait.SpanExporter.html
#[derive(Debug)]
pub struct NoopSpanExporter {}

impl exporter::trace::SpanExporter for NoopSpanExporter {
    fn export(&self, _batch: Vec<Arc<exporter::trace::SpanData>>) -> exporter::trace::ExportResult {
        exporter::trace::ExportResult::Success
    }

    fn shutdown(&self) {
        // Noop
    }
}
