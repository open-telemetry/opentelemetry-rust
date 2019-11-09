//! # Trace Tracer SDK
//!
//! The OpenTelemetry library achieves in-process context propagation of
//! `Span`s by way of the `Tracer`.
//!
//! The `Tracer` is responsible for tracking the currently active `Span`,
//! and exposes methods for creating and activating new `Spans`.
//!
//! Docs: https://github.com/open-telemetry/opentelemetry-specification/blob/master/specification/api-tracing.md#tracer
use crate::api;
use crate::exporter::trace::jaeger;
use crate::sdk;

/// `Tracer` implementation to create and manage spans
#[derive(Clone, Debug)]
pub struct Tracer(pub(crate) jaeger::Tracer);

impl api::Tracer for Tracer {
    /// This implementation of `api::Tracer` produces `sdk::Span` instances.
    type Span = sdk::Span;

    /// Returns a span with an inactive `SpanContext`. Used by functions that
    /// need to return a default span like `get_active_span` if no span is present.
    fn invalid(&self) -> Self::Span {
        sdk::Span::new(jaeger::Span::inactive())
    }

    /// Starts a new `Span`.
    ///
    /// Each span has zero or one parent spans and zero or more child spans, which
    /// represent causally related operations. A tree of related spans comprises a
    /// trace. A span is said to be a _root span_ if it does not have a parent. Each
    /// trace includes a single root span, which is the shared ancestor of all other
    /// spans in the trace.
    fn start(&self, name: &'static str, parent_span: Option<api::SpanContext>) -> Self::Span {
        let start_options = self.0.span(name);
        let started = match parent_span.map(jaeger::SpanContext::from) {
            Some(span_context) => start_options.child_of(&span_context).start(),
            None => start_options.start(),
        };

        sdk::Span::new(started)
    }

    /// Returns the current active span.
    ///
    /// When getting the current `Span`, the `Tracer` will return a placeholder
    /// `Span` with an invalid `SpanContext` if there is no currently active `Span`.
    fn get_active_span(&self) -> Self::Span {
        // TODO
        unimplemented!()
    }

    /// Mark a given `Span` as active.
    fn mark_span_as_active(&self, _span_id: u64) {
        // TODO
        unimplemented!()
    }
}
