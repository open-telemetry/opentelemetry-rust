//! # No-op OpenTelemetry Trace Implementation
//!
//! This implementation is returned as the global tracer if no `Tracer`
//! has been set. It is also useful for testing purposes as it is intended
//! to have minimal resource utilization and runtime impact.
use crate::api::trace::TraceContextExt;
use crate::api::trace::TraceState;
use crate::{
    api,
    exporter::trace::{ExportResult, SpanData, SpanExporter},
};
use async_trait::async_trait;
use std::time::SystemTime;

/// A no-op instance of a `TracerProvider`.
#[derive(Debug, Default)]
pub struct NoopTracerProvider {
    _private: (),
}

impl NoopTracerProvider {
    /// Create a new no-op tracer provider
    pub fn new() -> Self {
        NoopTracerProvider { _private: () }
    }
}

impl api::trace::TracerProvider for NoopTracerProvider {
    type Tracer = NoopTracer;

    /// Returns a new `NoopTracer` instance.
    fn get_tracer(&self, _name: &'static str, _version: Option<&'static str>) -> Self::Tracer {
        NoopTracer::new()
    }
}

/// A no-op instance of a `Span`.
#[derive(Debug)]
pub struct NoopSpan {
    span_context: api::trace::SpanReference,
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
            span_context: api::trace::SpanReference::new(
                api::trace::TraceId::invalid(),
                api::trace::SpanId::invalid(),
                0,
                false,
                TraceState::default(),
            ),
        }
    }
}

impl api::trace::Span for NoopSpan {
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

    /// Returns an invalid `SpanReference`.
    fn span_context(&self) -> api::trace::SpanReference {
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
    fn set_status(&self, _code: api::trace::StatusCode, _message: String) {
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
#[derive(Debug, Default)]
pub struct NoopTracer {
    _private: (),
}

impl NoopTracer {
    /// Create a new no-op tracer
    pub fn new() -> Self {
        NoopTracer { _private: () }
    }
}

impl api::trace::Tracer for NoopTracer {
    type Span = api::trace::NoopSpan;

    /// Returns a `NoopSpan` as they are always invalid.
    fn invalid(&self) -> Self::Span {
        api::trace::NoopSpan::new()
    }

    /// Starts a new `NoopSpan` in a given context.
    ///
    /// If the context contains a valid span context, it is progagated.
    fn start_from_context(&self, name: &str, cx: &api::Context) -> Self::Span {
        let builder = self.span_builder(name);
        self.build_with_context(builder, cx)
    }

    /// Starts a `SpanBuilder`.
    fn span_builder(&self, name: &str) -> api::trace::SpanBuilder {
        api::trace::SpanBuilder::from_name(name.to_string())
    }

    /// Builds a `NoopSpan` from a `SpanBuilder`.
    ///
    /// If the span builder or context contains a valid span context, it is progagated.
    fn build_with_context(
        &self,
        mut builder: api::trace::SpanBuilder,
        cx: &api::Context,
    ) -> Self::Span {
        let parent_span_context = builder
            .parent_context
            .take()
            .or_else(|| Some(cx.span().span_context()).filter(|cx| cx.is_valid()))
            .or_else(|| cx.remote_span_context().cloned())
            .filter(|cx| cx.is_valid());
        if let Some(span_context) = parent_span_context {
            api::trace::NoopSpan { span_context }
        } else {
            self.invalid()
        }
    }
}

/// A no-op instance of an [`SpanExporter`].
///
/// [`SpanExporter`]: ../../../exporter/trace/trait.SpanExporter.html
#[derive(Debug, Default)]
pub struct NoopSpanExporter {
    _private: (),
}

impl NoopSpanExporter {
    /// Create a new noop span exporter
    pub fn new() -> Self {
        NoopSpanExporter { _private: () }
    }
}

#[async_trait]
impl SpanExporter for NoopSpanExporter {
    async fn export(&self, _batch: Vec<SpanData>) -> ExportResult {
        ExportResult::Success
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::trace::{Span, TraceState, Tracer};

    fn valid_span_context() -> api::trace::SpanReference {
        api::trace::SpanReference::new(
            api::trace::TraceId::from_u128(42),
            api::trace::SpanId::from_u64(42),
            0,
            true,
            TraceState::default(),
        )
    }

    #[test]
    fn noop_tracer_defaults_to_invalid_span() {
        let tracer = NoopTracer::new();
        let span = tracer.start_from_context("foo", &api::Context::new());
        assert!(!span.span_context().is_valid());
    }

    #[test]
    fn noop_tracer_propagates_valid_span_context_from_builder() {
        let tracer = NoopTracer::new();
        let builder = tracer.span_builder("foo").with_parent(valid_span_context());
        let span = tracer.build_with_context(builder, &api::Context::new());
        assert!(span.span_context().is_valid());
    }

    #[test]
    fn noop_tracer_propagates_valid_span_context_from_span() {
        let tracer = NoopTracer::new();
        let cx = api::Context::new().with_span(NoopSpan {
            span_context: valid_span_context(),
        });
        let span = tracer.start_from_context("foo", &cx);
        assert!(span.span_context().is_valid());
    }

    #[test]
    fn noop_tracer_propagates_valid_span_context_from_remote_span_context() {
        let tracer = NoopTracer::new();
        let cx = api::Context::new().with_remote_span_context(valid_span_context());
        let span = tracer.start_from_context("foo", &cx);
        assert!(span.span_context().is_valid());
    }
}
