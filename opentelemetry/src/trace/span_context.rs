use crate::trace::TraceState;
use crate::{SpanId, TraceFlags, TraceId};
use std::hash::Hash;

/// Immutable portion of a [`Span`] which can be serialized and propagated.
///
/// This representation conforms to the [W3C TraceContext specification].
///
/// Spans that do not have the `sampled` flag set in their [`TraceFlags`] will
/// be ignored by most tracing tools.
///
/// [`Span`]: crate::trace::Span
/// [W3C TraceContext specification]: https://www.w3.org/TR/trace-context
#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub struct SpanContext {
    trace_id: TraceId,
    span_id: SpanId,
    trace_flags: TraceFlags,
    is_remote: bool,
    trace_state: TraceState,
}

impl SpanContext {
    /// An invalid span context
    pub const NONE: SpanContext = SpanContext {
        trace_id: TraceId::INVALID,
        span_id: SpanId::INVALID,
        trace_flags: TraceFlags::NOT_SAMPLED,
        is_remote: false,
        trace_state: TraceState::NONE,
    };

    /// Create an invalid empty span context
    pub fn empty_context() -> Self {
        SpanContext::NONE
    }

    /// Construct a new `SpanContext`
    pub fn new(
        trace_id: TraceId,
        span_id: SpanId,
        trace_flags: TraceFlags,
        is_remote: bool,
        trace_state: TraceState,
    ) -> Self {
        SpanContext {
            trace_id,
            span_id,
            trace_flags,
            is_remote,
            trace_state,
        }
    }

    /// The [`TraceId`] for this span context.
    pub fn trace_id(&self) -> TraceId {
        self.trace_id
    }

    /// The [`SpanId`] for this span context.
    pub fn span_id(&self) -> SpanId {
        self.span_id
    }

    /// Returns details about the trace.
    ///
    /// Unlike `TraceState` values, these are present in all traces. The current
    /// version of the specification only supports a single flag [`TraceFlags::SAMPLED`].
    pub fn trace_flags(&self) -> TraceFlags {
        self.trace_flags
    }

    /// Returns `true` if the span context has a valid (non-zero) `trace_id` and a
    /// valid (non-zero) `span_id`.
    pub fn is_valid(&self) -> bool {
        self.trace_id != TraceId::INVALID && self.span_id != SpanId::INVALID
    }

    /// Returns `true` if the span context was propagated from a remote parent.
    pub fn is_remote(&self) -> bool {
        self.is_remote
    }

    /// Returns `true` if the `sampled` trace flag is set.
    ///
    /// Spans that are not sampled will be ignored by most tracing tools.
    pub fn is_sampled(&self) -> bool {
        self.trace_flags.is_sampled()
    }

    /// A reference to the span context's [`TraceState`].
    pub fn trace_state(&self) -> &TraceState {
        &self.trace_state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{trace::TraceContextExt, Context};

    #[test]
    fn test_context_span_debug() {
        let cx = Context::current();
        assert_eq!(
            format!("{cx:?}"),
            "Context { span: \"None\", entries count: 0, suppress_telemetry: false }"
        );
        let cx = Context::current().with_remote_span_context(SpanContext::NONE);
        assert_eq!(
            format!("{cx:?}"),
            "Context { \
               span: SpanContext { \
                       trace_id: 00000000000000000000000000000000, \
                       span_id: 0000000000000000, \
                       trace_flags: TraceFlags(0), \
                       is_remote: false, \
                       trace_state: TraceState([]) \
                     }, \
               entries count: 1, suppress_telemetry: false \
             }"
        );
    }
}
