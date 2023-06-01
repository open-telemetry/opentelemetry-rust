//! # W3C Trace Context HTTP Response Propagator
//!
//! The traceresponse HTTP response header field identifies a completed request
//! in a tracing system. It has four fields:
//!
//!    - version
//!    - trace-id
//!    - parent-id
//!    - trace-flags
//!
//! See the [w3c trace-context docs] for more details.
//!
//! [w3c trace-context docs]: https://w3c.github.io/trace-context/#traceresponse-header
use once_cell::sync::Lazy;
use opentelemetry::trace::{SpanContext, SpanId, TraceId, TraceState};
use opentelemetry_api::{
    propagation::{text_map_propagator::FieldIter, Extractor, Injector, TextMapPropagator},
    trace::{TraceContextExt, TraceFlags},
    Context,
};

const SUPPORTED_VERSION: u8 = 0;
const MAX_VERSION: u8 = 254;
const TRACERESPONSE_HEADER: &str = "traceresponse";

static TRACE_CONTEXT_HEADER_FIELDS: Lazy<[String; 1]> =
    Lazy::new(|| [TRACERESPONSE_HEADER.to_owned()]);

/// Propagates trace response using the [W3C TraceContext] format
///
/// [W3C TraceContext]: https://w3c.github.io/trace-context/#traceresponse-header
#[derive(Clone, Debug, Default)]
pub struct TraceContextResponsePropagator {
    _private: (),
}

impl TraceContextResponsePropagator {
    /// Create a new `TraceContextPropagator`.
    pub fn new() -> Self {
        TraceContextResponsePropagator { _private: () }
    }

    /// Extract span context from w3c trace-context header.
    fn extract_span_context(&self, extractor: &dyn Extractor) -> Result<SpanContext, ()> {
        let header_value = extractor.get(TRACERESPONSE_HEADER).unwrap_or("").trim();
        let parts = header_value.split_terminator('-').collect::<Vec<&str>>();
        // Ensure parts are not out of range.
        if parts.len() < 4 {
            return Err(());
        }

        // Ensure version is within range, for version 0 there must be 4 parts.
        let version = u8::from_str_radix(parts[0], 16).map_err(|_| ())?;
        if version > MAX_VERSION || version == 0 && parts.len() != 4 {
            return Err(());
        }

        // Ensure trace id is lowercase
        if parts[1].chars().any(|c| c.is_ascii_uppercase()) {
            return Err(());
        }

        // Parse trace id section
        let trace_id = TraceId::from_hex(parts[1]).map_err(|_| ())?;

        // Ensure span id is lowercase
        if parts[2].chars().any(|c| c.is_ascii_uppercase()) {
            return Err(());
        }

        // Parse span id section
        let span_id = SpanId::from_hex(parts[2]).map_err(|_| ())?;

        // Parse trace flags section
        let opts = u8::from_str_radix(parts[3], 16).map_err(|_| ())?;

        // Ensure opts are valid for version 0
        if version == 0 && opts > 2 {
            return Err(());
        }

        // Build trace flags clearing all flags other than the trace-context
        // supported sampling bit.
        let trace_flags = TraceFlags::new(opts) & TraceFlags::SAMPLED;

        // create context
        let span_context =
            SpanContext::new(trace_id, span_id, trace_flags, true, TraceState::default());

        // Ensure span is valid
        if !span_context.is_valid() {
            return Err(());
        }

        Ok(span_context)
    }
}

impl TextMapPropagator for TraceContextResponsePropagator {
    /// Properly encodes the values of the `SpanContext` and injects them
    /// into the `Injector`.
    fn inject_context(&self, cx: &Context, injector: &mut dyn Injector) {
        let span = match cx.span() {
            Some(span) => span,
            None => return,
        };

        let span_context = span.span_context();
        if span_context.is_valid() {
            let header_value = format!(
                "{:02x}-{:032x}-{:016x}-{:02x}",
                SUPPORTED_VERSION,
                span_context.trace_id(),
                span_context.span_id(),
                span_context.trace_flags() & TraceFlags::SAMPLED
            );
            injector.set(TRACERESPONSE_HEADER, header_value);
        }
    }

    /// Retrieves encoded `SpanContext`s using the `Extractor`. It decodes
    /// the `SpanContext` and returns it. If no `SpanContext` was retrieved
    /// OR if the retrieved SpanContext is invalid then an empty `SpanContext`
    /// is returned.
    fn extract_with_context(&self, cx: &Context, extractor: &dyn Extractor) -> Context {
        self.extract_span_context(extractor)
            .map(|sc| cx.with_remote_span_context(sc))
            .unwrap_or_else(|_| cx.clone())
    }

    fn fields(&self) -> FieldIter<'_> {
        FieldIter::new(TRACE_CONTEXT_HEADER_FIELDS.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::{testing::trace::TestSpan, trace::TraceState};
    use opentelemetry_api::{
        propagation::{Extractor, TextMapPropagator},
        trace::{SpanContext, SpanId, TraceId},
    };
    use std::{collections::HashMap, str::FromStr};

    #[rustfmt::skip]
    fn extract_data() -> Vec<(&'static str, SpanContext)> {
        vec![
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-00", SpanContext::new(TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from_u64(0x00f0_67aa_0ba9_02b7), TraceFlags::default(), true, TraceState::default())),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01", SpanContext::new(TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from_u64(0x00f0_67aa_0ba9_02b7), TraceFlags::SAMPLED, true, TraceState::default())),
            ("02-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01", SpanContext::new(TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from_u64(0x00f0_67aa_0ba9_02b7), TraceFlags::SAMPLED, true, TraceState::default())),
            ("02-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-09", SpanContext::new(TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from_u64(0x00f0_67aa_0ba9_02b7), TraceFlags::SAMPLED, true, TraceState::default())),
            ("02-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-08", SpanContext::new(TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from_u64(0x00f0_67aa_0ba9_02b7), TraceFlags::default(), true, TraceState::default())),
            ("02-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-09-XYZxsf09", SpanContext::new(TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from_u64(0x00f0_67aa_0ba9_02b7), TraceFlags::SAMPLED, true, TraceState::default())),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01-", SpanContext::new(TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from_u64(0x00f0_67aa_0ba9_02b7), TraceFlags::SAMPLED, true, TraceState::default())),
            ("01-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-09-", SpanContext::new(TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from_u64(0x00f0_67aa_0ba9_02b7), TraceFlags::SAMPLED, true, TraceState::default())),
        ]
    }

    #[rustfmt::skip]
    fn extract_data_invalid() -> Vec<(&'static str, &'static str)> {
        vec![
            ("0000-00000000000000000000000000000000-0000000000000000-01", "wrong version length"),
            ("00-ab00000000000000000000000000000000-cd00000000000000-01", "wrong trace ID length"),
            ("00-ab000000000000000000000000000000-cd0000000000000000-01", "wrong span ID length"),
            ("00-ab000000000000000000000000000000-cd00000000000000-0100", "wrong trace flag length"),
            ("qw-00000000000000000000000000000000-0000000000000000-01",   "bogus version"),
            ("00-qw000000000000000000000000000000-cd00000000000000-01",   "bogus trace ID"),
            ("00-ab000000000000000000000000000000-qw00000000000000-01",   "bogus span ID"),
            ("00-ab000000000000000000000000000000-cd00000000000000-qw",   "bogus trace flag"),
            ("A0-00000000000000000000000000000000-0000000000000000-01",   "upper case version"),
            ("00-AB000000000000000000000000000000-cd00000000000000-01",   "upper case trace ID"),
            ("00-ab000000000000000000000000000000-CD00000000000000-01",   "upper case span ID"),
            ("00-ab000000000000000000000000000000-cd00000000000000-A1",   "upper case trace flag"),
            ("00-00000000000000000000000000000000-0000000000000000-01",   "zero trace ID and span ID"),
            ("00-ab000000000000000000000000000000-cd00000000000000-09",   "trace-flag unused bits set"),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7",      "missing options"),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-",     "empty options"),
        ]
    }

    #[rustfmt::skip]
    fn inject_data() -> Vec<(&'static str, SpanContext)> {
        vec![
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01", SpanContext::new(TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from_u64(0x00f0_67aa_0ba9_02b7), TraceFlags::SAMPLED, true, TraceState::from_str("foo=bar").unwrap())),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-00", SpanContext::new(TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from_u64(0x00f0_67aa_0ba9_02b7), TraceFlags::default(), true, TraceState::from_str("foo=bar").unwrap())),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01", SpanContext::new(TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from_u64(0x00f0_67aa_0ba9_02b7), TraceFlags::new(0xff), true, TraceState::from_str("foo=bar").unwrap())),
            ("", SpanContext::empty_context()),
        ]
    }

    #[test]
    fn extract_w3c_traceresponse() {
        let propagator = TraceContextResponsePropagator::new();

        for (traceresponse, expected_context) in extract_data() {
            let mut extractor = HashMap::new();
            extractor.insert(TRACERESPONSE_HEADER.to_string(), traceresponse.to_string());

            assert_eq!(
                propagator
                    .extract(&extractor)
                    .span()
                    .unwrap()
                    .span_context(),
                &expected_context
            )
        }
    }

    #[test]
    fn extract_w3c_traceresponse_reject_invalid() {
        let propagator = TraceContextResponsePropagator::new();

        for (invalid_header, reason) in extract_data_invalid() {
            let mut extractor = HashMap::new();
            extractor.insert(TRACERESPONSE_HEADER.to_string(), invalid_header.to_string());

            assert_eq!(
                propagator
                    .extract(&extractor)
                    .span()
                    .unwrap()
                    .span_context(),
                &SpanContext::empty_context(),
                "{}",
                reason
            )
        }
    }

    #[test]
    fn inject_w3c_traceresponse() {
        let propagator = TraceContextResponsePropagator::new();

        for (expected_trace_response, context) in inject_data() {
            let mut injector = HashMap::new();
            propagator.inject_context(
                &Context::current_with_span(TestSpan(context)),
                &mut injector,
            );

            assert_eq!(
                Extractor::get(&injector, TRACERESPONSE_HEADER).unwrap_or(""),
                expected_trace_response
            );
        }
    }
}
