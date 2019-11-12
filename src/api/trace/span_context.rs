//! # OpenTelemetry SpanContext interface
//!
//! A `SpanContext` represents the portion of a `Span` which must be serialized and propagated along
//! side of a distributed context. `SpanContext`s are immutable.
//!
//! The OpenTelemetry `SpanContext` representation conforms to the [w3c TraceContext specification].
//! It contains two identifiers - a `TraceId` and a `SpanId` - along with a set of common
//! `TraceFlags` and system-specific `TraceState` values.
//!
//! The spec can be viewed here: https://github.com/open-telemetry/opentelemetry-specification/blob/master/specification/api-tracing.md#spancontext
//!
//! [w3c TraceContext specification]: https://www.w3.org/TR/trace-context/

const TRACE_FLAGS_BIT_MASK_SAMPLED: u8 = 0x01;
const TRACE_FLAGS_BIT_MASK_UNUSED: u8 = 0xFE;

/// TraceFlagsSampled is a byte with sampled bit set. It is a convenient value initializer
/// for SpanContext TraceFlags field when a trace is sampled.
pub const TRACE_FLAG_SAMPLED: u8 = TRACE_FLAGS_BIT_MASK_SAMPLED;
/// Useful for extracting trace context
pub const TRACE_FLAGS_UNUSED: u8 = TRACE_FLAGS_BIT_MASK_UNUSED;

/// Immutable portion of a `Span` which can be serialized and propagated.
#[derive(Clone, Debug, PartialEq)]
pub struct SpanContext {
    trace_id: u128,
    span_id: u64,
    trace_flags: u8,
    is_remote: bool,
}

impl SpanContext {
    /// Create an invalid empty span context
    pub fn empty_context() -> Self {
        SpanContext::new(0, 0, 0, false)
    }

    /// Construct a new `SpanContext`
    pub fn new(trace_id: u128, span_id: u64, trace_flags: u8, is_remote: bool) -> Self {
        SpanContext {
            trace_id,
            span_id,
            trace_flags,
            is_remote,
        }
    }

    /// A valid trace identifier is a non-zero `u128`.
    pub fn trace_id(&self) -> u128 {
        self.trace_id
    }

    /// A valid span identifier is a non-zero `u64`.
    pub fn span_id(&self) -> u64 {
        self.span_id
    }

    /// Returns details about the trace. Unlike `Tracestate` values, these are
    /// present in all traces. Currently, the only option is a boolean sampled flag.
    pub fn trace_flags(&self) -> u8 {
        self.trace_flags
    }

    /// Returns a bool flag which is true if the `SpanContext` has a valid (non-zero) `trace_id`
    /// and a valid (non-zero) `span_id`.
    pub fn is_valid(&self) -> bool {
        self.trace_id != 0 && self.span_id != 0
    }

    /// Returns true if the `SpanContext` was propagated from a remote parent.
    pub fn is_remote(&self) -> bool {
        self.is_remote
    }

    /// Returns true if the `SpanContext` is sampled.
    pub fn is_sampled(&self) -> bool {
        (self.trace_flags & TRACE_FLAGS_BIT_MASK_SAMPLED) == TRACE_FLAGS_BIT_MASK_SAMPLED
    }
}
