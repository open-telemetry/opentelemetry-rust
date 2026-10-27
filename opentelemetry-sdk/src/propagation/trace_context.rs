//! # W3C Trace Context Propagator
//!

use opentelemetry::{
    propagation::{text_map_propagator::FieldIter, Extractor, Injector, TextMapPropagator},
    trace::{SpanContext, SpanId, TraceContextExt, TraceFlags, TraceId, TraceState},
    Context,
};
use std::str::FromStr;
use std::sync::OnceLock;

const SUPPORTED_VERSION: u8 = 0;
const INVALID_VERSION: u8 = 0xff;
const TRACEPARENT_HEADER: &str = "traceparent";
// Length of a version 00 `traceparent`: `vv-<32 hex>-<16 hex>-ff`. Higher versions
// share this prefix and may only append fields after another `-`.
const TRACEPARENT_LEN: usize = 55;
const VERSION_RANGE: std::ops::Range<usize> = 0..2;
const TRACE_ID_RANGE: std::ops::Range<usize> = 3..35;
const SPAN_ID_RANGE: std::ops::Range<usize> = 36..52;
const TRACE_FLAGS_RANGE: std::ops::Range<usize> = 53..55;
const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";
const TRACESTATE_HEADER: &str = "tracestate";

// TODO Replace this with LazyLock once it is stable.
static TRACE_CONTEXT_HEADER_FIELDS: OnceLock<[String; 2]> = OnceLock::new();

fn trace_context_header_fields() -> &'static [String; 2] {
    TRACE_CONTEXT_HEADER_FIELDS
        .get_or_init(|| [TRACEPARENT_HEADER.to_owned(), TRACESTATE_HEADER.to_owned()])
}

/// Propagates `SpanContext`s in [W3C TraceContext] format under `traceparent` and `tracestate` header.
///
/// The `traceparent` header represents the incoming request in a
/// tracing system in a common format, understood by all vendors.
/// Here’s an example of a `traceparent` header.
///
/// `traceparent: 00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01`
///
/// The `traceparent` HTTP header field identifies the incoming request in a
/// tracing system. It has four fields:
///
///    - version
///    - trace-id
///    - parent-id
///    - trace-flags
///
/// The `tracestate` header provides additional vendor-specific trace
/// identification information across different distributed tracing systems.
/// Here's an example of a `tracestate` header
///
/// `tracestate: vendorname1=opaqueValue1,vendorname2=opaqueValue2`
///
/// The `trace-flags` field carries the `sampled` and `random-trace-id` flags;
/// all other bits are zeroed on extract and inject as the specification requires.
///
/// See the [w3c trace-context docs] for more details.
///
/// [w3c trace-context docs]: https://w3c.github.io/trace-context/
/// [W3C TraceContext]: https://www.w3.org/TR/trace-context/
#[derive(Clone, Debug, Default)]
pub struct TraceContextPropagator {
    _private: (),
}

impl TraceContextPropagator {
    /// Create a new `TraceContextPropagator`.
    pub fn new() -> Self {
        TraceContextPropagator { _private: () }
    }

    /// Extract span context from w3c trace-context header.
    ///
    /// Parsing follows <https://www.w3.org/TR/trace-context-2/#traceparent-header>,
    /// including the rules for parsing higher versions.
    fn extract_span_context(&self, extractor: &dyn Extractor) -> Result<SpanContext, ()> {
        // A request carrying several traceparent fields is ambiguous, so restart the trace.
        let header_value = match extractor.get_all(TRACEPARENT_HEADER).as_deref() {
            Some([value]) => *value,
            _ => return Err(()),
        };
        let header = header_value
            .trim_matches(|c| c == ' ' || c == '\t')
            .as_bytes();

        // Every version starts with the same fixed width, dash separated fields.
        if header.len() < TRACEPARENT_LEN
            || header[VERSION_RANGE.end] != b'-'
            || header[TRACE_ID_RANGE.end] != b'-'
            || header[SPAN_ID_RANGE.end] != b'-'
        {
            return Err(());
        }

        let version = decode_lower_hex(&header[VERSION_RANGE]).ok_or(())? as u8;
        if version == INVALID_VERSION {
            return Err(());
        }

        // Version 00 has exactly four fields, higher versions may only append
        // fields after another dash.
        if header.len() > TRACEPARENT_LEN
            && (version == SUPPORTED_VERSION || header[TRACEPARENT_LEN] != b'-')
        {
            return Err(());
        }

        let trace_id = TraceId::from(decode_lower_hex(&header[TRACE_ID_RANGE]).ok_or(())?);
        let span_id = SpanId::from(decode_lower_hex(&header[SPAN_ID_RANGE]).ok_or(())? as u64);
        let opts = decode_lower_hex(&header[TRACE_FLAGS_RANGE]).ok_or(())? as u8;

        // Build trace flags clearing all flags other than the trace-context
        // defined sampled and random-trace-id bits. Unknown flags are accepted
        // but zeroed out, as required by
        // https://www.w3.org/TR/trace-context-2/#other-flags
        let trace_flags = TraceFlags::new(opts) & (TraceFlags::SAMPLED | TraceFlags::RANDOM);

        // Ensure trace id and span id are valid before looking at tracestate, which
        // must not be parsed when traceparent is invalid.
        if trace_id == TraceId::INVALID || span_id == SpanId::INVALID {
            return Err(());
        }

        // Multiple tracestate header fields are combined into a single list, and an
        // invalid tracestate is discarded without affecting the traceparent.
        let trace_state = extractor
            .get_all(TRACESTATE_HEADER)
            .and_then(|values| TraceState::from_str(&values.join(",")).ok())
            .unwrap_or_default();

        Ok(SpanContext::new(
            trace_id,
            span_id,
            trace_flags,
            true,
            trace_state,
        ))
    }
}

impl TextMapPropagator for TraceContextPropagator {
    /// Properly encodes the values of the `SpanContext` and injects them
    /// into the `Injector`.
    fn inject_context(&self, cx: &Context, injector: &mut dyn Injector) {
        let span = cx.span();
        let span_context = span.span_context();
        if span_context.is_valid() {
            let trace_flags =
                span_context.trace_flags() & (TraceFlags::SAMPLED | TraceFlags::RANDOM);
            let mut header_value = String::with_capacity(TRACEPARENT_LEN);
            push_lower_hex(&mut header_value, &[SUPPORTED_VERSION]);
            header_value.push('-');
            push_lower_hex(&mut header_value, &span_context.trace_id().to_bytes());
            header_value.push('-');
            push_lower_hex(&mut header_value, &span_context.span_id().to_bytes());
            header_value.push('-');
            push_lower_hex(&mut header_value, &[trace_flags.to_u8()]);
            injector.set(TRACEPARENT_HEADER, header_value);

            // Avoid sending empty tracestate headers.
            let trace_state = span_context.trace_state().header();
            if !trace_state.is_empty() {
                injector.set(TRACESTATE_HEADER, trace_state);
            }
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
        FieldIter::new(trace_context_header_fields())
    }
}

/// Decodes lowercase hex digits (`0-9a-f`), returning `None` on any other byte.
///
/// Callers pass at most 32 digits, so the result cannot overflow.
fn decode_lower_hex(bytes: &[u8]) -> Option<u128> {
    bytes.iter().try_fold(0u128, |acc, &b| {
        let nibble = match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'f' => b - b'a' + 10,
            _ => return None,
        };
        Some(acc << 4 | u128::from(nibble))
    })
}

/// Appends `bytes` to `out` as lowercase hex, two digits per byte.
fn push_lower_hex(out: &mut String, bytes: &[u8]) {
    for &b in bytes {
        out.push(HEX_DIGITS[usize::from(b >> 4)] as char);
        out.push(HEX_DIGITS[usize::from(b & 0x0f)] as char);
    }
}

#[cfg(all(test, feature = "testing", feature = "trace"))]
mod tests {
    use super::*;
    use crate::testing::trace::TestSpan;
    use std::collections::HashMap;

    #[rustfmt::skip]
    fn extract_data() -> Vec<(&'static str, &'static str, SpanContext)> {
        vec![
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-00", "foo=bar", SpanContext::new(TraceId::from(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from(0x00f0_67aa_0ba9_02b7), TraceFlags::default(), true, TraceState::from_str("foo=bar").unwrap())),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01", "foo=bar", SpanContext::new(TraceId::from(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from(0x00f0_67aa_0ba9_02b7), TraceFlags::SAMPLED, true, TraceState::from_str("foo=bar").unwrap())),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-02", "foo=bar", SpanContext::new(TraceId::from(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from(0x00f0_67aa_0ba9_02b7), TraceFlags::RANDOM, true, TraceState::from_str("foo=bar").unwrap())),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-03", "foo=bar", SpanContext::new(TraceId::from(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from(0x00f0_67aa_0ba9_02b7), TraceFlags::SAMPLED | TraceFlags::RANDOM, true, TraceState::from_str("foo=bar").unwrap())),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-07", "foo=bar", SpanContext::new(TraceId::from(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from(0x00f0_67aa_0ba9_02b7), TraceFlags::SAMPLED | TraceFlags::RANDOM, true, TraceState::from_str("foo=bar").unwrap())),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-fe", "foo=bar", SpanContext::new(TraceId::from(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from(0x00f0_67aa_0ba9_02b7), TraceFlags::RANDOM, true, TraceState::from_str("foo=bar").unwrap())),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-09", "foo=bar", SpanContext::new(TraceId::from(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from(0x00f0_67aa_0ba9_02b7), TraceFlags::SAMPLED, true, TraceState::from_str("foo=bar").unwrap())),
            ("02-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01", "foo=bar", SpanContext::new(TraceId::from(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from(0x00f0_67aa_0ba9_02b7), TraceFlags::SAMPLED, true, TraceState::from_str("foo=bar").unwrap())),
            ("02-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-09", "foo=bar", SpanContext::new(TraceId::from(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from(0x00f0_67aa_0ba9_02b7), TraceFlags::SAMPLED, true, TraceState::from_str("foo=bar").unwrap())),
            ("02-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-08", "foo=bar", SpanContext::new(TraceId::from(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from(0x00f0_67aa_0ba9_02b7), TraceFlags::default(), true, TraceState::from_str("foo=bar").unwrap())),
            ("02-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-09-XYZxsf09", "foo=bar", SpanContext::new(TraceId::from(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from(0x00f0_67aa_0ba9_02b7), TraceFlags::SAMPLED, true, TraceState::from_str("foo=bar").unwrap())),
            ("01-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-09-", "foo=bar", SpanContext::new(TraceId::from(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from(0x00f0_67aa_0ba9_02b7), TraceFlags::SAMPLED, true, TraceState::from_str("foo=bar").unwrap())),
            ("cc-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01", "foo=bar", SpanContext::new(TraceId::from(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from(0x00f0_67aa_0ba9_02b7), TraceFlags::SAMPLED, true, TraceState::from_str("foo=bar").unwrap())),
            ("cc-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01-what-the-future-will-be-like", "foo=bar", SpanContext::new(TraceId::from(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from(0x00f0_67aa_0ba9_02b7), TraceFlags::SAMPLED, true, TraceState::from_str("foo=bar").unwrap())),
            (" \t00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01\t ", "foo=bar", SpanContext::new(TraceId::from(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from(0x00f0_67aa_0ba9_02b7), TraceFlags::SAMPLED, true, TraceState::from_str("foo=bar").unwrap())),
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
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7",      "missing options"),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-",     "empty options"),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01-",  "version 00 with trailing dash"),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01-what-the-future-will-be-like", "version 00 with extra fields"),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01.",  "version 00 with trailing data"),
            ("0000-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01", "version too long with valid ids"),
            ("0-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",    "version too short"),
            ("ff-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",   "forbidden version ff"),
            ("+0-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",   "signed version"),
            ("00-4bf92f3577b34da6a3ce929d0e0e473-600f067aa0ba902b7-01",   "short trace ID"),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b-701",   "short span ID"),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-1",    "short trace flags"),
            ("00-+bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",   "signed trace ID"),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-+0f067aa0ba902b7-01",   "signed span ID"),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-+1",   "signed trace flags"),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-0000000000000000-01",   "zero span ID"),
            ("00-00000000000000000000000000000000-00f067aa0ba902b7-01",   "zero trace ID"),
            ("00_4bf92f3577b34da6a3ce929d0e0e4736_00f067aa0ba902b7_01",   "wrong delimiter"),
            ("cc-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01.what-the-future-will-be-like", "future version without dash after flags"),
            ("cc-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-0",    "future version too short"),
            ("\n00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01\n", "non-OWS whitespace"),
            ("\u{a0}00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01", "non-breaking space"),
            ("",                                                          "empty header"),
        ]
    }

    #[rustfmt::skip]
    fn inject_data() -> Vec<(&'static str, &'static str, SpanContext)> {
        vec![
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01", "foo=bar", SpanContext::new(TraceId::from(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from(0x00f0_67aa_0ba9_02b7), TraceFlags::SAMPLED, true, TraceState::from_str("foo=bar").unwrap())),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-00", "foo=bar", SpanContext::new(TraceId::from(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from(0x00f0_67aa_0ba9_02b7), TraceFlags::default(), true, TraceState::from_str("foo=bar").unwrap())),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-02", "foo=bar", SpanContext::new(TraceId::from(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from(0x00f0_67aa_0ba9_02b7), TraceFlags::RANDOM, true, TraceState::from_str("foo=bar").unwrap())),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-03", "foo=bar", SpanContext::new(TraceId::from(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from(0x00f0_67aa_0ba9_02b7), TraceFlags::SAMPLED | TraceFlags::RANDOM, true, TraceState::from_str("foo=bar").unwrap())),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-03", "foo=bar", SpanContext::new(TraceId::from(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from(0x00f0_67aa_0ba9_02b7), TraceFlags::new(0xff), true, TraceState::from_str("foo=bar").unwrap())),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-02", "foo=bar", SpanContext::new(TraceId::from(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from(0x00f0_67aa_0ba9_02b7), TraceFlags::new(0x06), true, TraceState::from_str("foo=bar").unwrap())),
            ("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-02", "foo=bar", SpanContext::new(TraceId::from(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736), SpanId::from(0x00f0_67aa_0ba9_02b7), TraceFlags::new(0x82), true, TraceState::from_str("foo=bar").unwrap())),
            ("", "", SpanContext::empty_context()),
        ]
    }

    #[test]
    fn extract_w3c() {
        let propagator = TraceContextPropagator::new();

        for (trace_parent, trace_state, expected_context) in extract_data() {
            let mut extractor = HashMap::new();
            extractor.insert(TRACEPARENT_HEADER.to_string(), trace_parent.to_string());
            extractor.insert(TRACESTATE_HEADER.to_string(), trace_state.to_string());

            assert_eq!(
                propagator.extract(&extractor).span().span_context(),
                &expected_context
            )
        }
    }

    #[test]
    fn extract_w3c_tracestate() {
        let propagator = TraceContextPropagator::new();
        let state = "foo=bar".to_string();
        let parent = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-00".to_string();

        let mut extractor = HashMap::new();
        extractor.insert(TRACEPARENT_HEADER.to_string(), parent);
        extractor.insert(TRACESTATE_HEADER.to_string(), state.clone());

        assert_eq!(
            propagator
                .extract(&extractor)
                .span()
                .span_context()
                .trace_state()
                .header(),
            state
        )
    }

    #[test]
    fn extract_w3c_discards_invalid_tracestate() {
        let propagator = TraceContextPropagator::new();
        let parent = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";

        for state in [
            "=x",
            "a=",
            "a=x\ty",
            "a==b",
            "foo=bar,=x",
            "congo=1,rojo=2,congo=3",
            "nokeyvalue",
        ] {
            let mut extractor = HashMap::new();
            extractor.insert(TRACEPARENT_HEADER.to_string(), parent.to_string());
            extractor.insert(TRACESTATE_HEADER.to_string(), state.to_string());

            let cx = propagator.extract(&extractor);
            let span_context = cx.span().span_context().clone();

            assert!(span_context.is_valid(), "{state:?}");
            assert_eq!(
                span_context.trace_state(),
                &TraceState::default(),
                "{state:?}"
            );
        }
    }

    /// An extractor that may hold several fields for the same header.
    struct MultiValueExtractor(HashMap<String, Vec<String>>);

    impl Extractor for MultiValueExtractor {
        fn get(&self, key: &str) -> Option<&str> {
            self.0
                .get(key)
                .and_then(|values| values.first())
                .map(String::as_str)
        }

        fn keys(&self) -> Vec<&str> {
            self.0.keys().map(String::as_str).collect()
        }

        fn get_all(&self, key: &str) -> Option<Vec<&str>> {
            self.0
                .get(key)
                .map(|values| values.iter().map(String::as_str).collect())
        }
    }

    #[test]
    fn extract_w3c_combines_multiple_tracestate_fields() {
        let propagator = TraceContextPropagator::new();
        let extractor = MultiValueExtractor(HashMap::from([
            (
                TRACEPARENT_HEADER.to_string(),
                vec!["00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01".to_string()],
            ),
            (
                TRACESTATE_HEADER.to_string(),
                vec![
                    "rojo=00f067aa0ba902b7".to_string(),
                    "".to_string(),
                    " congo=t61rcWkgMzE".to_string(),
                ],
            ),
        ]));

        assert_eq!(
            propagator
                .extract(&extractor)
                .span()
                .span_context()
                .trace_state()
                .header(),
            "rojo=00f067aa0ba902b7,congo=t61rcWkgMzE"
        );
    }

    #[test]
    fn extract_w3c_rejects_duplicate_keys_across_tracestate_fields() {
        let propagator = TraceContextPropagator::new();
        let extractor = MultiValueExtractor(HashMap::from([
            (
                TRACEPARENT_HEADER.to_string(),
                vec!["00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01".to_string()],
            ),
            (
                TRACESTATE_HEADER.to_string(),
                vec!["congo=1".to_string(), "congo=2".to_string()],
            ),
        ]));

        let cx = propagator.extract(&extractor);
        let span = cx.span();
        assert!(span.span_context().is_valid());
        assert_eq!(span.span_context().trace_state(), &TraceState::default());
    }

    #[test]
    fn extract_w3c_rejects_duplicate_traceparent() {
        let propagator = TraceContextPropagator::new();
        let header = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01".to_string();
        let extractor = MultiValueExtractor(HashMap::from([
            (TRACEPARENT_HEADER.to_string(), vec![header.clone(), header]),
            (TRACESTATE_HEADER.to_string(), vec!["foo=bar".to_string()]),
        ]));

        assert_eq!(
            propagator.extract(&extractor).span().span_context(),
            &SpanContext::empty_context()
        );
    }

    #[test]
    fn extract_w3c_reject_invalid() {
        let propagator = TraceContextPropagator::new();

        for (invalid_header, reason) in extract_data_invalid() {
            let mut extractor = HashMap::new();
            extractor.insert(TRACEPARENT_HEADER.to_string(), invalid_header.to_string());

            assert_eq!(
                propagator.extract(&extractor).span().span_context(),
                &SpanContext::empty_context(),
                "{reason}"
            )
        }
    }

    #[test]
    fn inject_w3c() {
        let propagator = TraceContextPropagator::new();

        for (expected_trace_parent, expected_trace_state, context) in inject_data() {
            let mut injector = HashMap::new();
            propagator.inject_context(
                &Context::current_with_span(TestSpan(context)),
                &mut injector,
            );

            assert_eq!(
                Extractor::get(&injector, TRACEPARENT_HEADER).unwrap_or(""),
                expected_trace_parent
            );

            assert_eq!(
                Extractor::get(&injector, TRACESTATE_HEADER).unwrap_or(""),
                expected_trace_state
            );
        }
    }

    #[test]
    fn inject_w3c_skips_empty_tracestate() {
        let propagator = TraceContextPropagator::new();
        let context = SpanContext::new(
            TraceId::from(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736),
            SpanId::from(0x00f0_67aa_0ba9_02b7),
            TraceFlags::SAMPLED,
            true,
            TraceState::default(),
        );

        let mut injector: HashMap<String, String> = HashMap::new();
        propagator.inject_context(
            &Context::current_with_span(TestSpan(context)),
            &mut injector,
        );

        assert!(Extractor::get(&injector, TRACEPARENT_HEADER).is_some());
        assert_eq!(Extractor::get(&injector, TRACESTATE_HEADER), None);
    }

    #[test]
    fn inject_w3c_tracestate() {
        let propagator = TraceContextPropagator::new();
        let state = "foo=bar";

        let mut injector: HashMap<String, String> = HashMap::new();
        injector.set(TRACESTATE_HEADER, state.to_string());

        Context::map_current(|cx| propagator.inject_context(cx, &mut injector));

        assert_eq!(Extractor::get(&injector, TRACESTATE_HEADER), Some(state))
    }

    #[test]
    fn round_trip_random_flag() {
        let propagator = TraceContextPropagator::new();

        for flags in ["00", "01", "02", "03"] {
            let header = format!("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-{flags}");
            let mut extractor = HashMap::new();
            extractor.insert(TRACEPARENT_HEADER.to_string(), header.clone());

            let cx = propagator.extract(&extractor);

            let mut injector = HashMap::new();
            propagator.inject_context(&cx, &mut injector);
            assert_eq!(
                Extractor::get(&injector, TRACEPARENT_HEADER),
                Some(header.as_str())
            );
        }
    }
}
