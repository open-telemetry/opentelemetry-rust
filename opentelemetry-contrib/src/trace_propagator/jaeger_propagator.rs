//! # Jaeger Propagator
//!
//! Extract and inject values from Jaeger's `uber-trace-id` header.
//!
//! See [`Jaeger documentation`] for detail of Jaeger propagation format.
//!
//! [`Jaeger documentation`]: https://www.jaegertracing.io/docs/1.18/client-libraries/#propagation-format

use opentelemetry::api::{
    Context, Extractor, FieldIter, Injector, SpanContext, SpanId, TextMapFormat, TraceContextExt,
    TraceId, TRACE_FLAG_DEBUG, TRACE_FLAG_NOT_SAMPLED, TRACE_FLAG_SAMPLED,
};
use std::str::FromStr;

const JAEGER_HEADER: &str = "uber-trace-id";
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
    fn extract_span_context(&self, header_value: &str) -> Result<SpanContext, ()> {
        let parts = header_value.split_terminator(':').collect::<Vec<&str>>();
        if parts.len() != 4 {
            return Err(());
        }

        // extract trace id
        let trace_id = self.extract_trace_id(parts[0])?;
        let span_id = self.extract_span_id(parts[1])?;
        // Ignore parent span id since it's deprecated.
        let flag = self.extract_flag(parts[3])?;

        Ok(SpanContext::new(trace_id, span_id, flag, true))
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
}

impl TextMapFormat for JaegerPropagator {
    fn inject_context(&self, cx: &Context, injector: &mut dyn Injector) {
        let span_context = cx.span().span_context();
        if span_context.is_valid() {
            let flag: u8 = if span_context.is_sampled() {
                if span_context.is_debug() {
                    0x03
                } else {
                    0x01
                }
            } else {
                0x00
            };
            let header_value = format!(
                "{:032x}:{:016x}:{:01}:{:01}",
                span_context.trace_id().to_u128(),
                span_context.span_id().to_u64(),
                DEPRECATED_PARENT_SPAN,
                flag,
            );
            injector.set(JAEGER_HEADER, header_value);
        }
    }

    fn extract_with_context(&self, cx: &Context, extractor: &dyn Extractor) -> Context {
        let header_value = extractor.get(JAEGER_HEADER).unwrap_or("");
        // if there is no :, it means header_value could be encoded as url, try decode first
        let extract_result = if !header_value.contains(':') {
            self.extract_span_context(header_value.replace("%3A", ":").as_str())
        } else {
            self.extract_span_context(header_value)
        };
        cx.with_remote_span_context(extract_result.unwrap_or_else(|_| SpanContext::empty_context()))
    }

    fn fields(&self) -> FieldIter {
        FieldIter::new(JAEGER_HEADER_FIELD.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use crate::trace_propagator::jaeger_propagator::{JaegerPropagator, JAEGER_HEADER};
    use opentelemetry::api;
    use opentelemetry::api::{
        Context, Injector, Span, SpanContext, SpanId, TextMapFormat, TraceContextExt, TraceId,
        TRACE_FLAG_DEBUG, TRACE_FLAG_NOT_SAMPLED, TRACE_FLAG_SAMPLED,
    };
    use std::collections::HashMap;
    use std::time::SystemTime;

    const LONG_TRACE_ID_STR: &str = "000000000000004d0000000000000016";
    const SHORT_TRACE_ID_STR: &str = "4d0000000000000016";
    const TRACE_ID: u128 = 0x0000_0000_0000_004d_0000_0000_0000_0016;
    const SPAN_ID_STR: &str = "0000000000017c29";
    const SPAN_ID: u64 = 0x0000_0000_0001_7c29;

    fn get_extract_data() -> Vec<(&'static str, &'static str, u8, SpanContext)> {
        vec![
            (
                LONG_TRACE_ID_STR,
                SPAN_ID_STR,
                1,
                SpanContext::new(
                    TraceId::from_u128(TRACE_ID),
                    SpanId::from_u64(SPAN_ID),
                    TRACE_FLAG_SAMPLED,
                    true,
                ),
            ),
            (
                SHORT_TRACE_ID_STR,
                SPAN_ID_STR,
                1,
                SpanContext::new(
                    TraceId::from_u128(TRACE_ID),
                    SpanId::from_u64(SPAN_ID),
                    TRACE_FLAG_SAMPLED,
                    true,
                ),
            ),
            (
                LONG_TRACE_ID_STR,
                SPAN_ID_STR,
                3,
                SpanContext::new(
                    TraceId::from_u128(TRACE_ID),
                    SpanId::from_u64(SPAN_ID),
                    TRACE_FLAG_DEBUG | TRACE_FLAG_SAMPLED,
                    true,
                ),
            ),
            (
                LONG_TRACE_ID_STR,
                SPAN_ID_STR,
                0,
                SpanContext::new(
                    TraceId::from_u128(TRACE_ID),
                    SpanId::from_u64(SPAN_ID),
                    TRACE_FLAG_NOT_SAMPLED,
                    true,
                ),
            ),
            (
                "invalidtractid",
                SPAN_ID_STR,
                0,
                SpanContext::empty_context(),
            ),
            (
                LONG_TRACE_ID_STR,
                "invalidspanID",
                0,
                SpanContext::empty_context(),
            ),
            (
                LONG_TRACE_ID_STR,
                SPAN_ID_STR,
                120,
                SpanContext::empty_context(),
            ),
        ]
    }

    fn get_inject_data() -> Vec<(SpanContext, String)> {
        vec![
            (
                SpanContext::new(
                    TraceId::from_u128(TRACE_ID),
                    SpanId::from_u64(SPAN_ID),
                    TRACE_FLAG_SAMPLED,
                    true,
                ),
                format!("{}:{}:0:1", LONG_TRACE_ID_STR, SPAN_ID_STR),
            ),
            (
                SpanContext::new(
                    TraceId::from_u128(TRACE_ID),
                    SpanId::from_u64(SPAN_ID),
                    TRACE_FLAG_NOT_SAMPLED,
                    true,
                ),
                format!("{}:{}:0:0", LONG_TRACE_ID_STR, SPAN_ID_STR),
            ),
            (
                SpanContext::new(
                    TraceId::from_u128(TRACE_ID),
                    SpanId::from_u64(SPAN_ID),
                    TRACE_FLAG_DEBUG | TRACE_FLAG_SAMPLED,
                    true,
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
            context.remote_span_context(),
            Some(&SpanContext::empty_context())
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
            assert_eq!(context.remote_span_context(), Some(&expected));
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
            context.remote_span_context(),
            Some(&SpanContext::empty_context())
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
            context.remote_span_context(),
            Some(&SpanContext::empty_context())
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
            context.remote_span_context(),
            Some(&SpanContext::new(
                TraceId::from_u128(TRACE_ID),
                SpanId::from_u64(SPAN_ID),
                1,
                true
            ))
        );
    }

    #[derive(Debug)]
    struct TestSpan(SpanContext);

    impl Span for TestSpan {
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
        fn end_with_timestamp(&self, _timestamp: SystemTime) {}
    }

    #[test]
    fn test_inject() {
        let propagator = JaegerPropagator::new();
        for (span_context, header_value) in get_inject_data() {
            let mut injector = HashMap::new();
            propagator.inject_context(
                &Context::current_with_span(TestSpan(span_context)),
                &mut injector,
            );
            assert_eq!(injector.get(JAEGER_HEADER), Some(&header_value));
        }
    }
}
