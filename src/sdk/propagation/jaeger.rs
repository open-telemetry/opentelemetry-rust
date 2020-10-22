//! # Jaeger Propagator
//!
//! Extract and inject values from Jaeger's `uber-trace-id` header.
//!
//! See [`Jaeger documentation`] for detail of Jaeger propagation format.
//!
//! [`Jaeger documentation`]: https://www.jaegertracing.io/docs/1.18/client-libraries/#propagation-format
use crate::api::{
    propagation::{text_map_propagator::FieldIter, Extractor, Injector, TextMapPropagator},
    trace::{
        SpanId, SpanReference, TraceContextExt, TraceId, TraceState, TRACE_FLAG_DEBUG,
        TRACE_FLAG_NOT_SAMPLED, TRACE_FLAG_SAMPLED,
    },
    Context,
};
use std::borrow::Cow;
use std::str::FromStr;

const JAEGER_HEADER: &str = "uber-trace-id";
const JAEGER_BAGGAGE_PREFIX: &str = "uberctx-";
const DEPRECATED_PARENT_SPAN: &str = "0";

lazy_static::lazy_static! {
    static ref JAEGER_HEADER_FIELD: [String; 1] = [JAEGER_HEADER.to_string()];
}

/// The Jaeger propagator propagates span contexts in jaeger's propagation format.
///
/// See [`Jaeger documentation`] for format details.
///
/// Note that jaeger header can be set in http header or encoded as url
///
///  [`Jaeger documentation`]: https://www.jaegertracing.io/docs/1.18/client-libraries/#propagation-format
#[derive(Clone, Debug)]
pub struct JaegerPropagator {
    _private: (),
}

impl Default for JaegerPropagator {
    fn default() -> Self {
        JaegerPropagator { _private: () }
    }
}

impl JaegerPropagator {
    /// Create a Jaeger propagator
    pub fn new() -> Self {
        JaegerPropagator::default()
    }

    /// Extract span context from header value
    fn extract_span_reference(&self, extractor: &dyn Extractor) -> Result<SpanReference, ()> {
        let mut header_value = Cow::from(extractor.get(JAEGER_HEADER).unwrap_or(""));
        // if there is no :, it means header_value could be encoded as url, try decode first
        if !header_value.contains(':') {
            header_value = Cow::from(header_value.replace("%3A", ":"));
        }

        let parts = header_value.split_terminator(':').collect::<Vec<&str>>();
        if parts.len() != 4 {
            return Err(());
        }

        // extract trace id
        let trace_id = self.extract_trace_id(parts[0])?;
        let span_id = self.extract_span_id(parts[1])?;
        // Ignore parent span id since it's deprecated.
        let flag = self.extract_flag(parts[3])?;
        let trace_state = self.extract_trace_state(extractor)?;

        Ok(SpanReference::new(
            trace_id,
            span_id,
            flag,
            true,
            trace_state,
        ))
    }

    /// Extract trace id from the header.
    fn extract_trace_id(&self, trace_id: &str) -> Result<TraceId, ()> {
        if trace_id.len() > 32 {
            return Err(());
        }

        // allow variable length, padding 0 when length is less than 32
        let padded_trace_id = format!("{:0>32}", trace_id);

        u128::from_str_radix(padded_trace_id.as_str(), 16)
            .map(TraceId::from_u128)
            .map_err(|_| ())
    }

    /// Extract span id from the header.
    fn extract_span_id(&self, span_id: &str) -> Result<SpanId, ()> {
        if span_id.len() != 16 {
            return Err(());
        }

        u64::from_str_radix(span_id, 16)
            .map(SpanId::from_u64)
            .map_err(|_| ())
    }

    /// Extract flag from the header
    ///
    /// First bit control whether to sample
    /// Second bit control whether it's a debug trace
    /// Third bit is not used.
    /// Forth bit is firehose flag, which is not supported in OT now.
    fn extract_flag(&self, flag: &str) -> Result<u8, ()> {
        if flag.len() > 2 {
            return Err(());
        }
        let flag = u8::from_str(flag).map_err(|_| ())?;
        if flag & 0x01 == 0x01 {
            if flag & 0x02 == 0x02 {
                Ok(TRACE_FLAG_SAMPLED | TRACE_FLAG_DEBUG)
            } else {
                Ok(TRACE_FLAG_SAMPLED)
            }
        } else {
            // Debug flag should only be set when sampled flag is set.
            // So if debug flag is set alone. We will just use not sampled flag
            Ok(TRACE_FLAG_NOT_SAMPLED)
        }
    }

    fn extract_trace_state(&self, extractor: &dyn Extractor) -> Result<TraceState, ()> {
        let uber_context_keys = extractor
            .keys()
            .into_iter()
            .filter(|key| key.starts_with(JAEGER_BAGGAGE_PREFIX))
            .filter_map(|key| {
                extractor
                    .get(key)
                    .map(|value| (key.to_string(), value.to_string()))
            });

        TraceState::from_key_value(uber_context_keys)
    }
}

impl TextMapPropagator for JaegerPropagator {
    fn inject_context(&self, cx: &Context, injector: &mut dyn Injector) {
        let span_reference = cx.span().span_reference();
        if span_reference.is_valid() {
            let flag: u8 = if span_reference.is_sampled() {
                if span_reference.is_debug() {
                    0x03
                } else {
                    0x01
                }
            } else {
                0x00
            };
            let header_value = format!(
                "{:032x}:{:016x}:{:01}:{:01}",
                span_reference.trace_id().to_u128(),
                span_reference.span_id().to_u64(),
                DEPRECATED_PARENT_SPAN,
                flag,
            );
            injector.set(JAEGER_HEADER, header_value);
        }
    }

    fn extract_with_context(&self, cx: &Context, extractor: &dyn Extractor) -> Context {
        cx.with_remote_span_reference(
            self.extract_span_reference(extractor)
                .unwrap_or_else(|_| SpanReference::empty_context()),
        )
    }

    fn fields(&self) -> FieldIter {
        FieldIter::new(JAEGER_HEADER_FIELD.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{
        propagation::{Injector, TextMapPropagator},
        trace::{
            SpanId, SpanReference, TraceContextExt, TraceId, TraceState, TRACE_FLAG_DEBUG,
            TRACE_FLAG_NOT_SAMPLED, TRACE_FLAG_SAMPLED,
        },
        Context,
    };
    use crate::testing::utils::TestSpan;
    use std::collections::HashMap;

    const LONG_TRACE_ID_STR: &str = "000000000000004d0000000000000016";
    const SHORT_TRACE_ID_STR: &str = "4d0000000000000016";
    const TRACE_ID: u128 = 0x0000_0000_0000_004d_0000_0000_0000_0016;
    const SPAN_ID_STR: &str = "0000000000017c29";
    const SPAN_ID: u64 = 0x0000_0000_0001_7c29;

    fn get_extract_data() -> Vec<(&'static str, &'static str, u8, SpanReference)> {
        vec![
            (
                LONG_TRACE_ID_STR,
                SPAN_ID_STR,
                1,
                SpanReference::new(
                    TraceId::from_u128(TRACE_ID),
                    SpanId::from_u64(SPAN_ID),
                    TRACE_FLAG_SAMPLED,
                    true,
                    TraceState::default(),
                ),
            ),
            (
                SHORT_TRACE_ID_STR,
                SPAN_ID_STR,
                1,
                SpanReference::new(
                    TraceId::from_u128(TRACE_ID),
                    SpanId::from_u64(SPAN_ID),
                    TRACE_FLAG_SAMPLED,
                    true,
                    TraceState::default(),
                ),
            ),
            (
                LONG_TRACE_ID_STR,
                SPAN_ID_STR,
                3,
                SpanReference::new(
                    TraceId::from_u128(TRACE_ID),
                    SpanId::from_u64(SPAN_ID),
                    TRACE_FLAG_DEBUG | TRACE_FLAG_SAMPLED,
                    true,
                    TraceState::default(),
                ),
            ),
            (
                LONG_TRACE_ID_STR,
                SPAN_ID_STR,
                0,
                SpanReference::new(
                    TraceId::from_u128(TRACE_ID),
                    SpanId::from_u64(SPAN_ID),
                    TRACE_FLAG_NOT_SAMPLED,
                    true,
                    TraceState::default(),
                ),
            ),
            (
                "invalidtractid",
                SPAN_ID_STR,
                0,
                SpanReference::empty_context(),
            ),
            (
                LONG_TRACE_ID_STR,
                "invalidspanID",
                0,
                SpanReference::empty_context(),
            ),
            (
                LONG_TRACE_ID_STR,
                SPAN_ID_STR,
                120,
                SpanReference::empty_context(),
            ),
        ]
    }

    fn get_inject_data() -> Vec<(SpanReference, String)> {
        vec![
            (
                SpanReference::new(
                    TraceId::from_u128(TRACE_ID),
                    SpanId::from_u64(SPAN_ID),
                    TRACE_FLAG_SAMPLED,
                    true,
                    TraceState::default(),
                ),
                format!("{}:{}:0:1", LONG_TRACE_ID_STR, SPAN_ID_STR),
            ),
            (
                SpanReference::new(
                    TraceId::from_u128(TRACE_ID),
                    SpanId::from_u64(SPAN_ID),
                    TRACE_FLAG_NOT_SAMPLED,
                    true,
                    TraceState::default(),
                ),
                format!("{}:{}:0:0", LONG_TRACE_ID_STR, SPAN_ID_STR),
            ),
            (
                SpanReference::new(
                    TraceId::from_u128(TRACE_ID),
                    SpanId::from_u64(SPAN_ID),
                    TRACE_FLAG_DEBUG | TRACE_FLAG_SAMPLED,
                    true,
                    TraceState::default(),
                ),
                format!("{}:{}:0:3", LONG_TRACE_ID_STR, SPAN_ID_STR),
            ),
        ]
    }

    #[test]
    fn test_extract_empty() {
        let map: HashMap<String, String> = HashMap::new();
        let propagator = JaegerPropagator::new();
        let context = propagator.extract(&map);
        assert_eq!(
            context.remote_span_reference(),
            Some(&SpanReference::empty_context())
        )
    }

    #[test]
    fn test_extract() {
        for (trace_id, span_id, flag, expected) in get_extract_data() {
            let mut map: HashMap<String, String> = HashMap::new();
            map.set(
                JAEGER_HEADER,
                format!("{}:{}:0:{}", trace_id, span_id, flag),
            );
            let propagator = JaegerPropagator::new();
            let context = propagator.extract(&map);
            assert_eq!(context.remote_span_reference(), Some(&expected));
        }
    }

    #[test]
    fn test_extract_too_many_parts() {
        let mut map: HashMap<String, String> = HashMap::new();
        map.set(
            JAEGER_HEADER,
            format!("{}:{}:0:1:aa", LONG_TRACE_ID_STR, SPAN_ID_STR),
        );
        let propagator = JaegerPropagator::new();
        let context = propagator.extract(&map);
        assert_eq!(
            context.remote_span_reference(),
            Some(&SpanReference::empty_context())
        );
    }

    #[test]
    fn test_extract_invalid_flag() {
        let mut map: HashMap<String, String> = HashMap::new();
        map.set(
            JAEGER_HEADER,
            format!("{}:{}:0:aa", LONG_TRACE_ID_STR, SPAN_ID_STR),
        );
        let propagator = JaegerPropagator::new();
        let context = propagator.extract(&map);
        assert_eq!(
            context.remote_span_reference(),
            Some(&SpanReference::empty_context())
        );
    }

    #[test]
    fn test_extract_from_url() {
        let mut map: HashMap<String, String> = HashMap::new();
        map.set(
            JAEGER_HEADER,
            format!("{}%3A{}%3A0%3A1", LONG_TRACE_ID_STR, SPAN_ID_STR),
        );
        let propagator = JaegerPropagator::new();
        let context = propagator.extract(&map);
        assert_eq!(
            context.remote_span_reference(),
            Some(&SpanReference::new(
                TraceId::from_u128(TRACE_ID),
                SpanId::from_u64(SPAN_ID),
                1,
                true,
                TraceState::default(),
            ))
        );
    }
    #[test]
    fn test_inject() {
        let propagator = JaegerPropagator::new();
        for (span_reference, header_value) in get_inject_data() {
            let mut injector = HashMap::new();
            propagator.inject_context(
                &Context::current_with_span(TestSpan(span_reference)),
                &mut injector,
            );
            assert_eq!(injector.get(JAEGER_HEADER), Some(&header_value));
        }
    }
}
