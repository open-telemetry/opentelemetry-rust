//! # No-op OpenTelemetry Trace Implementation
//!
//! This implementation is returned as the global tracer if no `Tracer`
//! has been set. It is also useful for testing purposes as it is intended
//! to have minimal resource utilization and runtime impact.
use crate::api;
use std::any::Any;
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
            span_context: api::SpanContext::new(0, 0, 0, false),
        }
    }
}

impl api::Span for NoopSpan {
    /// Ignores all events
    fn add_event(&mut self, _message: String) {
        // Ignore
    }

    /// Ignores all events with timestamps
    fn add_event_with_timestamp(&mut self, _message: String, _timestamp: SystemTime) {
        // Ignored
    }

    /// Ignores all links
    fn add_link(&mut self, _link: api::SpanContext) {
        // Ignored
    }

    /// Returns an invalid `SpanContext`.
    fn get_context(&self) -> api::SpanContext {
        self.span_context.clone()
    }

    /// Returns false, signifying that this span is never recording.
    fn is_recording(&self) -> bool {
        false
    }

    /// Ignores all attributes
    fn set_attribute(&mut self, _attribute: api::KeyValue) {
        // Ignored
    }

    /// Ignores status
    fn set_status(&mut self, _status: String) {
        // Ignored
    }

    /// Ignors name updates
    fn update_name(&mut self, _new_name: String) {
        // Ignored
    }

    /// Ignores `Span` endings.
    fn end(&mut self) {
        // Ignored
    }

    /// Returns self as dyn Any
    fn as_any(&self) -> &dyn Any {
        self
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
    fn start(&self, _name: &str, _context: Option<api::SpanContext>) -> Self::Span {
        api::NoopSpan::new()
    }

    /// Returns a new `NoopSpan` as this tracer does not maintain a registry.
    fn get_active_span(&self) -> Self::Span {
        api::NoopSpan::new()
    }

    /// Ignores active span state.
    fn mark_span_as_active(&self, _span: &Self::Span) {
        // Noop
    }

    /// Ignores active span state.
    fn mark_span_as_inactive(&self, _span_id: u64) {
        // Noop
    }
}
