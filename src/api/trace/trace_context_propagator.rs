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

use crate::{api, api::TraceContextExt};

static SUPPORTED_VERSION: u8 = 0;
static MAX_VERSION: u8 = 254;
static TRACEPARENT_HEADER: &str = "Traceparent";

/// Extracts and injects `SpanContext`s into `Carrier`s using the
/// trace-context format.
#[derive(Debug, Default)]
pub struct TraceContextPropagator {}

impl TraceContextPropagator {
    /// Create a new `TraceContextPropagator`.
    pub fn new() -> Self {
        TraceContextPropagator {}
    }

    /// Extract span context from w3c trace-context header.
    fn extract_span_context(&self, carrier: &dyn api::Carrier) -> Result<api::SpanContext, ()> {
        let header_value = carrier.get(TRACEPARENT_HEADER).unwrap_or("").trim();
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

        // Parse trace id section
        let trace_id = u128::from_str_radix(parts[1], 16)
            .map_err(|_| ())
            .map(api::TraceId::from_u128)?;

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
        // Build trace flags
        let trace_flags = opts & !api::TRACE_FLAGS_UNUSED;

        // create context
        let span_context = api::SpanContext::new(trace_id, span_id, trace_flags, true);

        // Ensure span is valid
        if !span_context.is_valid() {
            return Err(());
        }

        Ok(span_context)
    }
}

impl api::HttpTextFormat for TraceContextPropagator {
    /// Properly encodes the values of the `SpanContext` and injects them
    /// into the `Carrier`.
    fn inject_context(&self, context: &api::Context, carrier: &mut dyn api::Carrier) {
        let span_context = context.span().span_context();
        if span_context.is_valid() {
            let header_value = format!(
                "{:02x}-{:032x}-{:016x}-{:02x}",
                SUPPORTED_VERSION,
                span_context.trace_id().to_u128(),
                span_context.span_id().to_u64(),
                span_context.trace_flags() & api::TRACE_FLAG_SAMPLED
            );
            carrier.set(TRACEPARENT_HEADER, header_value)
        }
    }

    /// Retrieves encoded `SpanContext`s using the `Carrier`. It decodes
    /// the `SpanContext` and returns it. If no `SpanContext` was retrieved
    /// OR if the retrieved SpanContext is invalid then an empty `SpanContext`
    /// is returned.
    fn extract(&self, carrier: &dyn api::Carrier) -> api::Context {
        self.extract_span_context(carrier)
            .map(api::Context::current_with_remote_span_context)
            .unwrap_or_else(|_| api::Context::current())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{Carrier, HttpTextFormat};
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
            let mut carrier: HashMap<&'static str, String> = HashMap::new();
            carrier.insert(TRACEPARENT_HEADER, header.to_owned());
            assert_eq!(
                propagator.extract(&carrier).remote_span_context(),
                Some(&expected_context)
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
        fn end(&self) {}
    }

    #[test]
    fn inject_w3c() {
        let propagator = TraceContextPropagator::new();

        for (expected_header, context) in inject_data() {
            let mut carrier = HashMap::new();
            propagator.inject_context(
                &api::Context::current_with_span(TestSpan(context)),
                &mut carrier,
            );

            assert_eq!(
                Carrier::get(&carrier, TRACEPARENT_HEADER).unwrap_or(""),
                expected_header
            )
        }
    }
}
