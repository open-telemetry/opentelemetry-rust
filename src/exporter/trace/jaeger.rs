use crate::api;
use std::str::FromStr;

pub use rustracing::{
    sampler::{AllSampler, Sampler},
    tag::{Tag, TagValue},
};
pub use rustracing_jaeger::{reporter::JaegerCompactReporter, span::SpanContext, Span, Tracer};

impl From<api::SpanContext> for rustracing_jaeger::span::SpanContext {
    fn from(context: api::SpanContext) -> Self {
        let parent_id = 0; // TODO
        let jaeger_trace_str = format!(
            "{:x}:{:x}:{:x}:{:x}",
            context.trace_id(),
            context.span_id(),
            parent_id,
            context.trace_flags()
        );
        let span_context_state =
            rustracing_jaeger::span::SpanContextState::from_str(&jaeger_trace_str)
                .expect("should always parse");

        rustracing::span::SpanContext::new(span_context_state, Vec::new())
    }
}
