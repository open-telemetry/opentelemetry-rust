//! # B3 Propagator
//!
//! The `B3Propagator` facilitates `SpanContext` propagation using
//! B3 Headers. This propagator supports both version of B3 headers,
//!  1. Single Header:
//!    X-B3: {trace_id}-{span_id}-{sampling_state}-{parent_span_id}
//!  2. Multiple Headers:
//!    X-B3-TraceId: {trace_id}
//!    X-B3-ParentSpanId: {parent_span_id}
//!    X-B3-SpanId: {span_id}
//!    X-B3-Sampled: {sampling_state}
//!    X-B3-Flags: {debug_flag}
//!
//! If `single_header` is set to `true` then `X-B3` header is used to inject
//! and extract. Otherwise, separate headers are used to inject and extract.
use crate::{api, api::TraceContextExt};

static B3_SINGLE_HEADER: &str = "X-B3";
static B3_DEBUG_FLAG_HEADER: &str = "X-B3-Flags";
static B3_TRACE_ID_HEADER: &str = "X-B3-TraceId";
static B3_SPAN_ID_HEADER: &str = "X-B3-SpanId";
static B3_SAMPLED_HEADER: &str = "X-B3-Sampled";
static B3_PARENT_SPAN_ID_HEADER: &str = "X-B3-ParentSpanId";

/// Extracts and injects `SpanContext`s into `Carrier`s using B3 header format.
#[derive(Clone, Debug)]
pub struct B3Propagator {
    single_header: bool,
}

impl B3Propagator {
    /// Create a new `HttpB3Propagator`.
    pub fn new(single_header: bool) -> Self {
        B3Propagator { single_header }
    }

    /// Extract trace id from hex encoded &str value.
    fn extract_trace_id(&self, trace_id: &str) -> Result<api::TraceId, std::num::ParseIntError> {
        u128::from_str_radix(trace_id, 16).map(api::TraceId::from_u128)
    }

    /// Extract span id from hex encoded &str value.
    fn extract_span_id(&self, span_id: &str) -> Result<api::SpanId, std::num::ParseIntError> {
        u64::from_str_radix(span_id, 16).map(api::SpanId::from_u64)
    }

    /// Extract sampled state from encoded &str value
    fn extract_sampled_state(&self, sampled: &str) -> Result<u8, ()> {
        match sampled {
            "" | "0" => Ok(0),
            "1" => Ok(api::TRACE_FLAG_SAMPLED),
            "true" if !self.single_header => Ok(api::TRACE_FLAG_SAMPLED),
            "d" if self.single_header => Ok(api::TRACE_FLAG_SAMPLED),
            _ => Err(()),
        }
    }

    fn extract_debug_flag(&self, debug: &str) -> Result<u8, ()> {
        match debug {
            "" | "0" => Ok(0),
            "1" => Ok(api::TRACE_FLAG_SAMPLED),
            _ => Err(()),
        }
    }

    /// Extract a `SpanContext` from a single B3 header.
    fn extract_single_header(&self, carrier: &dyn api::Carrier) -> Result<api::SpanContext, ()> {
        let header_value = carrier.get(B3_SINGLE_HEADER).unwrap_or("");
        let parts = header_value.split_terminator('-').collect::<Vec<&str>>();
        // Ensure length is within range.
        if parts.len() > 4 || parts.len() < 2 {
            return Err(());
        }

        let trace_id = self.extract_trace_id(parts[0]).map_err(|_| ())?;
        let span_id = self.extract_span_id(parts[1]).map_err(|_| ())?;
        let trace_flags = if parts.len() > 2 {
            self.extract_sampled_state(parts[2])?
        } else {
            0
        };

        // Ensure parent id was valid
        if parts.len() == 4 {
            let _ = self.extract_span_id(parts[3]).map_err(|_| ())?;
        }

        let span_context = api::SpanContext::new(trace_id, span_id, trace_flags, true);

        // Ensure span is valid
        if !span_context.is_valid() {
            return Err(());
        }

        Ok(span_context)
    }

    /// Extract a `SpanContext` from multiple B3 headers.
    fn extract_multi_header(&self, carrier: &dyn api::Carrier) -> Result<api::SpanContext, ()> {
        let trace_id = self
            .extract_trace_id(carrier.get(B3_TRACE_ID_HEADER).unwrap_or(""))
            .map_err(|_| ())?;
        let span_id = self
            .extract_span_id(carrier.get(B3_SPAN_ID_HEADER).unwrap_or(""))
            .map_err(|_| ())?;
        // Only ensure valid parent span header if present.
        if let Some(parent) = carrier.get(B3_PARENT_SPAN_ID_HEADER) {
            let _ = self.extract_span_id(parent).map_err(|_| ());
        }
        let mut sampled =
            self.extract_sampled_state(carrier.get(B3_SAMPLED_HEADER).unwrap_or(""))?;
        let debug = self.extract_debug_flag(carrier.get(B3_DEBUG_FLAG_HEADER).unwrap_or(""))?;

        if debug == api::TRACE_FLAG_SAMPLED {
            sampled = api::TRACE_FLAG_SAMPLED;
        }

        let span_context = api::SpanContext::new(trace_id, span_id, sampled, true);

        if span_context.is_valid() {
            Ok(span_context)
        } else {
            Err(())
        }
    }
}

impl api::HttpTextFormat for B3Propagator {
    /// Properly encodes the values of the `Context`'s `SpanContext` and injects
    /// them into the `Carrier`.
    fn inject_context(&self, context: &api::Context, carrier: &mut dyn api::Carrier) {
        let span_context = context.span().span_context();
        if span_context.is_valid() {
            if self.single_header {
                let sampled = span_context.trace_flags() & api::TRACE_FLAG_SAMPLED;
                carrier.set(
                    B3_SINGLE_HEADER,
                    format!(
                        "{:032x}-{:016x}-{:01}",
                        span_context.trace_id().to_u128(),
                        span_context.span_id().to_u64(),
                        sampled
                    ),
                );
            } else {
                carrier.set(
                    B3_TRACE_ID_HEADER,
                    format!("{:032x}", span_context.trace_id().to_u128()),
                );
                carrier.set(
                    B3_SPAN_ID_HEADER,
                    format!("{:016x}", span_context.span_id().to_u64()),
                );

                let sampled = if span_context.is_sampled() { "1" } else { "0" };
                carrier.set(B3_SAMPLED_HEADER, sampled.to_string())
            }
        }
    }

    /// Retrieves encoded data using the provided `Carrier`. If no data for this
    /// format was retrieved OR if the retrieved data is invalid, then the current
    /// `Context` is returned.
    fn extract_with_context(&self, cx: &api::Context, carrier: &dyn api::Carrier) -> api::Context {
        let span_context = if self.single_header {
            self.extract_single_header(carrier)
                .unwrap_or_else(|_| api::SpanContext::empty_context())
        } else {
            self.extract_multi_header(carrier)
                .unwrap_or_else(|_| api::SpanContext::empty_context())
        };

        cx.with_remote_span_context(span_context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::trace::span_context::{SpanId, TraceId};
    use crate::api::HttpTextFormat;
    use std::collections::HashMap;

    #[rustfmt::skip]
    fn single_header_extract_data() -> Vec<(&'static str, api::SpanContext)> {
        vec![
            ("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7", api::SpanContext::new(TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 0, true)),
            ("4bf92f3577b34da6a3ce929d0e0e4736-f067aa0ba902b7-0", api::SpanContext::new(TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 0, true)),
            ("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-1", api::SpanContext::new(TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 1, true)),
            ("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-d", api::SpanContext::new(TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 1, true)),
            ("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-1-00000000000000cd", api::SpanContext::new(TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 1, true)),
            ("0", api::SpanContext::empty_context()),
        ]
    }

    #[rustfmt::skip]
    #[allow(clippy::type_complexity)]
    fn multi_header_extract_data() -> Vec<((Option<&'static str>, Option<&'static str>, Option<&'static str>, Option<&'static str>, Option<&'static str>), api::SpanContext)> {
        vec![
            ((Some("4bf92f3577b34da6a3ce929d0e0e4736"), Some("00f067aa0ba902b7"), None, None, None), api::SpanContext::new(TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 0, true)),
            ((Some("4bf92f3577b34da6a3ce929d0e0e4736"), Some("00f067aa0ba902b7"), Some("0"), None, None), api::SpanContext::new(TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 0, true)),
            ((Some("4bf92f3577b34da6a3ce929d0e0e4736"), Some("00f067aa0ba902b7"), Some("1"), None, None), api::SpanContext::new(TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 1, true)),
            ((Some("4bf92f3577b34da6a3ce929d0e0e4736"), Some("00f067aa0ba902b7"), Some("true"), None, None), api::SpanContext::new(TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 1, true)),
            ((Some("4bf92f3577b34da6a3ce929d0e0e4736"), Some("00f067aa0ba902b7"), None, Some("1"), None), api::SpanContext::new(TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 1, true)),
            ((Some("4bf92f3577b34da6a3ce929d0e0e4736"), Some("00f067aa0ba902b7"), Some("1"), None, Some("00f067aa0ba90200")), api::SpanContext::new(TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 1, true)),
            ((None, None, Some("0"), None, None), api::SpanContext::empty_context()),
        ]
    }

    #[rustfmt::skip]
    fn single_header_inject_data() -> Vec<(&'static str, api::SpanContext)> {
        vec![
            ("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-1", api::SpanContext::new(TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 1, true)),
            ("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-0", api::SpanContext::new(TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 0, true)),
            ("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-1", api::SpanContext::new(TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 0xff, true)),
        ]
    }

    #[rustfmt::skip]
    fn multi_header_inject_data() -> Vec<(&'static str, &'static str, &'static str, api::SpanContext)> {
        vec![
            ("4bf92f3577b34da6a3ce929d0e0e4736", "00f067aa0ba902b7", "1", api::SpanContext::new(TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 1, true)),
            ("4bf92f3577b34da6a3ce929d0e0e4736", "00f067aa0ba902b7", "0", api::SpanContext::new(TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 0, true)),
            ("4bf92f3577b34da6a3ce929d0e0e4736", "00f067aa0ba902b7", "1", api::SpanContext::new(TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from_u64(0x00f0_67aa_0ba9_02b7), 0xff, true)),
        ]
    }

    #[test]
    fn extract_b3() {
        let single_header_propagator = B3Propagator::new(true);
        let multi_header_propagator = B3Propagator::new(false);

        for (header, expected_context) in single_header_extract_data() {
            let mut carrier: HashMap<String, String> = HashMap::new();
            carrier.insert(B3_SINGLE_HEADER.to_string(), header.to_owned());
            assert_eq!(
                single_header_propagator
                    .extract(&carrier)
                    .remote_span_context(),
                Some(&expected_context)
            )
        }

        for ((trace, span, sampled, debug, parent), expected_context) in multi_header_extract_data()
        {
            let mut carrier = HashMap::new();
            if let Some(trace_id) = trace {
                carrier.insert(B3_TRACE_ID_HEADER.to_string(), trace_id.to_owned());
            }
            if let Some(span_id) = span {
                carrier.insert(B3_SPAN_ID_HEADER.to_string(), span_id.to_owned());
            }
            if let Some(sampled) = sampled {
                carrier.insert(B3_SAMPLED_HEADER.to_string(), sampled.to_owned());
            }
            if let Some(debug) = debug {
                carrier.insert(B3_DEBUG_FLAG_HEADER.to_string(), debug.to_owned());
            }
            if let Some(parent) = parent {
                carrier.insert(B3_PARENT_SPAN_ID_HEADER.to_string(), parent.to_owned());
            }
            assert_eq!(
                multi_header_propagator
                    .extract(&carrier)
                    .remote_span_context(),
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
    fn inject_b3() {
        let single_header_propagator = B3Propagator::new(true);
        let multi_header_propagator = B3Propagator::new(false);

        for (expected_header, context) in single_header_inject_data() {
            let mut carrier = HashMap::new();
            single_header_propagator.inject_context(
                &api::Context::current_with_span(TestSpan(context)),
                &mut carrier,
            );

            assert_eq!(
                carrier.get(B3_SINGLE_HEADER),
                Some(&expected_header.to_owned())
            )
        }

        for (trace_id, span_id, sampled, context) in multi_header_inject_data() {
            let mut carrier = HashMap::new();
            multi_header_propagator.inject_context(
                &api::Context::current_with_span(TestSpan(context)),
                &mut carrier,
            );

            assert_eq!(carrier.get(B3_TRACE_ID_HEADER), Some(&trace_id.to_owned()));
            assert_eq!(carrier.get(B3_SPAN_ID_HEADER), Some(&span_id.to_owned()));
            assert_eq!(carrier.get(B3_SAMPLED_HEADER), Some(&sampled.to_owned()));
            assert_eq!(carrier.get(B3_PARENT_SPAN_ID_HEADER), None);
        }
    }
}
