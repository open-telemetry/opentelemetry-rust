use crate::api;
use std::time::SystemTime;

pub struct NoopProvider {}

impl api::Provider for NoopProvider {
    type Tracer = NoopTracer;

    fn get_tracer<S: Into<String>>(&self, _name: S) -> Self::Tracer {
        NoopTracer {}
    }
}

#[derive(Debug)]
pub struct NoopSpan {
    span_context: api::SpanContext,
}

impl NoopSpan {
    pub fn new() -> Self {
        NoopSpan {
            span_context: api::SpanContext::new(0, 0, 0),
        }
    }
}

impl api::Span for NoopSpan {
    fn id(&self) -> u64 {
        0
    }

    fn parent(&self) -> Option<u64> {
        None
    }

    fn add_event(&mut self, _message: String) {
        // Ignore
    }
    fn add_event_with_timestamp(&mut self, _message: String, _timestamp: SystemTime) {
        // Ignored
    }

    fn get_context(&self) -> api::SpanContext {
        self.span_context.clone()
    }

    fn is_recording(&self) -> bool {
        false
    }

    fn set_attribute(&mut self, _attribute: crate::KeyValue) {
        // Ignored
    }

    fn end(&mut self) {
        // Ignored
    }
}

pub struct NoopTracer {}

impl api::Tracer for NoopTracer {
    type Span = api::NoopSpan;

    fn invalid(&self) -> Self::Span {
        api::NoopSpan::new()
    }

    fn start(&self, _name: String, _context: Option<api::SpanContext>) -> Self::Span {
        api::NoopSpan::new()
    }

    fn get_active_span(&self) -> Self::Span {
        api::NoopSpan::new()
    }

    fn get_span_by_id(&self, _span_id: u64) -> Self::Span {
        api::NoopSpan::new()
    }

    fn mark_span_as_active(&self, _span_id: u64) {
        // Noop
    }
}
