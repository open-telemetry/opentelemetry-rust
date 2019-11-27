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

use crate::api;

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
        let trace_id = u128::from_str_radix(parts[1], 16).map_err(|_| ())?;

        // Parse span id section
        let span_id = u64::from_str_radix(parts[2], 16).map_err(|_| ())?;

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
    fn inject(&self, context: api::SpanContext, carrier: &mut dyn api::Carrier) {
        if context.is_valid() {
            let header_value = format!(
                "{:02x}-{:032x}-{:016x}-{:02x}",
                SUPPORTED_VERSION,
                context.trace_id(),
                context.span_id(),
                context.trace_flags() & api::TRACE_FLAG_SAMPLED
            );
            carrier.set(TRACEPARENT_HEADER, header_value)
        }
    }

    /// Retrieves encoded `SpanContext`s using the `Carrier`. It decodes
    /// the `SpanContext` and returns it. If no `SpanContext` was retrieved
    /// OR if the retrieved SpanContext is invalid then an empty `SpanContext`
    /// is returned.
    fn extract(&self, carrier: &dyn api::Carrier) -> api::SpanContext {
        self.extract_span_context(carrier)
            .unwrap_or_else(|_| api::SpanContext::empty_context())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::api::{Carrier, HttpTextFormat};
    use std::collections::HashMap;

    #[rustfmt::skip]
    fn extract_data() -> Vec<(&'static str, api::SpanContext)> {
        vec![
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-00", api::SpanContext::new(0x4bf92f3577b34da6a3ce929d0e0e4736, 0x00f067aa0ba902b7, 0, true)),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01", api::SpanContext::new(0x4bf92f3577b34da6a3ce929d0e0e4736, 0x00f067aa0ba902b7, 1, true)),
            ("02-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01", api::SpanContext::new(0x4bf92f3577b34da6a3ce929d0e0e4736, 0x00f067aa0ba902b7, 1, true)),
            ("02-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-09", api::SpanContext::new(0x4bf92f3577b34da6a3ce929d0e0e4736, 0x00f067aa0ba902b7, 1, true)),
            ("02-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-08", api::SpanContext::new(0x4bf92f3577b34da6a3ce929d0e0e4736, 0x00f067aa0ba902b7, 0, true)),
            ("02-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-09-XYZxsf09", api::SpanContext::new(0x4bf92f3577b34da6a3ce929d0e0e4736, 0x00f067aa0ba902b7, 1, true)),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01-", api::SpanContext::new(0x4bf92f3577b34da6a3ce929d0e0e4736, 0x00f067aa0ba902b7, 1, true)),
            ("01-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-09-", api::SpanContext::new(0x4bf92f3577b34da6a3ce929d0e0e4736, 0x00f067aa0ba902b7, 1, true)),
        ]
    }
    #[rustfmt::skip]
    fn inject_data() -> Vec<(&'static str, api::SpanContext)> {
        vec![
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01", api::SpanContext::new(0x4bf92f3577b34da6a3ce929d0e0e4736, 0x00f067aa0ba902b7, 1, true)),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-00", api::SpanContext::new(0x4bf92f3577b34da6a3ce929d0e0e4736, 0x00f067aa0ba902b7, 0, true)),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01", api::SpanContext::new(0x4bf92f3577b34da6a3ce929d0e0e4736, 0x00f067aa0ba902b7, 0xff, true)),
            ("", api::SpanContext::empty_context()),
        ]
    }

    #[test]
    fn extract_w3c() {
        let propagator = TraceContextPropagator::new();

        for (header, expected_context) in extract_data() {
            let mut carrier: HashMap<&'static str, String> = HashMap::new();
            carrier.insert(TRACEPARENT_HEADER, header.to_owned());
            assert_eq!(propagator.extract(&carrier), expected_context)
        }
    }

    #[test]
    fn inject_w3c() {
        let propagator = TraceContextPropagator::new();

        for (expected_header, context) in inject_data() {
            let mut carrier = HashMap::new();
            propagator.inject(context, &mut carrier);

            assert_eq!(
                Carrier::get(&carrier, TRACEPARENT_HEADER).unwrap_or(""),
                expected_header
            )
        }
    }
}
