//! B3 trace context propagation for OpenTelemetry Rust.
//!
//! This crate provides a standalone copy of the B3 implementation in
//! `opentelemetry-zipkin`. Existing users can change their imports to
//! `opentelemetry_propagator_b3::{B3Encoding, Propagator}`.
//!
//! Both single and multiple headers are accepted when extracting context, with
//! a valid single header taking precedence. [`Propagator::new`] injects the
//! single `b3` header, as required by the OpenTelemetry specification; use
//! [`Propagator::with_encoding`] with [`B3Encoding::MultipleHeader`] to inject
//! the multiple `x-b3-*` headers instead.
//!
//! ```
//! use opentelemetry::global;
//! use opentelemetry_propagator_b3::Propagator;
//!
//! global::set_text_map_propagator(Propagator::new());
//! ```

#![warn(missing_docs, unreachable_pub, missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use once_cell::sync::Lazy;
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

// B3 private markers use the high bits so they don't collide with W3C trace
// flags, which are assigned from the low bits.
const TRACE_FLAG_DEFERRED: TraceFlags = TraceFlags::new(0x80);
const TRACE_FLAG_DEBUG: TraceFlags = TraceFlags::new(0x40);

static B3_SINGLE_FIELDS: Lazy<[String; 1]> = Lazy::new(|| [B3_SINGLE_HEADER.to_owned()]);
static B3_MULTI_FIELDS: Lazy<[String; 4]> = Lazy::new(|| {
    [
        B3_TRACE_ID_HEADER.to_owned(),
        B3_SPAN_ID_HEADER.to_owned(),
        B3_SAMPLED_HEADER.to_owned(),
        B3_DEBUG_FLAG_HEADER.to_owned(),
    ]
});
static B3_SINGLE_AND_MULTI_FIELDS: Lazy<[String; 5]> = Lazy::new(|| {
    [
        B3_SINGLE_HEADER.to_owned(),
        B3_TRACE_ID_HEADER.to_owned(),
        B3_SPAN_ID_HEADER.to_owned(),
        B3_SAMPLED_HEADER.to_owned(),
        B3_DEBUG_FLAG_HEADER.to_owned(),
    ]
});

/// B3Encoding is a bitmask to represent B3 encoding type
#[derive(Clone, Debug)]
pub enum B3Encoding {
    /// Unspecified is an unspecified B3 encoding, which injects the single header
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
            inject_encoding: B3Encoding::SingleHeader,
        }
    }
}

impl Propagator {
    /// Create a new `HttpB3Propagator` that uses the single header.
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
        if !is_lower_hex(trace_id) || (trace_id.len() != 16 && trace_id.len() != 32) {
            Err(())
        } else {
            TraceId::from_hex(trace_id).map_err(|_| ())
        }
    }

    /// Extract span id from hex encoded &str value.
    fn extract_span_id(&self, span_id: &str) -> Result<SpanId, ()> {
        // Only allow lower case hex string
        if !is_lower_hex(span_id) || span_id.len() != 16 {
            Err(())
        } else {
            SpanId::from_hex(span_id).map_err(|_| ())
        }
    }

    /// Extract sampled state from the sampling field of the single `b3` header.
    fn extract_single_sampled_state(&self, sampled: &str) -> Result<TraceFlags, ()> {
        match sampled {
            "0" => Ok(TraceFlags::default()),
            "1" => Ok(TraceFlags::SAMPLED),
            "d" => Ok(TRACE_FLAG_DEBUG | TraceFlags::SAMPLED), // debug implies sampled
            _ => Err(()),
        }
    }

    /// Extract sampled state from the `X-B3-Sampled` header.
    /// For legacy support and being lenient to other tracing implementations we
    /// allow "true" and "false" as inputs for interop purposes.
    fn extract_multi_sampled_state(&self, sampled: &str) -> Result<TraceFlags, ()> {
        match sampled {
            "0" | "false" => Ok(TraceFlags::default()),
            "1" | "true" => Ok(TraceFlags::SAMPLED),
            _ => Err(()),
        }
    }

    /// Extract the debug flag from the `X-B3-Flags` header. Only "1" means debug,
    /// absent or other values are ignored.
    fn extract_debug_flag(&self, debug: Option<&str>) -> Option<TraceFlags> {
        match debug {
            Some("1") => Some(TRACE_FLAG_DEBUG | TraceFlags::SAMPLED), // debug implies sampled
            _ => None,
        }
    }

    /// Extract a `SpanContext` from a single B3 header.
    fn extract_single_header(&self, extractor: &dyn Extractor) -> Result<SpanContext, ()> {
        let header_value = extractor.get(B3_SINGLE_HEADER).unwrap_or("");
        let mut parts = header_value.split('-');

        let trace_id = self.extract_trace_id(parts.next().ok_or(())?)?;
        let span_id = self.extract_span_id(parts.next().ok_or(())?)?;
        let trace_flags = match parts.next() {
            Some(sampled) => self.extract_single_sampled_state(sampled)?,
            None => TRACE_FLAG_DEFERRED,
        };

        // Ensure parent id was valid
        if let Some(parent) = parts.next() {
            let _ = self.extract_span_id(parent)?;
        }

        // Ensure there are no more than 4 parts.
        if parts.next().is_some() {
            return Err(());
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
            let _ = self.extract_span_id(parent)?;
        }

        let debug = self.extract_debug_flag(extractor.get(B3_DEBUG_FLAG_HEADER));
        let sampled_opt = extractor.get(B3_SAMPLED_HEADER);

        let flag = if let Some(debug_flag) = debug {
            // if debug is set, then X-B3-Sampled should not be sent. Will ignore
            debug_flag
        } else if let Some(sampled) = sampled_opt {
            // if debug is not set and X-B3-Sampled is not set, then deferred
            // if sample value is invalid, then return empty context.
            self.extract_multi_sampled_state(sampled)?
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

    /// Whether the single header should be injected. If inject_encoding is
    /// Unspecified, default to the single header.
    fn injects_single_header(&self) -> bool {
        matches!(self.inject_encoding, B3Encoding::UnSpecified)
            || self.inject_encoding.support(&B3Encoding::SingleHeader)
    }
}

/// Whether `s` only contains lower case hex characters.
fn is_lower_hex(s: &str) -> bool {
    s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// B3 sampling state for `flags`: `"d"` (debug), `"1"` (sampled), `"0"`
/// (not sampled), or `None` when the decision is deferred. Bits that B3 has no
/// encoding for, such as the W3C random-trace-id flag, are ignored.
fn b3_sampling_state(flags: TraceFlags) -> Option<&'static str> {
    if flags & TRACE_FLAG_DEBUG == TRACE_FLAG_DEBUG {
        Some("d")
    } else if flags & TRACE_FLAG_DEFERRED == TRACE_FLAG_DEFERRED {
        None
    } else if flags.is_sampled() {
        Some("1")
    } else {
        Some("0")
    }
}

impl TextMapPropagator for Propagator {
    /// Properly encodes the values of the `Context`'s `SpanContext` and injects
    /// them into the `Injector`.
    fn inject_context(&self, context: &Context, injector: &mut dyn Injector) {
        let span = context.span();
        let span_context = span.span_context();
        if span_context.is_valid() {
            let sampling_state = b3_sampling_state(span_context.trace_flags());
            if self.injects_single_header() {
                let value = match sampling_state {
                    Some(state) => format!(
                        "{}-{}-{state}",
                        span_context.trace_id(),
                        span_context.span_id()
                    ),
                    None => format!("{}-{}", span_context.trace_id(), span_context.span_id()),
                };

                injector.set(B3_SINGLE_HEADER, value);
            }
            if self.inject_encoding.support(&B3Encoding::MultipleHeader) {
                injector.set(B3_TRACE_ID_HEADER, span_context.trace_id().to_string());
                injector.set(B3_SPAN_ID_HEADER, span_context.span_id().to_string());

                match sampling_state {
                    Some("d") => injector.set(B3_DEBUG_FLAG_HEADER, "1".to_string()),
                    Some(sampled) => injector.set(B3_SAMPLED_HEADER, sampled.to_string()),
                    None => {}
                }
            }
        } else {
            let flag = if span_context.is_sampled() { "1" } else { "0" };
            if self.injects_single_header() {
                injector.set(B3_SINGLE_HEADER, flag.to_string())
            }
            if self.inject_encoding.support(&B3Encoding::MultipleHeader) {
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
        } else {
            // SingleHeader, or Unspecified which defaults to the single header.
            B3_SINGLE_FIELDS.as_ref()
        };

        FieldIter::new(field_slice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::testing::trace::TestSpan;
    use opentelemetry_sdk::propagation::TraceContextPropagator;
    use std::collections::HashMap;

    const TRACE_ID_STR: &str = "4bf92f3577b34da6a3ce929d0e0e4736";
    const SPAN_ID_STR: &str = "00f067aa0ba902b7";
    const TRACE_ID_HEX: u128 = 0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736;
    const SPAN_ID_HEX: u64 = 0x00f0_67aa_0ba9_02b7;

    #[rustfmt::skip]
    fn single_header_extract_data() -> Vec<(&'static str, SpanContext)> {
        vec![
            ("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7", SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TRACE_FLAG_DEFERRED, true, TraceState::default())), // deferred
            ("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-0", SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TraceFlags::default(), true, TraceState::default())), // not sampled
            ("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-1", SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TraceFlags::SAMPLED, true, TraceState::default())), // sampled
            ("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-d", SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TRACE_FLAG_DEBUG | TraceFlags::SAMPLED, true, TraceState::default())), // debug implies sampled
            ("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-1-00000000000000cd", SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TraceFlags::SAMPLED, true, TraceState::default())), // with parent span id
            ("a3ce929d0e0e4736-00f067aa0ba902b7-1-00000000000000cd", SpanContext::new(TraceId::from(0x0000_0000_0000_0000_a3ce_929d_0e0e_4736), SpanId::from(SPAN_ID_HEX), TraceFlags::SAMPLED, true, TraceState::default())), // padding 64 bit traceID
            ("0", SpanContext::empty_context()),
            ("-", SpanContext::empty_context()),
        ]
    }

    #[rustfmt::skip]
    #[allow(clippy::type_complexity)]
    fn multi_header_extract_data() -> Vec<((Option<&'static str>, Option<&'static str>, Option<&'static str>, Option<&'static str>, Option<&'static str>), SpanContext)> {
        // (TraceId, SpanId, Sampled, FlagId, ParentSpanId)
        vec![
            ((Some(TRACE_ID_STR), Some(SPAN_ID_STR), None, None, None), SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TRACE_FLAG_DEFERRED, true, TraceState::default())), // deferred
            ((Some(TRACE_ID_STR), Some(SPAN_ID_STR), Some("0"), None, None), SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TraceFlags::default(), true, TraceState::default())), // not sampled
            ((Some(TRACE_ID_STR), Some(SPAN_ID_STR), Some("1"), None, None), SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TraceFlags::SAMPLED, true, TraceState::default())), // sampled
            ((Some(TRACE_ID_STR), Some(SPAN_ID_STR), Some("true"), None, None), SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TraceFlags::SAMPLED, true, TraceState::default())),
            ((Some(TRACE_ID_STR), Some(SPAN_ID_STR), Some("false"), None, None), SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TraceFlags::default(), true, TraceState::default())), // use true/false to set sample
            ((Some(TRACE_ID_STR), Some(SPAN_ID_STR), None, Some("1"), None), SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TRACE_FLAG_DEBUG | TraceFlags::SAMPLED, true, TraceState::default())), // debug
            ((Some(TRACE_ID_STR), Some(SPAN_ID_STR), Some("0"), Some("1"), Some("00f067aa0ba90200")), SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TRACE_FLAG_DEBUG | TraceFlags::SAMPLED, true, TraceState::default())),  // debug flag should override sample flag
            ((Some(TRACE_ID_STR), Some(SPAN_ID_STR), Some("1"), Some("2"), Some("00f067aa0ba90200")), SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TraceFlags::SAMPLED, true, TraceState::default())), // invalid debug flag, should ignore
            ((Some(TRACE_ID_STR), Some(SPAN_ID_STR), Some("1"), Some("0"), None), SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TraceFlags::SAMPLED, true, TraceState::default())), // debug flag 0 should not override sample flag
            ((Some(TRACE_ID_STR), Some(SPAN_ID_STR), None, Some("0"), None), SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TRACE_FLAG_DEFERRED, true, TraceState::default())), // debug flag 0 without sample flag is deferred
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
            "ab000000000000000000000000000000-cd00000000000000-false", // invalid sample flag(set to false)
            "ab000000000000000000000000000000-cd00000000000000-", // trailing separator
            "ab000000000000000000000000000000-cd00000000000000-1-", // trailing separator after sample flag
            "+b000000000000000000000000000000-cd00000000000000-1", // trace id with sign prefix
            "ab000000000000000000000000000000-+d00000000000000-1", // span id with sign prefix
            "ab000000000000000000000000000000-cd00000000000000-1-ef00000000000000-ef00000000000000", // too many parts
            "ab000000000000000000000000000000", // missing span id
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
            (Some("+bf92f3577b34da6a3ce929d0e0e4736"), Some(SPAN_ID_STR), Some("1"), None, None), // trace id with sign prefix
            (Some(TRACE_ID_STR), Some("+0f067aa0ba902b7"), Some("1"), None, None), // span id with sign prefix
            (Some(TRACE_ID_STR), Some(SPAN_ID_STR), Some("1"), None, Some("00f067aa0ba9020")), // parent span id length is wrong
            (Some(TRACE_ID_STR), Some(SPAN_ID_STR), Some("1"), None, Some("00F067AA0BA90200")), // parent span id is upper case hex string
        ]
    }

    #[rustfmt::skip]
    #[allow(clippy::type_complexity)]
    fn single_multi_header_extract_data() -> Vec<((Option<&'static str>, Option<&'static str>, Option<&'static str>, Option<&'static str>, Option<&'static str>), &'static str, SpanContext)> {
        // (TraceId, SpanId, Sampled, FlagId, ParentSpanId), b3
        vec![
            ((Some(TRACE_ID_STR), Some(SPAN_ID_STR), None, None, None), "4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-0",
             SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TraceFlags::default(), true, TraceState::default())), // single header take precedence
            ((Some(TRACE_ID_STR), Some(SPAN_ID_STR), Some("0"), None, None), "-", SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TraceFlags::default(), true, TraceState::default())), // when single header is invalid, fall back to multiple headers
            ((Some("0"), Some("0"), Some("0"), None, None), "4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-0", SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TraceFlags::default(), true, TraceState::default())) // invalid multiple header should go unnoticed since single header take precedence.
        ]
    }

    #[rustfmt::skip]
    fn single_header_inject_data() -> Vec<(&'static str, SpanContext)> {
        vec![
            ("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-1", SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TraceFlags::SAMPLED, true, TraceState::default())),
            ("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-1", SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TraceFlags::SAMPLED | TraceFlags::RANDOM, true, TraceState::default())),
            ("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-d", SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TRACE_FLAG_DEBUG, true, TraceState::default())),
            ("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7", SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TRACE_FLAG_DEFERRED, true, TraceState::default())),
            ("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-0", SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TraceFlags::default(), true, TraceState::default())),
            ("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-0", SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TraceFlags::RANDOM, true, TraceState::default())),
            ("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-d", SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TRACE_FLAG_DEBUG | TraceFlags::SAMPLED | TraceFlags::RANDOM, true, TraceState::default())),
            ("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7", SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TRACE_FLAG_DEFERRED | TraceFlags::RANDOM, true, TraceState::default())),
            ("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-d", SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TRACE_FLAG_DEFERRED | TRACE_FLAG_DEBUG, true, TraceState::default())),
            ("1", SpanContext::new(TraceId::INVALID, SpanId::INVALID, TraceFlags::SAMPLED, true, TraceState::default())),
            ("1", SpanContext::new(TraceId::INVALID, SpanId::INVALID, TraceFlags::SAMPLED | TraceFlags::RANDOM, true, TraceState::default())),
            ("0", SpanContext::new(TraceId::INVALID, SpanId::INVALID, TraceFlags::default(), true, TraceState::default())),
        ]
    }

    #[rustfmt::skip]
    #[allow(clippy::type_complexity)]
    fn multi_header_inject_data() -> Vec<(Option<&'static str>, Option<&'static str>, Option<&'static str>, Option<&'static str>, SpanContext)> {
        // TraceId, SpanId, isSampled, isDebug
        vec![
            (Some(TRACE_ID_STR), Some(SPAN_ID_STR), Some("1"), None, SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TraceFlags::SAMPLED, true, TraceState::default())),
            (Some(TRACE_ID_STR), Some(SPAN_ID_STR), Some("1"), None, SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TraceFlags::SAMPLED | TraceFlags::RANDOM, true, TraceState::default())),
            (Some(TRACE_ID_STR), Some(SPAN_ID_STR), None, Some("1"), SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TRACE_FLAG_DEBUG, true, TraceState::default())),
            (Some(TRACE_ID_STR), Some(SPAN_ID_STR), None, None, SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TRACE_FLAG_DEFERRED, true, TraceState::default())),
            (Some(TRACE_ID_STR), Some(SPAN_ID_STR), Some("0"), None, SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TraceFlags::default(), true, TraceState::default())),
            (Some(TRACE_ID_STR), Some(SPAN_ID_STR), Some("0"), None, SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TraceFlags::RANDOM, true, TraceState::default())),
            (Some(TRACE_ID_STR), Some(SPAN_ID_STR), None, Some("1"), SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TRACE_FLAG_DEBUG | TraceFlags::SAMPLED | TraceFlags::RANDOM, true, TraceState::default())),
            (Some(TRACE_ID_STR), Some(SPAN_ID_STR), None, None, SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TRACE_FLAG_DEFERRED | TraceFlags::RANDOM, true, TraceState::default())),
            (Some(TRACE_ID_STR), Some(SPAN_ID_STR), None, Some("1"), SpanContext::new(TraceId::from(TRACE_ID_HEX), SpanId::from(SPAN_ID_HEX), TRACE_FLAG_DEFERRED | TRACE_FLAG_DEBUG, true, TraceState::default())),
            (None, None, Some("0"), None, SpanContext::empty_context()),
            (None, None, Some("1"), None, SpanContext::new(TraceId::INVALID, SpanId::INVALID, TraceFlags::SAMPLED, true, TraceState::default())),
            (None, None, Some("1"), None, SpanContext::new(TraceId::INVALID, SpanId::INVALID, TraceFlags::SAMPLED | TraceFlags::RANDOM, true, TraceState::default()))
        ]
    }

    #[rustfmt::skip]
    #[allow(clippy::type_complexity)]
    fn single_multi_header_inject_data() -> Vec<(Option<&'static str>, Option<&'static str>, Option<&'static str>, Option<&'static str>, Option<&'static str>, SpanContext)> {
        let trace_id: TraceId = TraceId::from(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736);
        let span_id: SpanId = SpanId::from(0x00f0_67aa_0ba9_02b7);
        vec![
            (Some(TRACE_ID_STR), Some(SPAN_ID_STR), Some("1"), None, Some("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-1"), SpanContext::new(trace_id, span_id, TraceFlags::SAMPLED, true, TraceState::default())), // sampled
            (Some(TRACE_ID_STR), Some(SPAN_ID_STR), None, Some("1"), Some("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-d"), SpanContext::new(trace_id, span_id, TRACE_FLAG_DEBUG, true, TraceState::default())), // debug
            (Some(TRACE_ID_STR), Some(SPAN_ID_STR), Some("0"), None, Some("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-0"), SpanContext::new(trace_id, span_id, TraceFlags::default(), true, TraceState::default())), // not sampled
            (Some(TRACE_ID_STR), Some(SPAN_ID_STR), None, None, Some("4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7"), SpanContext::new(trace_id, span_id, TRACE_FLAG_DEFERRED, true, TraceState::default())), // unset sampled
            (None, None, Some("0"), None, Some("0"), SpanContext::empty_context()),
            (None, None, Some("1"), None, Some("1"), SpanContext::new(TraceId::INVALID, SpanId::INVALID, TraceFlags::SAMPLED, true, TraceState::default())),
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

        // Extraction must not depend on the inject encoding.
        let all_propagators = [
            &single_header_propagator,
            &multi_header_propagator,
            &single_multi_propagator,
            &unspecific_header_propagator,
        ];

        for ((trace, span, sampled, debug, parent), expected_context) in multi_header_extract_data()
        {
            let extractor = extract_extrator_from_test_data(trace, span, sampled, debug, parent);
            for propagator in all_propagators {
                assert_eq!(
                    propagator.extract(&extractor).span().span_context().clone(),
                    expected_context,
                    "{propagator:?} {extractor:?}"
                );
            }
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
            for propagator in all_propagators {
                assert_eq!(
                    propagator.extract(&extractor).span().span_context(),
                    &SpanContext::empty_context(),
                    "{propagator:?} {extractor:?}"
                );
            }
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
            let mut injector_unspecific = HashMap::new();
            single_header_propagator.inject_context(
                &Context::current_with_span(TestSpan(context.clone())),
                &mut injector,
            );
            unspecified_header_propagator.inject_context(
                &Context::current_with_span(TestSpan(context)),
                &mut injector_unspecific,
            );

            assert_eq!(injector, injector_unspecific);
            assert_eq!(
                injector.get(B3_SINGLE_HEADER),
                Some(&expected_header.to_owned())
            );
            // Single header encoding must not inject multiple headers.
            assert_eq!(injector.len(), 1, "{injector:?}");
        }

        for (trace_id, span_id, sampled, flag, context) in multi_header_inject_data() {
            let mut injector_multi_header = HashMap::new();
            multi_header_propagator.inject_context(
                &Context::current_with_span(TestSpan(context)),
                &mut injector_multi_header,
            );

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
    fn default_injects_single_header() {
        let context = SpanContext::new(
            TraceId::from(TRACE_ID_HEX),
            SpanId::from(SPAN_ID_HEX),
            TraceFlags::SAMPLED,
            true,
            TraceState::default(),
        );

        for propagator in [Propagator::new(), Propagator::default()] {
            let mut injector = HashMap::new();
            propagator.inject_context(
                &Context::current_with_span(TestSpan(context.clone())),
                &mut injector,
            );

            assert_eq!(
                injector,
                HashMap::from([(
                    B3_SINGLE_HEADER.to_string(),
                    format!("{TRACE_ID_STR}-{SPAN_ID_STR}-1")
                )])
            );
        }
    }

    #[test]
    fn test_get_fields() {
        let single_header_propagator = Propagator::with_encoding(B3Encoding::SingleHeader);
        let multi_header_propagator = Propagator::with_encoding(B3Encoding::MultipleHeader);
        let single_multi_header_propagator =
            Propagator::with_encoding(B3Encoding::SingleAndMultiHeader);

        for propagator in [
            single_header_propagator,
            Propagator::with_encoding(B3Encoding::UnSpecified),
            Propagator::new(),
        ] {
            assert_eq!(
                propagator.fields().collect::<Vec<&str>>(),
                vec![B3_SINGLE_HEADER]
            );
        }
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
    #[test]
    fn w3c_random_flag_injects_sampled_state_via_b3() {
        let mut headers = HashMap::new();
        headers.insert(
            "traceparent".to_string(),
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-03".to_string(),
        );
        let cx = TraceContextPropagator::new().extract(&headers);

        let mut single = HashMap::new();
        Propagator::with_encoding(B3Encoding::SingleHeader).inject_context(&cx, &mut single);
        assert_eq!(
            single.get(B3_SINGLE_HEADER),
            Some(&"4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-1".to_owned())
        );

        let mut multi = HashMap::new();
        Propagator::with_encoding(B3Encoding::MultipleHeader).inject_context(&cx, &mut multi);
        assert_eq!(multi.get(B3_SAMPLED_HEADER), Some(&"1".to_owned()));
        assert_eq!(multi.get(B3_DEBUG_FLAG_HEADER), None);
    }

    #[test]
    fn local_sampling_decision_clears_b3_deferred_marker() {
        use opentelemetry::trace::{Span, Tracer, TracerProvider};
        use opentelemetry_sdk::trace::{Sampler, SdkTracerProvider};

        let mut incoming = HashMap::new();
        incoming.insert(
            B3_SINGLE_HEADER.to_string(),
            "4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7".to_string(),
        );
        let parent_cx = Propagator::with_encoding(B3Encoding::SingleHeader).extract(&incoming);
        assert_eq!(
            parent_cx.span().span_context().trace_flags(),
            TRACE_FLAG_DEFERRED
        );

        for (sampler, sampled) in [(Sampler::AlwaysOff, false), (Sampler::AlwaysOn, true)] {
            let provider = SdkTracerProvider::builder().with_sampler(sampler).build();
            let child = provider
                .tracer("test")
                .start_with_context("child", &parent_cx);
            assert_eq!(child.is_recording(), sampled);
            let span_id = child.span_context().span_id().to_string();
            let child_cx = Context::current_with_span(child);
            let sampled_flag = if sampled { "1" } else { "0" };

            let mut single = HashMap::new();
            Propagator::with_encoding(B3Encoding::SingleHeader)
                .inject_context(&child_cx, &mut single);
            assert_eq!(
                single.get(B3_SINGLE_HEADER),
                Some(&format!("{TRACE_ID_STR}-{span_id}-{sampled_flag}"))
            );

            let mut multi = HashMap::new();
            Propagator::with_encoding(B3Encoding::MultipleHeader)
                .inject_context(&child_cx, &mut multi);
            assert_eq!(
                multi.get(B3_SAMPLED_HEADER).map(String::as_str),
                Some(sampled_flag)
            );
            assert_eq!(multi.get(B3_DEBUG_FLAG_HEADER), None);
        }
    }

    #[test]
    fn b3_private_flags_stay_off_traceparent() {
        let w3c = TraceContextPropagator::new();

        // Deferred sampling has no sampling state on the wire.
        let mut deferred = HashMap::new();
        deferred.insert(
            B3_SINGLE_HEADER.to_string(),
            "4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7".to_string(),
        );
        let cx = Propagator::with_encoding(B3Encoding::SingleHeader).extract(&deferred);
        assert_eq!(cx.span().span_context().trace_flags(), TRACE_FLAG_DEFERRED);
        let mut out = HashMap::new();
        w3c.inject_context(&cx, &mut out);
        assert_eq!(
            out.get("traceparent"),
            Some(&"00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-00".to_owned())
        );

        // Debug sampling is indicated by x-b3-flags set to 1.
        let mut debug = HashMap::new();
        debug.insert(B3_TRACE_ID_HEADER.to_string(), TRACE_ID_STR.to_string());
        debug.insert(B3_SPAN_ID_HEADER.to_string(), SPAN_ID_STR.to_string());
        debug.insert(B3_DEBUG_FLAG_HEADER.to_string(), "1".to_string());
        let cx = Propagator::with_encoding(B3Encoding::MultipleHeader).extract(&debug);
        assert_eq!(
            cx.span().span_context().trace_flags(),
            TRACE_FLAG_DEBUG | TraceFlags::SAMPLED
        );
        let mut out = HashMap::new();
        w3c.inject_context(&cx, &mut out);
        assert_eq!(
            out.get("traceparent"),
            Some(&"00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01".to_owned())
        );
    }
}
