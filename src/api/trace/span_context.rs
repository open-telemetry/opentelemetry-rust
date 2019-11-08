// A `SpanContext` represents the portion of a `Span` which must be serialized and propagated along
// side of a distributed context. `SpanContext`s are immutable. `SpanContext` MUST be a final
// (sealed) class.
//
// The OpenTelemetry `SpanContext` representation conforms to the [w3c TraceContext specification].
// It contains two identifiers - a `TraceId` and a `SpanId` - along with a set of common
// `TraceFlags` and system-specific `TraceState` values.
//
// The spec can be viewed here: https://github.com/open-telemetry/opentelemetry-specification/blob/master/specification/api-tracing.md#spancontext
//
// [w3c TraceContext specification]: https://www.w3.org/TR/trace-context/
#[derive(Clone, Debug)]
pub struct SpanContext {
    trace_id: u128,
    span_id: u64,
    trace_flags: u8,
}

impl SpanContext {
    pub fn new(trace_id: u128, span_id: u64, trace_flags: u8) -> Self {
        SpanContext {
            trace_id,
            span_id,
            trace_flags,
        }
    }

    // `TraceId` A valid trace identifier is a 16-byte array with at least one non-zero byte.
    pub fn trace_id(&self) -> u128 {
        self.trace_id
    }

    // `SpanId` A valid span identifier is an 8-byte array with at least one non-zero byte.
    pub fn span_id(&self) -> u64 {
        self.span_id
    }

    // `TraceFlags` contain details about the trace. Unlike `Tracestate` values, `TraceFlags` are
    // present in all traces. Currently, the only TraceFlags is a boolean sampled flag.
    pub fn trace_flags(&self) -> u8 {
        self.trace_flags
    }

    // `IsValid` is a boolean flag which returns true if the `SpanContext` has a non-zero `TraceID`
    // and a non-zero `SpanID`.
    pub fn is_valid(&self) -> bool {
        self.trace_id > 0 && self.span_id > 0
    }

    // `IsRemote` is a boolean flag which returns true if the `SpanContext` was propagated from a
    // remote parent.
    pub fn is_remote(&self) -> bool {
        false // TODO handle remote state
    }
}
