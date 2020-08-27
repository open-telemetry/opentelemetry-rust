//! # Trace Context Propagator
//!
//! The `traceparent` header represents the incoming request in a
//! tracing system in a common format, understood by all vendors.
//! Here’s an example of a `traceparent` header.
//!
//! `traceparent: 00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01`
//!
//! The `traceparent` HTTP header field identifies the incoming request in a
//! tracing system. It has four fields:
//!
//!    - version
//!    - trace-id
//!    - parent-id
//!    - trace-flags
//!
//! See the [w3c trace-context docs] for more details.
//!
//! [w3c trace-context docs]: https://w3c.github.io/trace-context/

use crate::api::context::propagation::text_propagator::FieldIter;
use crate::{api, api::TraceContextExt};

const SUPPORTED_VERSION: u8 = 0;
const MAX_VERSION: u8 = 254;
const TRACEPARENT_HEADER: &str = "traceparent";
const TRACESTATE_HEADER: &str = "tracestate";

lazy_static::lazy_static! {
    static ref TRACE_CONTEXT_HEADER_FIELDS: [String; 2] = [
        TRACEPARENT_HEADER.to_string(),
        TRACESTATE_HEADER.to_string()
    ];
}

/// Propagates `SpanContext`s in [W3C TraceContext] format.
///
/// [W3C TraceContext]: https://www.w3.org/TR/trace-context/
#[derive(Clone, Debug, Default)]
pub struct TraceContextPropagator {
    _private: (),
}

/// TraceState carries system-specific configuration data, represented as a list
/// of key-value pairs. TraceState allows multiple tracing systems to
/// participate in the same trace.
///
/// Please review the [W3C specification] for details on this field.
///
/// [W3C specification]: https://www.w3.org/TR/trace-context/#tracestate-header
#[derive(Debug)]
struct TraceState(String);

impl TraceContextPropagator {
    /// Create a new `TraceContextPropagator`.
    pub fn new() -> Self {
        TraceContextPropagator { _private: () }
    }

    /// Extract span context from w3c trace-context header.
    fn extract_span_context(&self, extractor: &dyn api::Extractor) -> Result<api::SpanContext, ()> {
        let header_value = extractor.get(TRACEPARENT_HEADER).unwrap_or("").trim();
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
        let trace_id = u128::from_str_radix(parts[1], 16)
            .map_err(|_| ())
            .map(api::TraceId::from_u128)?;

        // Ensure span id is lowercase
        if parts[2].chars().any(|c| c.is_ascii_uppercase()) {
            return Err(());
        }

        // Parse span id section
        let span_id = u64::from_str_radix(parts[2], 16)
            .map_err(|_| ())
            .map(api::SpanId::from_u64)?;

        // Parse trace flags section
        let opts = u8::from_str_radix(parts[3], 16).map_err(|_| ())?;

        // Ensure opts are valid for version 0
        if version == 0 && opts > 2 {
            return Err(());
        }

        // Build trace flags clearing all flags other than the trace-context
        // supported sampling bit.
        let trace_flags = opts & api::TRACE_FLAG_SAMPLED;

        // create context
        let span_context = api::SpanContext::new(trace_id, span_id, trace_flags, true);

        // Ensure span is valid
        if !span_context.is_valid() {
            return Err(());
        }

        Ok(span_context)
    }
}

impl api::TextMapFormat for TraceContextPropagator {
    /// Properly encodes the values of the `SpanContext` and injects them
    /// into the `Injector`.
    fn inject_context(&self, cx: &api::Context, injector: &mut dyn api::Injector) {
        if let Some(TraceState(header_value)) = cx.get() {
            injector.set(TRACESTATE_HEADER, header_value.clone());
        }

        let span_context = cx.span().span_context();
        if span_context.is_valid() {
            let header_value = format!(
                "{:02x}-{:032x}-{:016x}-{:02x}",
                SUPPORTED_VERSION,
                span_context.trace_id().to_u128(),
                span_context.span_id().to_u64(),
                span_context.trace_flags() & api::TRACE_FLAG_SAMPLED
            );
            injector.set(TRACEPARENT_HEADER, header_value)
        }
    }

    /// Retrieves encoded `SpanContext`s using the `Extractor`. It decodes
    /// the `SpanContext` and returns it. If no `SpanContext` was retrieved
    /// OR if the retrieved SpanContext is invalid then an empty `SpanContext`
    /// is returned.
    fn extract_with_context(
        &self,
        cx: &api::Context,
        extractor: &dyn api::Extractor,
    ) -> api::Context {
        self.extract_span_context(extractor)
            .map(|sc| {
                let cx = cx.with_remote_span_context(sc);

                // If successfully parsed span context, check trace state
                if let Some(state) = extractor
                    .get(TRACESTATE_HEADER)
                    .filter(|val| !val.is_empty())
                {
                    cx.with_value(TraceState(state.to_string()))
                } else {
                    cx
                }
            })
            .unwrap_or_else(|_| cx.clone())
    }

    fn fields(&self) -> FieldIter {
        FieldIter::new(TRACE_CONTEXT_HEADER_FIELDS.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{Extractor, TextMapFormat};
    use std::collections::HashMap;

    #[rustfmt::skip]
    fn extract_data() -> Vec<(&'static str, api::SpanContext)> {
        vec![
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-00", api::SpanContext::new(api::TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), api::SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 0, true)),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01", api::SpanContext::new(api::TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), api::SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 1, true)),
            ("02-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01", api::SpanContext::new(api::TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), api::SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 1, true)),
            ("02-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-09", api::SpanContext::new(api::TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), api::SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 1, true)),
            ("02-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-08", api::SpanContext::new(api::TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), api::SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 0, true)),
            ("02-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-09-XYZxsf09", api::SpanContext::new(api::TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), api::SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 1, true)),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01-", api::SpanContext::new(api::TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), api::SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 1, true)),
            ("01-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-09-", api::SpanContext::new(api::TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), api::SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 1, true)),
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
    fn inject_data() -> Vec<(&'static str, api::SpanContext)> {
        vec![
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01", api::SpanContext::new(api::TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), api::SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 1, true)),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-00", api::SpanContext::new(api::TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), api::SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 0, true)),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01", api::SpanContext::new(api::TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), api::SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 0xff, true)),
            ("", api::SpanContext::empty_context()),
        ]
    }

    #[test]
    fn extract_w3c() {
        let propagator = TraceContextPropagator::new();

        for (header, expected_context) in extract_data() {
            let mut extractor = HashMap::new();
            extractor.insert(TRACEPARENT_HEADER.to_string(), header.to_string());

            assert_eq!(
                propagator.extract(&extractor).remote_span_context(),
                Some(&expected_context)
            )
        }
    }

    #[test]
    fn extract_w3c_tracestate() {
        let propagator = TraceContextPropagator::new();
        let state = "opaque_value".to_string();
        let parent = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-00".to_string();

        let mut extractor = HashMap::new();
        extractor.insert(TRACEPARENT_HEADER.to_string(), parent);
        extractor.insert(TRACESTATE_HEADER.to_string(), state.clone());

        assert_eq!(
            propagator
                .extract(&extractor)
                .get::<TraceState>()
                .unwrap()
                .0,
            state
        )
    }

    #[test]
    fn extract_w3c_reject_invalid() {
        let propagator = TraceContextPropagator::new();

        for (invalid_header, reason) in extract_data_invalid() {
            let mut extractor = HashMap::new();
            extractor.insert(TRACEPARENT_HEADER.to_string(), invalid_header.to_string());

            assert_eq!(
                propagator.extract(&extractor).remote_span_context(),
                None,
                "{}",
                reason
            )
        }
    }

    #[derive(Debug)]
    struct TestSpan(api::SpanContext);

    impl api::Span for TestSpan {
        fn add_event_with_timestamp(
            &self,
            _name: String,
            _timestamp: std::time::SystemTime,
            _attributes: Vec<api::KeyValue>,
        ) {
        }
        fn span_context(&self) -> api::SpanContext {
            self.0.clone()
        }
        fn is_recording(&self) -> bool {
            false
        }
        fn set_attribute(&self, _attribute: api::KeyValue) {}
        fn set_status(&self, _code: api::StatusCode, _message: String) {}
        fn update_name(&self, _new_name: String) {}
        fn end_with_timestamp(&self, _timestamp: std::time::SystemTime) {}
    }

    #[test]
    fn inject_w3c() {
        let propagator = TraceContextPropagator::new();

        for (expected_header, context) in inject_data() {
            let mut injector = HashMap::new();
            propagator.inject_context(
                &api::Context::current_with_span(TestSpan(context)),
                &mut injector,
            );

            assert_eq!(
                Extractor::get(&injector, TRACEPARENT_HEADER).unwrap_or(""),
                expected_header
            )
        }
    }

    #[test]
    fn inject_w3c_tracestate() {
        let propagator = TraceContextPropagator::new();
        let state = "opaque_value";

        let mut injector = HashMap::new();
        propagator.inject_context(
            &api::Context::current_with_value(TraceState(state.to_string())),
            &mut injector,
        );

        assert_eq!(Extractor::get(&injector, TRACESTATE_HEADER), Some(state))
    }
}
