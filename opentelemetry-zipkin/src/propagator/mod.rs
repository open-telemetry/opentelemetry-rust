//! # B3 Propagator
//!
//! The `B3Propagator` facilitates `SpanContext` propagation using
//! B3 Headers. This propagator supports both version of B3 headers,
//!  1. Single Header:
//!    b3: {trace_id}-{span_id}-{sampling_state}-{parent_span_id}
//!  2. Multiple Headers:
//!    X-B3-TraceId: {trace_id}
//!    X-B3-ParentSpanId: {parent_span_id}
//!    X-B3-SpanId: {span_id}
//!    X-B3-Sampled: {sampling_state}
//!    X-B3-Flags: {debug_flag}
//!
//! If `inject_encoding` is set to `B3Encoding::SingleHeader` then `b3` header is used to inject
//! and extract. Otherwise, separate headers are used to inject and extract.
use opentelemetry::{
    propagation::{text_map_propagator::FieldIter, Extractor, Injector, TextMapPropagator},
    trace::{SpanContext, SpanId, TraceContextExt, TraceFlags, TraceId, TraceState},
    Context,
};

const B3_SINGLE_HEADER: &str = "b3";
/// As per spec, the multiple header should be case sensitive. But different protocol will use
/// different formats. For example, HTTP will use X-B3-$name while gRPC will use x-b3-$name. So here
/// we leave it to be lower case since we cannot tell what kind of protocol will be used.
/// Go implementation also uses lower case.
const B3_DEBUG_FLAG_HEADER: &str = "x-b3-flags";
const B3_TRACE_ID_HEADER: &str = "x-b3-traceid";
const B3_SPAN_ID_HEADER: &str = "x-b3-spanid";
const B3_SAMPLED_HEADER: &str = "x-b3-sampled";
const B3_PARENT_SPAN_ID_HEADER: &str = "x-b3-parentspanid";

const TRACE_FLAG_DEFERRED: TraceFlags = TraceFlags::new(0x02);
const TRACE_FLAG_DEBUG: TraceFlags = TraceFlags::new(0x04);

lazy_static::lazy_static! {
    static ref B3_SINGLE_FIELDS: [String; 1] = [B3_SINGLE_HEADER.to_string()];
    static ref B3_MULTI_FIELDS: [String; 4] = [B3_TRACE_ID_HEADER.to_string(), B3_SPAN_ID_HEADER.to_string(), B3_SAMPLED_HEADER.to_string(), B3_DEBUG_FLAG_HEADER.to_string()];
    static ref B3_SINGLE_AND_MULTI_FIELDS: [String; 5] = [B3_SINGLE_HEADER.to_string(), B3_TRACE_ID_HEADER.to_string(), B3_SPAN_ID_HEADER.to_string(), B3_SAMPLED_HEADER.to_string(), B3_DEBUG_FLAG_HEADER.to_string()];
}

/// B3Encoding is a bitmask to represent B3 encoding type
#[derive(Clone, Debug)]
pub enum B3Encoding {
    /// Unspecified is an unspecified B3 encoding
    UnSpecified = 0,
    /// MultipleHeader is a B3 encoding that uses multiple headers
    /// to transmit tracing information prefixed with `X-B3-`
    MultipleHeader = 1,
    /// SingleHeader is B3 encoding that uses a single header named `b3`
    /// to transmit tracing information
    SingleHeader = 2,
    /// SingleAndMultiHeader is B3 encoding that uses both single header and multiple headers
    /// to transmit tracing information. Note that if both single header and multiple headers are
    /// provided, the single header will take precedence when extracted
    SingleAndMultiHeader = 3,
}

impl B3Encoding {
    /// support determines if current encoding supports the `e`
    pub fn support(&self, other: &Self) -> bool {
        (self.clone() as u8) & (other.clone() as u8) == (other.clone() as u8)
    }
}

/// Extracts and injects `SpanContext`s into `Extractor`s or `Injector`s using B3 header format.
#[derive(Clone, Debug)]
pub struct Propagator {
    inject_encoding: B3Encoding,
}

impl Default for Propagator {
    fn default() -> Self {
        Propagator {
            inject_encoding: B3Encoding::MultipleHeader,
        }
    }
}

impl Propagator {
    /// Create a new `HttpB3Propagator` that uses multiple headers.
    pub fn new() -> Self {
        Propagator::default()
    }

    /// Create a new `HttpB3Propagator` that uses `encoding` as encoding method
    pub fn with_encoding(encoding: B3Encoding) -> Self {
        Propagator {
            inject_encoding: encoding,
        }
    }

    /// Extract trace id from hex encoded &str value.
    fn extract_trace_id(&self, trace_id: &str) -> Result<TraceId, ()> {
        // Only allow lower case hex string
        if trace_id.to_lowercase() != trace_id || (trace_id.len() != 16 && trace_id.len() != 32) {
            Err(())
        } else {
            u128::from_str_radix(trace_id, 16)
                .map(TraceId::from_u128)
                .map_err(|_| ())
        }
    }

    /// Extract span id from hex encoded &str value.
    fn extract_span_id(&self, span_id: &str) -> Result<SpanId, ()> {
        // Only allow lower case hex string
        if span_id.to_lowercase() != span_id || span_id.len() != 16 {
            Err(())
        } else {
            u64::from_str_radix(span_id, 16)
                .map(SpanId::from_u64)
                .map_err(|_| ())
        }
    }

    /// Extract sampled state from encoded &str value
    /// For legacy support and  being lenient to other tracing implementations we
    /// allow "true" and "false" as inputs for interop purposes.
    fn extract_sampled_state(&self, sampled: &str) -> Result<TraceFlags, ()> {
        match sampled {
            "0" | "false" => Ok(TraceFlags::default()),
            "1" => Ok(TraceFlags::SAMPLED),
            "true" if !self.inject_encoding.support(&B3Encoding::SingleHeader) => {
                Ok(TraceFlags::SAMPLED)
            }
            "d" if self.inject_encoding.support(&B3Encoding::SingleHeader) => Ok(TRACE_FLAG_DEBUG),
            _ => Err(()),
        }
    }

    fn extract_debug_flag(&self, debug: &str) -> Result<TraceFlags, ()> {
        match debug {
            "0" => Ok(TraceFlags::default()),
            "1" => Ok(TRACE_FLAG_DEBUG | TraceFlags::SAMPLED), // debug implies sampled
            _ => Err(()),
        }
    }

    /// Extract a `SpanContext` from a single B3 header.
    fn extract_single_header(&self, extractor: &dyn Extractor) -> Result<SpanContext, ()> {
        let header_value = extractor.get(B3_SINGLE_HEADER).unwrap_or("");
        let parts = header_value.split_terminator('-').collect::<Vec<&str>>();
        // Ensure length is within range.
        if parts.len() > 4 || parts.len() < 2 {
            return Err(());
        }

        let trace_id = self.extract_trace_id(parts[0])?;
        let span_id = self.extract_span_id(parts[1])?;
        let trace_flags = if parts.len() > 2 {
            self.extract_sampled_state(parts[2])?
        } else {
            TRACE_FLAG_DEFERRED
        };

        // Ensure parent id was valid
        if parts.len() == 4 {
            let _ = self.extract_span_id(parts[3])?;
        }

        let span_context =
            SpanContext::new(trace_id, span_id, trace_flags, true, TraceState::default());

        // Ensure span is valid
        if !span_context.is_valid() {
            return Err(());
        }

        Ok(span_context)
    }

    /// Extract a `SpanContext` from multiple B3 headers.
    fn extract_multi_header(&self, extractor: &dyn Extractor) -> Result<SpanContext, ()> {
        let trace_id = self
            .extract_trace_id(extractor.get(B3_TRACE_ID_HEADER).unwrap_or(""))
            .map_err(|_| ())?;
        let span_id = self
            .extract_span_id(extractor.get(B3_SPAN_ID_HEADER).unwrap_or(""))
            .map_err(|_| ())?;
        // Only ensure valid parent span header if present.
        if let Some(parent) = extractor.get(B3_PARENT_SPAN_ID_HEADER) {
            let _ = self.extract_span_id(parent).map_err(|_| ());
        }

        let debug = self.extract_debug_flag(extractor.get(B3_DEBUG_FLAG_HEADER).unwrap_or(""));
        let sampled_opt = extractor.get(B3_SAMPLED_HEADER);

        let flag = if let Ok(debug_flag) = debug {
            // if debug is set, then X-B3-Sampled should not be sent. Will ignore
            debug_flag
        } else if let Some(sampled) = sampled_opt {
            // if debug is not set and X-B3-Sampled is not set, then deferred
            // if sample value is invalid, then return empty context.
            self.extract_sampled_state(sampled)?
        } else {
            TRACE_FLAG_DEFERRED
        };

        let span_context = SpanContext::new(trace_id, span_id, flag, true, TraceState::default());

        if span_context.is_valid() {
            Ok(span_context)
        } else {
            Err(())
        }
    }
}

impl TextMapPropagator for Propagator {
    /// Properly encodes the values of the `Context`'s `SpanContext` and injects
    /// them into the `Injector`.
    fn inject_context(&self, context: &Context, injector: &mut dyn Injector) {
        let span = context.span();
        let span_context = span.span_context();
        if span_context.is_valid() {
            let is_deferred =
                span_context.trace_flags() & TRACE_FLAG_DEFERRED == TRACE_FLAG_DEFERRED;
            let is_debug = span_context.trace_flags() & TRACE_FLAG_DEBUG == TRACE_FLAG_DEBUG;
            if self.inject_encoding.support(&B3Encoding::SingleHeader) {
                let mut value = format!(
                    "{:032x}-{:016x}",
                    span_context.trace_id().to_u128(),
                    span_context.span_id().to_u64(),
                );
                if !is_deferred {
                    let flag = if is_debug {
                        "d"
                    } else if span_context.is_sampled() {
                        "1"
                    } else {
                        "0"
                    };
                    value = format!("{}-{:01}", value, flag)
                }

                injector.set(B3_SINGLE_HEADER, value);
            }
            if self.inject_encoding.support(&B3Encoding::MultipleHeader)
                || self.inject_encoding.support(&B3Encoding::UnSpecified)
            {
                // if inject_encoding is Unspecified, default to use MultipleHeader
                injector.set(
                    B3_TRACE_ID_HEADER,
                    format!("{:032x}", span_context.trace_id().to_u128()),
                );
                injector.set(
                    B3_SPAN_ID_HEADER,
                    format!("{:016x}", span_context.span_id().to_u64()),
                );

                if is_debug {
                    injector.set(B3_DEBUG_FLAG_HEADER, "1".to_string());
                } else if !is_deferred {
                    let sampled = if span_context.is_sampled() { "1" } else { "0" };
                    injector.set(B3_SAMPLED_HEADER, sampled.to_string());
                }
            }
        } else {
            let flag = if span_context.is_sampled() { "1" } else { "0" };
            if self.inject_encoding.support(&B3Encoding::SingleHeader) {
                injector.set(B3_SINGLE_HEADER, flag.to_string())
            }
            if self.inject_encoding.support(&B3Encoding::MultipleHeader)
                || self.inject_encoding.support(&B3Encoding::UnSpecified)
            {
                injector.set(B3_SAMPLED_HEADER, flag.to_string())
            }
        }
    }

    /// Retrieves encoded data using the provided `Extractor`. If no data for this
    /// format was retrieved OR if the retrieved data is invalid, then the current
    /// `Context` is returned.
    fn extract_with_context(&self, cx: &Context, extractor: &dyn Extractor) -> Context {
        let extract_result = self.extract_single_header(extractor).or_else(|_| {
            // if invalid single header should fallback to multiple
            self.extract_multi_header(extractor)
        });

        if let Some(span_context) = extract_result.ok().filter(|cx| cx.is_valid()) {
            cx.with_remote_span_context(span_context)
        } else {
            cx.clone()
        }
    }

    fn fields(&self) -> FieldIter<'_> {
        let field_slice = if self
            .inject_encoding
            .support(&B3Encoding::SingleAndMultiHeader)
        {
            B3_SINGLE_AND_MULTI_FIELDS.as_ref()
        } else if self.inject_encoding.support(&B3Encoding::MultipleHeader) {
            B3_MULTI_FIELDS.as_ref()
        } else if self.inject_encoding.support(&B3Encoding::SingleHeader) {
            B3_SINGLE_FIELDS.as_ref()
        } else {
            B3_MULTI_FIELDS.as_ref()
        };

        FieldIter::new(field_slice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::{
        propagation::TextMapPropagator,
        testing::trace::TestSpan,
        trace::{SpanContext, SpanId, TraceFlags, TraceId},
    };
    use std::collections::HashMap;

    const TRACE_ID_STR: &str = "4bf92f3577b34da6a3ce929d0e0e4736";
    const SPAN_ID_STR: &str = "00f067aa0ba902b7";
    const TRACE_ID_HEX: u128 = 0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736;
    const SPAN_ID_HEX: u64 = 0x00f0_67aa_0ba9_02b7;

    #[rustfmt::skip]
    fn single_header_extract_data() -> Vec<(&'static str, SpanContext)> {
        vec![
            ("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7", SpanContext::new(TraceId::from_u128(TRACE_ID_HEX), SpanId::from_u64(SPAN_ID_HEX), TRACE_FLAG_DEFERRED, true, TraceState::default())), // deferred
            ("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-0", SpanContext::new(TraceId::from_u128(TRACE_ID_HEX), SpanId::from_u64(SPAN_ID_HEX), TraceFlags::default(), true, TraceState::default())), // not sampled
            ("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-1", SpanContext::new(TraceId::from_u128(TRACE_ID_HEX), SpanId::from_u64(SPAN_ID_HEX), TraceFlags::SAMPLED, true, TraceState::default())), // sampled
            ("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-d", SpanContext::new(TraceId::from_u128(TRACE_ID_HEX), SpanId::from_u64(SPAN_ID_HEX), TRACE_FLAG_DEBUG, true, TraceState::default())), // debug
            ("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-1-00000000000000cd", SpanContext::new(TraceId::from_u128(TRACE_ID_HEX), SpanId::from_u64(SPAN_ID_HEX), TraceFlags::SAMPLED, true, TraceState::default())), // with parent span id
            ("a3ce929d0e0e4736-00f067aa0ba902b7-1-00000000000000cd", SpanContext::new(TraceId::from_u128(0x0000_0000_0000_0000_a3ce_929d_0e0e_4736), SpanId::from_u64(SPAN_ID_HEX), TraceFlags::SAMPLED, true, TraceState::default())), // padding 64 bit traceID
            ("0", SpanContext::empty_context()),
            ("-", SpanContext::empty_context()),
        ]
    }

    #[rustfmt::skip]
    #[allow(clippy::type_complexity)]
    fn multi_header_extract_data() -> Vec<((Option<&'static str>, Option<&'static str>, Option<&'static str>, Option<&'static str>, Option<&'static str>), SpanContext)> {
        // (TraceId, SpanId, Sampled, FlagId, ParentSpanId)
        vec![
            ((Some(TRACE_ID_STR), Some(SPAN_ID_STR), None, None, None), SpanContext::new(TraceId::from_u128(TRACE_ID_HEX), SpanId::from_u64(SPAN_ID_HEX), TRACE_FLAG_DEFERRED, true, TraceState::default())), // deferred
            ((Some(TRACE_ID_STR), Some(SPAN_ID_STR), Some("0"), None, None), SpanContext::new(TraceId::from_u128(TRACE_ID_HEX), SpanId::from_u64(SPAN_ID_HEX), TraceFlags::default(), true, TraceState::default())), // not sampled
            ((Some(TRACE_ID_STR), Some(SPAN_ID_STR), Some("1"), None, None), SpanContext::new(TraceId::from_u128(TRACE_ID_HEX), SpanId::from_u64(SPAN_ID_HEX), TraceFlags::SAMPLED, true, TraceState::default())), // sampled
            ((Some(TRACE_ID_STR), Some(SPAN_ID_STR), Some("true"), None, None), SpanContext::new(TraceId::from_u128(TRACE_ID_HEX), SpanId::from_u64(SPAN_ID_HEX), TraceFlags::SAMPLED, true, TraceState::default())),
            ((Some(TRACE_ID_STR), Some(SPAN_ID_STR), Some("false"), None, None), SpanContext::new(TraceId::from_u128(TRACE_ID_HEX), SpanId::from_u64(SPAN_ID_HEX), TraceFlags::default(), true, TraceState::default())), // use true/false to set sample
            ((Some(TRACE_ID_STR), Some(SPAN_ID_STR), None, Some("1"), None), SpanContext::new(TraceId::from_u128(TRACE_ID_HEX), SpanId::from_u64(SPAN_ID_HEX), TRACE_FLAG_DEBUG | TraceFlags::SAMPLED, true, TraceState::default())), // debug
            ((Some(TRACE_ID_STR), Some(SPAN_ID_STR), Some("0"), Some("1"), Some("00f067aa0ba90200")), SpanContext::new(TraceId::from_u128(TRACE_ID_HEX), SpanId::from_u64(SPAN_ID_HEX), TRACE_FLAG_DEBUG | TraceFlags::SAMPLED, true, TraceState::default())),  // debug flag should override sample flag
            ((Some(TRACE_ID_STR), Some(SPAN_ID_STR), Some("1"), Some("2"), Some("00f067aa0ba90200")), SpanContext::new(TraceId::from_u128(TRACE_ID_HEX), SpanId::from_u64(SPAN_ID_HEX), TraceFlags::SAMPLED, true, TraceState::default())), // invalid debug flag, should ignore
            ((None, None, Some("0"), None, None), SpanContext::empty_context()),
        ]
    }

    #[rustfmt::skip]
    #[allow(clippy::type_complexity)]
    fn single_header_extract_invalid_data() -> Vec<&'static str> {
        vec![
            "ab00000000000000000000000000000000-cd00000000000000-1", // wrong trace id length
            "ab000000000000000000000000000000-cd0000000000000000-1", // wrong span id length
            "00-ab000000000000000000000000000000-cd00000000000000-01", // wrong sampled state length
            "ab000000000000000000000000000000-cd00000000000000-1-cd0000000000000000", // wrong parent span id length
            "qw000000000000000000000000000000-cd00000000000000-1", // trace id with bug
            "ab000000000000000000000000000000-qw00000000000000-1", // span id with bug
            "ab000000000000000000000000000000-cd00000000000000-q", // sample flag bug
            "AB000000000000000000000000000000-cd00000000000000-1", // upper case trace id
            "ab000000000000000000000000000000-CD00000000000000-1", // upper case span id
            "ab000000000000000000000000000000-cd00000000000000-1-EF00000000000000", // upper case parent span id
            "ab000000000000000000000000000000-cd00000000000000-true", // invalid sample flag(set to true)
        ]
    }

    #[rustfmt::skip]
    #[allow(clippy::type_complexity)]
    fn multi_header_extract_invalid_data() -> Vec<(Option<&'static str>, Option<&'static str>, Option<&'static str>, Option<&'static str>, Option<&'static str>)> {
        vec![
            (None, None, None, None, None),
            (None, Some(SPAN_ID_STR), None, None, None), // missing trace id
            (Some(TRACE_ID_STR), None, None, None, None), // missing span id
            (Some("ab00000000000000000000000000000000"), Some("cd00000000000000"), Some("1"), None, None), // trace ID length > 32
            (Some("ab0000000000000000000000000000"), Some("cd00000000000000"), Some("1"), None, None), // trace ID length > 16 and < 32
            (Some("ab0000000000"), Some("cd00000000000000"), Some("1"), None, None), // trace ID length < 16
            (Some("ab000000000000000000000000000000"), Some("cd0000000000000000"), Some("1"), None, None), // trace ID length is wrong
            (Some(TRACE_ID_STR), Some(SPAN_ID_STR), Some("10"), None, None), // flag length is wrong
            (Some(TRACE_ID_STR), Some(SPAN_ID_STR), Some("d"), None, None), // flag contains illegal char
            (Some("4bf92f3577b34da6a3ce929d0e0e4hhh"), Some(SPAN_ID_STR), Some("1"), None, None), // hex contains illegal char
            (Some("4BF92F3577B34DA6A3CE929D0E0E4736"), Some(SPAN_ID_STR), Some("1"), None, None), // trace id is upper case hex string
            (Some(TRACE_ID_STR), Some("00F067AA0BA902B7"), Some("1"), None, None), // span id is upper case hex string
        ]
    }

    #[rustfmt::skip]
    #[allow(clippy::type_complexity)]
    fn single_multi_header_extract_data() -> Vec<((Option<&'static str>, Option<&'static str>, Option<&'static str>, Option<&'static str>, Option<&'static str>), &'static str, SpanContext)> {
        // (TraceId, SpanId, Sampled, FlagId, ParentSpanId), b3
        vec![
            ((Some(TRACE_ID_STR), Some(SPAN_ID_STR), None, None, None), "4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-0",
             SpanContext::new(TraceId::from_u128(TRACE_ID_HEX), SpanId::from_u64(SPAN_ID_HEX), TraceFlags::default(), true, TraceState::default())), // single header take precedence
            ((Some(TRACE_ID_STR), Some(SPAN_ID_STR), Some("0"), None, None), "-", SpanContext::new(TraceId::from_u128(TRACE_ID_HEX), SpanId::from_u64(SPAN_ID_HEX), TraceFlags::default(), true, TraceState::default())), // when single header is invalid, fall back to multiple headers
            ((Some("0"), Some("0"), Some("0"), None, None), "4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-0", SpanContext::new(TraceId::from_u128(TRACE_ID_HEX), SpanId::from_u64(SPAN_ID_HEX), TraceFlags::default(), true, TraceState::default())) // invalid multiple header should go unnoticed since single header take precedence.
        ]
    }

    #[rustfmt::skip]
    fn single_header_inject_data() -> Vec<(&'static str, SpanContext)> {
        vec![
            ("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-1", SpanContext::new(TraceId::from_u128(TRACE_ID_HEX), SpanId::from_u64(SPAN_ID_HEX), TraceFlags::SAMPLED, true, TraceState::default())),
            ("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-d", SpanContext::new(TraceId::from_u128(TRACE_ID_HEX), SpanId::from_u64(SPAN_ID_HEX), TRACE_FLAG_DEBUG, true, TraceState::default())),
            ("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7", SpanContext::new(TraceId::from_u128(TRACE_ID_HEX), SpanId::from_u64(SPAN_ID_HEX), TRACE_FLAG_DEFERRED, true, TraceState::default())),
            ("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-0", SpanContext::new(TraceId::from_u128(TRACE_ID_HEX), SpanId::from_u64(SPAN_ID_HEX), TraceFlags::default(), true, TraceState::default())),
            ("1", SpanContext::new(TraceId::invalid(), SpanId::invalid(), TraceFlags::SAMPLED, true, TraceState::default())),
            ("0", SpanContext::new(TraceId::invalid(), SpanId::invalid(), TraceFlags::default(), true, TraceState::default())),
        ]
    }

    #[rustfmt::skip]
    #[allow(clippy::type_complexity)]
    fn multi_header_inject_data() -> Vec<(Option<&'static str>, Option<&'static str>, Option<&'static str>, Option<&'static str>, SpanContext)> {
        // TraceId, SpanId, isSampled, isDebug
        vec![
            (Some(TRACE_ID_STR), Some(SPAN_ID_STR), Some("1"), None, SpanContext::new(TraceId::from_u128(TRACE_ID_HEX), SpanId::from_u64(SPAN_ID_HEX), TraceFlags::SAMPLED, true, TraceState::default())),
            (Some(TRACE_ID_STR), Some(SPAN_ID_STR), None, Some("1"), SpanContext::new(TraceId::from_u128(TRACE_ID_HEX), SpanId::from_u64(SPAN_ID_HEX), TRACE_FLAG_DEBUG, true, TraceState::default())),
            (Some(TRACE_ID_STR), Some(SPAN_ID_STR), None, None, SpanContext::new(TraceId::from_u128(TRACE_ID_HEX), SpanId::from_u64(SPAN_ID_HEX), TRACE_FLAG_DEFERRED, true, TraceState::default())),
            (Some(TRACE_ID_STR), Some(SPAN_ID_STR), Some("0"), None, SpanContext::new(TraceId::from_u128(TRACE_ID_HEX), SpanId::from_u64(SPAN_ID_HEX), TraceFlags::default(), true, TraceState::default())),
            (None, None, Some("0"), None, SpanContext::empty_context()),
            (None, None, Some("1"), None, SpanContext::new(TraceId::invalid(), SpanId::invalid(), TraceFlags::SAMPLED, true, TraceState::default()))
        ]
    }

    #[rustfmt::skip]
    #[allow(clippy::type_complexity)]
    fn single_multi_header_inject_data() -> Vec<(Option<&'static str>, Option<&'static str>, Option<&'static str>, Option<&'static str>, Option<&'static str>, SpanContext)> {
        let trace_id: TraceId = TraceId::from_u128(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736);
        let span_id: SpanId = SpanId::from_u64(0x00f0_67aa_0ba9_02b7);
        vec![
            (Some(TRACE_ID_STR), Some(SPAN_ID_STR), Some("1"), None, Some("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-1"), SpanContext::new(trace_id, span_id, TraceFlags::SAMPLED, true, TraceState::default())), // sampled
            (Some(TRACE_ID_STR), Some(SPAN_ID_STR), None, Some("1"), Some("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-d"), SpanContext::new(trace_id, span_id, TRACE_FLAG_DEBUG, true, TraceState::default())), // debug
            (Some(TRACE_ID_STR), Some(SPAN_ID_STR), Some("0"), None, Some("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-0"), SpanContext::new(trace_id, span_id, TraceFlags::default(), true, TraceState::default())), // not sampled
            (Some(TRACE_ID_STR), Some(SPAN_ID_STR), None, None, Some("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7"), SpanContext::new(trace_id, span_id, TRACE_FLAG_DEFERRED, true, TraceState::default())), // unset sampled
            (None, None, Some("0"), None, Some("0"), SpanContext::empty_context()),
            (None, None, Some("1"), None, Some("1"), SpanContext::new(TraceId::invalid(), SpanId::invalid(), TraceFlags::SAMPLED, true, TraceState::default())),
        ]
    }

    fn extract_extrator_from_test_data(
        trace: Option<&'static str>,
        span: Option<&'static str>,
        sampled: Option<&'static str>,
        debug: Option<&'static str>,
        parent: Option<&'static str>,
    ) -> HashMap<String, String> {
        let mut extractor = HashMap::new();
        if let Some(trace_id) = trace {
            extractor.insert(B3_TRACE_ID_HEADER.to_string(), trace_id.to_owned());
        }
        if let Some(span_id) = span {
            extractor.insert(B3_SPAN_ID_HEADER.to_string(), span_id.to_owned());
        }
        if let Some(sampled) = sampled {
            extractor.insert(B3_SAMPLED_HEADER.to_string(), sampled.to_owned());
        }
        if let Some(debug) = debug {
            extractor.insert(B3_DEBUG_FLAG_HEADER.to_string(), debug.to_owned());
        }
        if let Some(parent) = parent {
            extractor.insert(B3_PARENT_SPAN_ID_HEADER.to_string(), parent.to_owned());
        }
        extractor
    }

    #[test]
    fn extract_b3() {
        let single_header_propagator = Propagator::with_encoding(B3Encoding::SingleHeader);
        let multi_header_propagator = Propagator::with_encoding(B3Encoding::MultipleHeader);
        let single_multi_propagator = Propagator::with_encoding(B3Encoding::SingleAndMultiHeader);
        let unspecific_header_propagator = Propagator::with_encoding(B3Encoding::UnSpecified);

        for (header, expected_context) in single_header_extract_data() {
            let mut extractor: HashMap<String, String> = HashMap::new();
            extractor.insert(B3_SINGLE_HEADER.to_string(), header.to_owned());
            assert_eq!(
                single_header_propagator
                    .extract(&extractor)
                    .span()
                    .span_context()
                    .clone(),
                expected_context
            )
        }

        for ((trace, span, sampled, debug, parent), expected_context) in multi_header_extract_data()
        {
            let extractor = extract_extrator_from_test_data(trace, span, sampled, debug, parent);
            assert_eq!(
                multi_header_propagator
                    .extract(&extractor)
                    .span()
                    .span_context()
                    .clone(),
                expected_context
            );
            assert_eq!(
                unspecific_header_propagator
                    .extract(&extractor)
                    .span()
                    .span_context()
                    .clone(),
                expected_context
            )
        }

        for ((trace, span, sampled, debug, parent), single_header, expected_context) in
            single_multi_header_extract_data()
        {
            let mut extractor =
                extract_extrator_from_test_data(trace, span, sampled, debug, parent);
            extractor.insert(B3_SINGLE_HEADER.to_string(), single_header.to_owned());
            assert_eq!(
                single_header_propagator
                    .extract(&extractor)
                    .span()
                    .span_context()
                    .clone(),
                expected_context
            );
            assert_eq!(
                single_multi_propagator
                    .extract(&extractor)
                    .span()
                    .span_context()
                    .clone(),
                expected_context
            )
        }

        for invalid_single_header in single_header_extract_invalid_data() {
            let mut extractor = HashMap::new();
            extractor.insert(
                B3_SINGLE_HEADER.to_string(),
                invalid_single_header.to_string(),
            );
            assert_eq!(
                single_header_propagator
                    .extract(&extractor)
                    .span()
                    .span_context(),
                &SpanContext::empty_context(),
            )
        }

        // Test invalid multiple headers
        for (trace, span, sampled, debug, parent) in multi_header_extract_invalid_data() {
            let extractor = extract_extrator_from_test_data(trace, span, sampled, debug, parent);
            assert_eq!(
                multi_header_propagator
                    .extract(&extractor)
                    .span()
                    .span_context(),
                &SpanContext::empty_context(),
            )
        }
    }

    #[test]
    fn inject_b3() {
        let single_header_propagator = Propagator::with_encoding(B3Encoding::SingleHeader);
        let multi_header_propagator = Propagator::with_encoding(B3Encoding::MultipleHeader);
        let single_multi_header_propagator =
            Propagator::with_encoding(B3Encoding::SingleAndMultiHeader);
        let unspecified_header_propagator = Propagator::with_encoding(B3Encoding::UnSpecified);

        for (expected_header, context) in single_header_inject_data() {
            let mut injector = HashMap::new();
            single_header_propagator.inject_context(
                &Context::current_with_span(TestSpan(context)),
                &mut injector,
            );

            assert_eq!(
                injector.get(B3_SINGLE_HEADER),
                Some(&expected_header.to_owned())
            )
        }

        for (trace_id, span_id, sampled, flag, context) in multi_header_inject_data() {
            let mut injector_multi_header = HashMap::new();
            let mut injector_unspecific = HashMap::new();
            multi_header_propagator.inject_context(
                &Context::current_with_span(TestSpan(context.clone())),
                &mut injector_multi_header,
            );
            unspecified_header_propagator.inject_context(
                &Context::current_with_span(TestSpan(context)),
                &mut injector_unspecific,
            );

            assert_eq!(injector_multi_header, injector_unspecific);

            assert_eq!(
                injector_multi_header
                    .get(B3_TRACE_ID_HEADER)
                    .map(|s| s.to_owned()),
                trace_id.map(|s| s.to_string())
            );
            assert_eq!(
                injector_multi_header
                    .get(B3_SPAN_ID_HEADER)
                    .map(|s| s.to_owned()),
                span_id.map(|s| s.to_string())
            );
            assert_eq!(
                injector_multi_header
                    .get(B3_SAMPLED_HEADER)
                    .map(|s| s.to_owned()),
                sampled.map(|s| s.to_string())
            );
            assert_eq!(
                injector_multi_header
                    .get(B3_DEBUG_FLAG_HEADER)
                    .map(|s| s.to_owned()),
                flag.map(|s| s.to_string())
            );
            assert_eq!(injector_multi_header.get(B3_PARENT_SPAN_ID_HEADER), None);
        }

        for (trace_id, span_id, sampled, flag, b3, context) in single_multi_header_inject_data() {
            let mut injector = HashMap::new();
            single_multi_header_propagator.inject_context(
                &Context::current_with_span(TestSpan(context)),
                &mut injector,
            );

            assert_eq!(
                injector.get(B3_TRACE_ID_HEADER).map(|s| s.to_owned()),
                trace_id.map(|s| s.to_string())
            );
            assert_eq!(
                injector.get(B3_SPAN_ID_HEADER).map(|s| s.to_owned()),
                span_id.map(|s| s.to_string())
            );
            assert_eq!(
                injector.get(B3_SAMPLED_HEADER).map(|s| s.to_owned()),
                sampled.map(|s| s.to_string())
            );
            assert_eq!(
                injector.get(B3_DEBUG_FLAG_HEADER).map(|s| s.to_owned()),
                flag.map(|s| s.to_string())
            );
            assert_eq!(
                injector.get(B3_SINGLE_HEADER).map(|s| s.to_owned()),
                b3.map(|s| s.to_string())
            );
            assert_eq!(injector.get(B3_PARENT_SPAN_ID_HEADER), None);
        }
    }

    #[test]
    fn test_get_fields() {
        let single_header_propagator = Propagator::with_encoding(B3Encoding::SingleHeader);
        let multi_header_propagator = Propagator::with_encoding(B3Encoding::MultipleHeader);
        let single_multi_header_propagator =
            Propagator::with_encoding(B3Encoding::SingleAndMultiHeader);

        assert_eq!(
            single_header_propagator.fields().collect::<Vec<&str>>(),
            vec![B3_SINGLE_HEADER]
        );
        assert_eq!(
            multi_header_propagator.fields().collect::<Vec<&str>>(),
            vec![
                B3_TRACE_ID_HEADER,
                B3_SPAN_ID_HEADER,
                B3_SAMPLED_HEADER,
                B3_DEBUG_FLAG_HEADER
            ]
        );
        assert_eq!(
            single_multi_header_propagator
                .fields()
                .collect::<Vec<&str>>(),
            vec![
                B3_SINGLE_HEADER,
                B3_TRACE_ID_HEADER,
                B3_SPAN_ID_HEADER,
                B3_SAMPLED_HEADER,
                B3_DEBUG_FLAG_HEADER
            ]
        );
    }
}
