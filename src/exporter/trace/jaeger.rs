//! # OpenTelemetry Jaeger Exporter
//!
//! This exporter currently delegates to the [rustracing_jaeger library]
//! which implements the [OpenTracing API].
//!
//! [rustracing_jaeger library]: https://github.com/sile/rustracing_jaeger
//! [OpenTracing API]: https://opentracing.io/
use crate::api;
use std::str::FromStr;

pub use rustracing::{
    sampler::{AllSampler, NullSampler, ProbabilisticSampler, Sampler},
    tag::{Tag, TagValue},
};
pub use rustracing_jaeger::{reporter::JaegerCompactReporter, span::SpanContext, Span, Tracer};

impl From<api::SpanContext> for rustracing_jaeger::span::SpanContext {
    /// Convert from `api::SpanContext` instances to `rustracing_jaeger`'s `SpanContext` type.
    fn from(context: api::SpanContext) -> Self {
        let jaeger_trace_str = format!(
            "{:x}:{:x}:0:{:x}",
            context.trace_id(),
            context.span_id(),
            context.trace_flags()
        );
        let span_context_state =
            rustracing_jaeger::span::SpanContextState::from_str(&jaeger_trace_str)
                .expect("should always parse");

        rustracing::span::SpanContext::new(span_context_state, Vec::new())
    }
}
