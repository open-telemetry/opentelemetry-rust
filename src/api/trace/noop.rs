//! # No-op OpenTelemetry Trace Implementation
//!
//! This implementation is returned as the global tracer if no `Tracer`
//! has been set. It is also useful for testing purposes as it is intended
//! to have minimal resource utilization and runtime impact.
use crate::api::trace::span_context::TraceState;
use crate::api::TraceContextExt;
use crate::{api, exporter};
use std::sync::Arc;
use std::time::SystemTime;

/// A no-op instance of a `TracerProvider`.
#[derive(Debug)]
pub struct NoopProvider {}

impl api::TracerProvider for NoopProvider {
    type Tracer = NoopTracer;

    /// Returns a new `NoopTracer` instance.
    fn get_tracer(&self, _name: &'static str, _version: Option<&'static str>) -> Self::Tracer {
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
                TraceState::default(),
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

    /// Starts a new `NoopSpan` in a given context.
    ///
    /// If the context contains a valid span context, it is progagated.
    fn start_from_context(&self, name: &str, cx: &api::Context) -> Self::Span {
        let builder = self.span_builder(name);
        self.build_with_context(builder, cx)
    }

    /// Starts a `SpanBuilder`.
    fn span_builder(&self, name: &str) -> api::SpanBuilder {
        api::SpanBuilder::from_name(name.to_string())
    }

    /// Builds a `NoopSpan` from a `SpanBuilder`.
    ///
    /// If the span builder or context contains a valid span context, it is progagated.
    fn build_with_context(&self, mut builder: api::SpanBuilder, cx: &api::Context) -> Self::Span {
        let parent_span_context = builder
            .parent_context
            .take()
            .or_else(|| Some(cx.span().span_context()).filter(|cx| cx.is_valid()))
            .or_else(|| cx.remote_span_context().cloned())
            .filter(|cx| cx.is_valid());
        if let Some(span_context) = parent_span_context {
            api::NoopSpan { span_context }
        } else {
            self.invalid()
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::trace::span_context::TraceState;
    use crate::api::{Span, Tracer};

    fn valid_span_context() -> api::SpanContext {
        api::SpanContext::new(
            api::TraceId::from_u128(42),
            api::SpanId::from_u64(42),
            0,
            true,
            TraceState::default(),
        )
    }

    #[test]
    fn noop_tracer_defaults_to_invalid_span() {
        let tracer = NoopTracer {};
        let span = tracer.start_from_context("foo", &api::Context::new());
        assert!(!span.span_context().is_valid());
    }

    #[test]
    fn noop_tracer_propagates_valid_span_context_from_builder() {
        let tracer = NoopTracer {};
        let builder = tracer.span_builder("foo").with_parent(valid_span_context());
        let span = tracer.build_with_context(builder, &api::Context::new());
        assert!(span.span_context().is_valid());
    }

    #[test]
    fn noop_tracer_propagates_valid_span_context_from_span() {
        let tracer = NoopTracer {};
        let cx = api::Context::new().with_span(NoopSpan {
            span_context: valid_span_context(),
        });
        let span = tracer.start_from_context("foo", &cx);
        assert!(span.span_context().is_valid());
    }

    #[test]
    fn noop_tracer_propagates_valid_span_context_from_remote_span_context() {
        let tracer = NoopTracer {};
        let cx = api::Context::new().with_remote_span_context(valid_span_context());
        let span = tracer.start_from_context("foo", &cx);
        assert!(span.span_context().is_valid());
    }
}
