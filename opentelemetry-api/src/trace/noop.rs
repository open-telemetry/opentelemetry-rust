//! No-op trace impls
//!
//! This implementation is returned as the global tracer if no `Tracer`
//! has been set. It is also useful for testing purposes as it is intended
//! to have minimal resource utilization and runtime impact.
use crate::{
    propagation::{text_map_propagator::FieldIter, Extractor, Injector, TextMapPropagator},
    trace,
    trace::{TraceContextExt, TraceFlags, TraceState},
    Context, KeyValue,
};
use std::borrow::Cow;
use std::time::SystemTime;

/// A no-op instance of a `TracerProvider`.
#[derive(Clone, Debug, Default)]
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
    fn versioned_tracer(
        &self,
        _name: impl Into<Cow<'static, str>>,
        _version: Option<&'static str>,
        _schema_url: Option<&'static str>,
    ) -> Self::Tracer {
        NoopTracer::new()
    }
}

/// A no-op instance of a `Span`.
#[derive(Clone, Debug)]
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
                trace::TraceId::INVALID,
                trace::SpanId::INVALID,
                TraceFlags::default(),
                false,
                TraceState::default(),
            ),
        }
    }
}

impl trace::Span for NoopSpan {
    /// Ignores all events
    fn add_event<T>(&mut self, _name: T, _attributes: Vec<KeyValue>)
    where
        T: Into<Cow<'static, str>>,
    {
        // Ignore
    }

    /// Ignores all events with timestamps
    fn add_event_with_timestamp<T>(
        &mut self,
        _name: T,
        _timestamp: SystemTime,
        _attributes: Vec<KeyValue>,
    ) where
        T: Into<Cow<'static, str>>,
    {
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
    fn set_attribute(&mut self, _attribute: KeyValue) {
        // Ignored
    }

    /// Ignores status
    fn set_status(&mut self, _status: trace::Status) {
        // Ignored
    }

    /// Ignores name updates
    fn update_name<T>(&mut self, _new_name: T)
    where
        T: Into<Cow<'static, str>>,
    {
        // Ignored
    }

    /// Ignores `Span` endings
    fn end_with_timestamp(&mut self, _timestamp: SystemTime) {
        // Ignored
    }
}

/// A no-op instance of a `Tracer`.
#[derive(Clone, Debug, Default)]
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
    type Span = NoopSpan;

    /// Builds a `NoopSpan` from a `SpanBuilder`.
    ///
    /// If the span builder or the context's current span contains a valid span context, it is
    /// propagated.
    fn build_with_context(&self, _builder: trace::SpanBuilder, parent_cx: &Context) -> Self::Span {
        if parent_cx.has_active_span() {
            NoopSpan {
                span_context: parent_cx.span().span_context().clone(),
            }
        } else {
            NoopSpan::new()
        }
    }
}

/// A no-op instance of an [`TextMapPropagator`].
///
/// [`TextMapPropagator`]: crate::propagation::TextMapPropagator
#[derive(Debug, Default)]
pub struct NoopTextMapPropagator {
    _private: (),
}

impl NoopTextMapPropagator {
    /// Create a new noop text map propagator
    pub fn new() -> Self {
        NoopTextMapPropagator { _private: () }
    }
}

impl TextMapPropagator for NoopTextMapPropagator {
    fn inject_context(&self, _cx: &Context, _injector: &mut dyn Injector) {
        // ignored
    }

    fn extract_with_context(&self, _cx: &Context, _extractor: &dyn Extractor) -> Context {
        Context::current()
    }

    fn fields(&self) -> FieldIter<'_> {
        FieldIter::new(&[])
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
            trace::TraceFlags::default(),
            true,
            TraceState::default(),
        )
    }

    #[test]
    fn noop_tracer_defaults_to_invalid_span() {
        let tracer = NoopTracer::new();
        let span = tracer.start_with_context("foo", &Context::new());
        assert!(!span.span_context().is_valid());
    }

    #[test]
    fn noop_tracer_propagates_valid_span_context_from_builder() {
        let tracer = NoopTracer::new();
        let builder = tracer.span_builder("foo");
        let span = tracer.build_with_context(
            builder,
            &Context::new().with_span(TestSpan(valid_span_context())),
        );
        assert!(span.span_context().is_valid());
    }

    #[test]
    fn noop_tracer_propagates_valid_span_context_from_explicitly_specified_context() {
        let tracer = NoopTracer::new();
        let cx = Context::new().with_span(NoopSpan {
            span_context: valid_span_context(),
        });
        let span = tracer.start_with_context("foo", &cx);
        assert!(span.span_context().is_valid());
    }

    #[test]
    fn noop_tracer_propagates_valid_span_context_from_remote_span_context() {
        let tracer = NoopTracer::new();
        let cx = Context::new().with_remote_span_context(valid_span_context());
        let span = tracer.start_with_context("foo", &cx);
        assert!(span.span_context().is_valid());
    }
}
