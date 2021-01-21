//! # No-op OpenTelemetry Trace Implementation
//!
//! This implementation is returned as the global tracer if no `Tracer`
//! has been set. It is also useful for testing purposes as it is intended
//! to have minimal resource utilization and runtime impact.
use crate::{
    sdk::export::trace::{ExportResult, SpanData, SpanExporter},
    trace,
    trace::{TraceContextExt, TraceState},
    Context, KeyValue,
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

impl trace::TracerProvider for NoopTracerProvider {
    type Tracer = NoopTracer;

    /// Returns a new `NoopTracer` instance.
    fn get_tracer(&self, _name: &'static str, _version: Option<&'static str>) -> Self::Tracer {
        NoopTracer::new()
    }
}

/// A no-op instance of a `Span`.
#[derive(Debug)]
pub struct NoopSpan {
    span_context: trace::SpanContext,
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
            span_context: trace::SpanContext::new(
                trace::TraceId::invalid(),
                trace::SpanId::invalid(),
                0,
                false,
                TraceState::default(),
            ),
        }
    }
}

impl trace::Span for NoopSpan {
    /// Ignores all events
    fn add_event(&self, _name: String, _attributes: Vec<KeyValue>) {
        // Ignore
    }

    /// Ignores all events with timestamps
    fn add_event_with_timestamp(
        &self,
        _name: String,
        _timestamp: SystemTime,
        _attributes: Vec<KeyValue>,
    ) {
        // Ignored
    }

    /// Returns an invalid `SpanContext`.
    fn span_context(&self) -> &trace::SpanContext {
        &self.span_context
    }

    /// Returns false, signifying that this span is never recording.
    fn is_recording(&self) -> bool {
        false
    }

    /// Ignores all attributes
    fn set_attribute(&self, _attribute: KeyValue) {
        // Ignored
    }

    /// Ignores status
    fn set_status(&self, _code: trace::StatusCode, _message: String) {
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

impl trace::Tracer for NoopTracer {
    type Span = trace::NoopSpan;

    /// Returns a `NoopSpan` as they are always invalid.
    fn invalid(&self) -> Self::Span {
        trace::NoopSpan::new()
    }

    /// Starts a new `NoopSpan` with a given context.
    ///
    /// If the context contains a valid span context, it is propagated.
    fn start_with_context(&self, name: &str, cx: Context) -> Self::Span {
        let mut builder = self.span_builder(name);
        builder.parent_context = Some(cx);
        self.build(builder)
    }

    /// Starts a `SpanBuilder`.
    fn span_builder(&self, name: &str) -> trace::SpanBuilder {
        trace::SpanBuilder::from_name(name.to_string())
    }

    /// Builds a `NoopSpan` from a `SpanBuilder`.
    ///
    /// If the span builder or context contains a valid span context, it is propagated.
    fn build(&self, mut builder: trace::SpanBuilder) -> Self::Span {
        let parent_span_context = builder.parent_context.take().and_then(|parent_cx| {
            if parent_cx.has_active_span() {
                Some(parent_cx.span().span_context().clone())
            } else {
                parent_cx.remote_span_context().cloned()
            }
        });
        if let Some(span_context) = parent_span_context {
            trace::NoopSpan { span_context }
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
    async fn export(&mut self, _batch: Vec<SpanData>) -> ExportResult {
        Ok(())
    }
}

#[cfg(all(test, feature = "testing", feature = "trace"))]
mod tests {
    use super::*;
    use crate::testing::trace::TestSpan;
    use crate::trace::{self, Span, Tracer};

    fn valid_span_context() -> trace::SpanContext {
        trace::SpanContext::new(
            trace::TraceId::from_u128(42),
            trace::SpanId::from_u64(42),
            0,
            true,
            TraceState::default(),
        )
    }

    #[test]
    fn noop_tracer_defaults_to_invalid_span() {
        let tracer = NoopTracer::new();
        let span = tracer.start_with_context("foo", Context::new());
        assert!(!span.span_context().is_valid());
    }

    #[test]
    fn noop_tracer_propagates_valid_span_context_from_builder() {
        let tracer = NoopTracer::new();
        let builder = tracer
            .span_builder("foo")
            .with_parent_context(Context::current_with_span(TestSpan(valid_span_context())));
        let span = tracer.build(builder);
        assert!(span.span_context().is_valid());
    }

    #[test]
    fn noop_tracer_propagates_valid_span_context_from_span() {
        let tracer = NoopTracer::new();
        let cx = Context::new().with_span(NoopSpan {
            span_context: valid_span_context(),
        });
        let span = tracer.start_with_context("foo", cx);
        assert!(span.span_context().is_valid());
    }

    #[test]
    fn noop_tracer_propagates_valid_span_context_from_remote_span_context() {
        let tracer = NoopTracer::new();
        let cx = Context::new().with_remote_span_context(valid_span_context());
        let span = tracer.start_with_context("foo", cx);
        assert!(span.span_context().is_valid());
    }
}
