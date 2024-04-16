use opentelemetry::propagation::PropagationError;
use opentelemetry::{
    global::{self, Error},
    propagation::{text_map_propagator::FieldIter, Extractor, Injector, TextMapPropagator},
    trace::{SpanContext, SpanId, TraceContextExt, TraceError, TraceFlags, TraceId, TraceState},
    Context,
};
use std::borrow::Cow;
use std::str::FromStr;

const JAEGER_HEADER: &str = "uber-trace-id";
const JAEGER_BAGGAGE_PREFIX: &str = "uberctx-";
const DEPRECATED_PARENT_SPAN: &str = "0";

const TRACE_FLAG_DEBUG: TraceFlags = TraceFlags::new(0x04);

/// `Propagator` implements the [Jaeger propagation format].
#[derive(Clone, Debug)]
pub struct Propagator {
    baggage_prefix: &'static str,
    header_name: &'static str,
    fields: [String; 1],
}

// Implement default using Propagator::new() to not break compatibility with previous versions
impl Default for Propagator {
    fn default() -> Self {
        Propagator::new()
    }
}

impl Propagator {
    /// Create a Jaeger propagator
    pub fn new() -> Self {
        Self::with_custom_header_and_baggage(JAEGER_HEADER, JAEGER_BAGGAGE_PREFIX)
    }

    /// Create a Jaeger propagator with custom header name
    pub fn with_custom_header(custom_header_name: &'static str) -> Self {
        Self::with_custom_header_and_baggage(custom_header_name, JAEGER_BAGGAGE_PREFIX)
    }

    /// Create a Jaeger propagator with custom header name and baggage prefix
    ///
    /// NOTE: it'll implicitly fallback to the default header names when the name of provided custom_* is empty
    /// Default header-name is `uber-trace-id` and baggage-prefix is `uberctx-`
    /// The format of serialized contexts and baggages stays unchanged and does not depend
    /// on provided header name and prefix.
    pub fn with_custom_header_and_baggage(
        custom_header_name: &'static str,
        custom_baggage_prefix: &'static str,
    ) -> Self {
        let custom_header_name = if custom_header_name.trim().is_empty() {
            JAEGER_HEADER
        } else {
            custom_header_name
        };

        let custom_baggage_prefix = if custom_baggage_prefix.trim().is_empty() {
            JAEGER_BAGGAGE_PREFIX
        } else {
            custom_baggage_prefix
        };

        Propagator {
            baggage_prefix: custom_baggage_prefix.trim(),
            header_name: custom_header_name.trim(),
            fields: [custom_header_name.to_owned()],
        }
    }

    /// Extract span context from header value
    fn extract_span_context(&self, extractor: &dyn Extractor) -> Option<SpanContext> {
        let mut header_value = Cow::from(extractor.get(self.header_name).unwrap_or(""));
        // if there is no :, it means header_value could be encoded as url, try decode first
        if !header_value.contains(':') {
            header_value = Cow::from(header_value.replace("%3A", ":"));
        }

        let parts = header_value.split_terminator(':').collect::<Vec<&str>>();
        if parts.len() != 4 {
            global::handle_error(Error::Propagation(PropagationError::extract(
                "invalid jaeger header format",
                "JaegerPropagator",
            )));
            return None;
        }

        match (
            self.extract_trace_id(parts[0]),
            self.extract_span_id(parts[1]),
            // Ignore parent span id since it's deprecated.
            self.extract_trace_flags(parts[3]),
            self.extract_trace_state(extractor),
        ) {
            (Ok(trace_id), Ok(span_id), Ok(flags), Ok(state)) => {
                Some(SpanContext::new(trace_id, span_id, flags, true, state))
            }
            _ => {
                global::handle_error(Error::Propagation(PropagationError::extract(
                    "invalid jaeger header format",
                    "JaegerPropagator",
                )));
                None
            }
        }
    }

    /// Extract trace id from the header.
    fn extract_trace_id(&self, trace_id: &str) -> Result<TraceId, ()> {
        if trace_id.len() > 32 {
            return Err(());
        }

        TraceId::from_hex(trace_id).map_err(|_| ())
    }

    /// Extract span id from the header.
    fn extract_span_id(&self, span_id: &str) -> Result<SpanId, ()> {
        match span_id.len() {
            // exact 16
            16 => SpanId::from_hex(span_id).map_err(|_| ()),
            // more than 16 is invalid
            17.. => Err(()),
            // less than 16 will result in padding on left
            _ => {
                let padded = format!("{span_id:0>16}");
                SpanId::from_hex(&padded).map_err(|_| ())
            }
        }
    }

    /// Extract flag from the header
    ///
    /// First bit controls whether to sample
    /// Second bit controls whether it's a debug trace
    /// Third bit is not used.
    /// Forth bit is firehose flag, which is not supported in OT now.
    fn extract_trace_flags(&self, flag: &str) -> Result<TraceFlags, ()> {
        if flag.len() > 2 {
            return Err(());
        }
        let flag = u8::from_str(flag).map_err(|_| ())?;
        if flag & 0x01 == 0x01 {
            if flag & 0x02 == 0x02 {
                Ok(TraceFlags::SAMPLED | TRACE_FLAG_DEBUG)
            } else {
                Ok(TraceFlags::SAMPLED)
            }
        } else {
            // Debug flag should only be set when sampled flag is set.
            // So if debug flag is set alone. We will just use not sampled flag
            Ok(TraceFlags::default())
        }
    }

    fn extract_trace_state(&self, extractor: &dyn Extractor) -> Result<TraceState, ()> {
        let baggage_keys = extractor
            .keys()
            .into_iter()
            .filter(|key| key.starts_with(self.baggage_prefix))
            .filter_map(|key| {
                extractor
                    .get(key)
                    .map(|value| (key.to_string(), value.to_string()))
            });

        match TraceState::from_key_value(baggage_keys) {
            Ok(trace_state) => Ok(trace_state),
            Err(trace_state_err) => {
                global::handle_error(Error::Trace(TraceError::Other(Box::new(trace_state_err))));
                Err(()) //todo: assign an error type instead of using ()
            }
        }
    }
}

impl TextMapPropagator for Propagator {
    fn inject_context(&self, cx: &Context, injector: &mut dyn Injector) {
        let span = cx.span();
        let span_context = span.span_context();
        if span_context.is_valid() {
            let flag: u8 = if span_context.is_sampled() {
                if span_context.trace_flags() & TRACE_FLAG_DEBUG == TRACE_FLAG_DEBUG {
                    0x03
                } else {
                    0x01
                }
            } else {
                0x00
            };
            let header_value = format!(
                "{}:{}:{:01}:{:01x}",
                span_context.trace_id(),
                span_context.span_id(),
                DEPRECATED_PARENT_SPAN,
                flag,
            );
            injector.set(self.header_name, header_value);
        }
    }

    fn extract_with_context(&self, cx: &Context, extractor: &dyn Extractor) -> Context {
        self.extract_span_context(extractor)
            .map(|sc| cx.with_remote_span_context(sc))
            .unwrap_or_else(|| cx.clone())
    }

    fn fields(&self) -> FieldIter<'_> {
        FieldIter::new(self.fields.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::testing::trace::TestSpan;
    use std::collections::HashMap;

    const LONG_TRACE_ID_STR: &str = "000000000000004d0000000000000016";
    const SHORT_TRACE_ID_STR: &str = "4d0000000000000016";
    const TRACE_ID: u128 = 0x0000_0000_0000_004d_0000_0000_0000_0016;
    const SPAN_ID_STR: &str = "0000000000017c29";
    const SHORT_SPAN_ID_STR: &str = "17c29";
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
                    TraceFlags::SAMPLED,
                    true,
                    TraceState::default(),
                ),
            ),
            (
                SHORT_TRACE_ID_STR,
                SPAN_ID_STR,
                1,
                SpanContext::new(
                    TraceId::from_u128(TRACE_ID),
                    SpanId::from_u64(SPAN_ID),
                    TraceFlags::SAMPLED,
                    true,
                    TraceState::default(),
                ),
            ),
            (
                SHORT_TRACE_ID_STR,
                SHORT_SPAN_ID_STR,
                1,
                SpanContext::new(
                    TraceId::from_u128(TRACE_ID),
                    SpanId::from_u64(SPAN_ID),
                    TraceFlags::SAMPLED,
                    true,
                    TraceState::default(),
                ),
            ),
            (
                LONG_TRACE_ID_STR,
                SPAN_ID_STR,
                3,
                SpanContext::new(
                    TraceId::from_u128(TRACE_ID),
                    SpanId::from_u64(SPAN_ID),
                    TRACE_FLAG_DEBUG | TraceFlags::SAMPLED,
                    true,
                    TraceState::default(),
                ),
            ),
            (
                LONG_TRACE_ID_STR,
                SPAN_ID_STR,
                0,
                SpanContext::new(
                    TraceId::from_u128(TRACE_ID),
                    SpanId::from_u64(SPAN_ID),
                    TraceFlags::default(),
                    true,
                    TraceState::default(),
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
                    TraceFlags::SAMPLED,
                    true,
                    TraceState::default(),
                ),
                format!("{}:{}:0:1", LONG_TRACE_ID_STR, SPAN_ID_STR),
            ),
            (
                SpanContext::new(
                    TraceId::from_u128(TRACE_ID),
                    SpanId::from_u64(SPAN_ID),
                    TraceFlags::default(),
                    true,
                    TraceState::default(),
                ),
                format!("{}:{}:0:0", LONG_TRACE_ID_STR, SPAN_ID_STR),
            ),
            (
                SpanContext::new(
                    TraceId::from_u128(TRACE_ID),
                    SpanId::from_u64(SPAN_ID),
                    TRACE_FLAG_DEBUG | TraceFlags::SAMPLED,
                    true,
                    TraceState::default(),
                ),
                format!("{}:{}:0:3", LONG_TRACE_ID_STR, SPAN_ID_STR),
            ),
        ]
    }

    /// Try to extract the context using the created Propagator with custom header name
    /// from the Extractor under the `context_key` key.
    fn _test_extract_with_header(construct_header: &'static str, context_key: &'static str) {
        let propagator = Propagator::with_custom_header(construct_header);
        for (trace_id, span_id, flag, expected) in get_extract_data() {
            let mut map: HashMap<String, String> = HashMap::new();
            map.set(context_key, format!("{}:{}:0:{}", trace_id, span_id, flag));
            let context = propagator.extract(&map);
            assert_eq!(context.span().span_context(), &expected);
        }
    }

    /// Try to inject the context using the created Propagator with custom header name
    /// and expect the serialized context existence under `expect_header` key.
    fn _test_inject_with_header(construct_header: &'static str, expect_header: &'static str) {
        let propagator = Propagator::with_custom_header(construct_header);
        for (span_context, header_value) in get_inject_data() {
            let mut injector = HashMap::new();
            propagator.inject_context(
                &Context::current_with_span(TestSpan(span_context)),
                &mut injector,
            );
            assert_eq!(injector.get(expect_header), Some(&header_value));
        }
    }

    #[test]
    fn test_propagator_creation_methods() {
        // Without specifying any custom header or baggage prefix, the header and prefix wil be the default values
        let default_propagator = Propagator::new();
        assert_eq!(default_propagator.header_name, JAEGER_HEADER);
        assert_eq!(default_propagator.baggage_prefix, JAEGER_BAGGAGE_PREFIX);

        // Propagators are cloneable
        let cloned_propagator = default_propagator.clone();
        assert_eq!(
            default_propagator.header_name,
            cloned_propagator.header_name
        );

        // Propagators implement debug
        assert_eq!(
            format!("{:?}", default_propagator),
            format!(
                "Propagator {{ baggage_prefix: \"{}\", header_name: \"{}\", fields: [\"{}\"] }}",
                JAEGER_BAGGAGE_PREFIX, JAEGER_HEADER, JAEGER_HEADER
            )
        );

        let custom_header_propagator = Propagator::with_custom_header("custom-header");
        assert_eq!(custom_header_propagator.header_name, "custom-header");
        assert_eq!(
            custom_header_propagator.baggage_prefix,
            JAEGER_BAGGAGE_PREFIX
        );

        // An empty custom header will result in the default header name
        let propgator_with_empty_custom_header = Propagator::with_custom_header("");
        assert_eq!(
            propgator_with_empty_custom_header.header_name,
            JAEGER_HEADER
        );
        assert_eq!(
            propgator_with_empty_custom_header.baggage_prefix,
            JAEGER_BAGGAGE_PREFIX
        );

        let propagator_with_custom_header_and_baggage_prefixes =
            Propagator::with_custom_header_and_baggage("custom-header", "custom-baggage-prefix");
        assert_eq!(
            propagator_with_custom_header_and_baggage_prefixes.header_name,
            "custom-header"
        );
        assert_eq!(
            propagator_with_custom_header_and_baggage_prefixes.baggage_prefix,
            "custom-baggage-prefix"
        );

        let propagator_with_empty_prefix =
            Propagator::with_custom_header_and_baggage("custom-header", "");
        assert_eq!(propagator_with_empty_prefix.header_name, "custom-header");
        assert_eq!(
            propagator_with_empty_prefix.baggage_prefix,
            JAEGER_BAGGAGE_PREFIX
        );
    }

    #[test]
    fn test_extract_span_context() {
        let propagator_with_custom_header =
            Propagator::with_custom_header_and_baggage("custom_header", "custom_baggage");
        let mut map: HashMap<String, String> = HashMap::new();
        map.insert(
            "custom_header".to_owned(),
            "12345:54321:ignored_parent_span_id:3".to_owned(),
        );
        assert_eq!(
            propagator_with_custom_header.extract_span_context(&map),
            Some(SpanContext::new(
                TraceId::from_hex("12345").unwrap(),
                SpanId::from_hex("54321").unwrap(),
                TRACE_FLAG_DEBUG | TraceFlags::SAMPLED,
                true,
                TraceState::default(),
            ))
        );

        map.clear();
        let mut map: HashMap<String, String> = HashMap::new();
        map.insert(
            "custom_header".to_owned(),
            "12345%3A54321%3Aignored_parent_span_id%3A3".to_owned(), // URL encoded
        );
        assert_eq!(
            propagator_with_custom_header.extract_span_context(&map),
            Some(SpanContext::new(
                TraceId::from_hex("12345").unwrap(),
                SpanId::from_hex("54321").unwrap(),
                TRACE_FLAG_DEBUG | TraceFlags::SAMPLED,
                true,
                TraceState::default(),
            ))
        );

        map.clear();
        map.set(
            "custom_header",
            "not:4:parts:long:delimited:by:colons".to_owned(),
        );
        assert_eq!(
            propagator_with_custom_header.extract_span_context(&map),
            None,
        );

        map.clear();
        map.set(
            "custom_header",
            "invalid_trace_id:54321:ignored_parent_span_id:3".to_owned(),
        );
        assert_eq!(
            propagator_with_custom_header.extract_span_context(&map),
            None,
        );

        map.clear();
        map.set(
            "custom_header",
            "12345:invalid_span_id:ignored_parent_span_id:3".to_owned(),
        );
        assert_eq!(
            propagator_with_custom_header.extract_span_context(&map),
            None,
        );

        map.clear();
        map.set(
            "custom_header",
            "12345:54321:ignored_parent_span_id:invalid_flag".to_owned(),
        );
        assert_eq!(
            propagator_with_custom_header.extract_span_context(&map),
            None,
        );

        map.clear();
        let mut map: HashMap<String, String> = HashMap::new();
        map.set(
            "custom_header",
            "12345%3A54321%3Aignored_parent_span_id%3A3".to_owned(), // URL encoded
        );
        let too_long_baggage_key = format!("{}{}", "custom_baggage", "_".repeat(256)); // A baggage key cannot be longer than 256 characters
        map.set(&too_long_baggage_key, "baggage_value".to_owned());
        assert_eq!(
            propagator_with_custom_header.extract_span_context(&map),
            None,
        );
    }

    #[test]
    fn test_extract_trace_id() {
        let propagator = Propagator::new();

        assert_eq!(
            propagator.extract_trace_id("12345"),
            Ok(TraceId::from_hex("12345").unwrap())
        );

        // A trace cannot be more than 32 characters
        assert_eq!(
            propagator.extract_trace_id("1".repeat(33).as_str()),
            Err(())
        );

        // A trace id must be a valid hex-string
        assert_eq!(propagator.extract_trace_id("invalid"), Err(()));
    }

    #[test]
    fn test_extract_span_id() {
        let propgator = Propagator::new();
        assert_eq!(
            propgator.extract_span_id("12345"),
            Ok(SpanId::from_u64(74565))
        );

        // Fail to extract span id with an invalid hex-string
        assert_eq!(propgator.extract_span_id("invalid"), Err(()));

        // Fail to extract span id with a hex-string that is too long
        assert_eq!(propgator.extract_span_id("1".repeat(17).as_str()), Err(()));
    }

    #[test]
    fn test_extract_trace_flags() {
        let propgator = Propagator::new();

        // Extract TraceFlags::SAMPLED flag
        assert_eq!(propgator.extract_trace_flags("1"), Ok(TraceFlags::SAMPLED));

        // Extract TraceFlags::DEBUG flag - requires TraceFlags::SAMPLED to be set
        assert_eq!(
            propgator.extract_trace_flags("3"),
            Ok(TRACE_FLAG_DEBUG | TraceFlags::SAMPLED)
        );

        // Attempt to extract the TraceFlags::DEBUG flag without the TraceFlags::SAMPLED flag and receive the default TraceFlags
        assert_eq!(
            propgator.extract_trace_flags("2"),
            Ok(TraceFlags::default())
        );
    }

    #[test]
    fn test_extract_trace_state() {
        let propagator = Propagator::with_custom_header_and_baggage("header", "baggage");

        // When a type that implements Extractor has keys that start with the custom baggage prefix, they and their associated
        // values are extracted into a TraceState
        // In this case, no keys start with the custom baggage prefix
        let mut map_of_keys_without_custom_baggage_prefix: HashMap<String, String> = HashMap::new();
        map_of_keys_without_custom_baggage_prefix.set("different_prefix_1", "value_1".to_string());
        let empty_trace_state = propagator
            .extract_trace_state(&map_of_keys_without_custom_baggage_prefix)
            .unwrap();
        assert_eq!(empty_trace_state, TraceState::NONE);

        // In this case, all keys start with the custom baggage prefix
        let mut map_of_keys_with_custom_baggage_prefix: HashMap<String, String> = HashMap::new();
        map_of_keys_with_custom_baggage_prefix.set("baggage_1", "value_1".to_string());

        let trace_state = propagator
            .extract_trace_state(&map_of_keys_with_custom_baggage_prefix)
            .unwrap();
        assert_eq!(
            trace_state,
            TraceState::from_key_value(vec![("baggage_1", "value_1")]).unwrap()
        );

        // If a key that starts with the custom baggage prefix is an invalid TraceState, the result will be Err(())
        let too_long_baggage_key = format!("{}{}", "baggage_1", "_".repeat(256)); // A baggage key cannot be longer than 256 characters
        let mut map_of_invalid_keys_with_custom_baggage_prefix: HashMap<String, String> =
            HashMap::new();
        map_of_invalid_keys_with_custom_baggage_prefix
            .set(&too_long_baggage_key, "value_1".to_string());
        assert_eq!(
            propagator.extract_trace_state(&map_of_invalid_keys_with_custom_baggage_prefix),
            Err(())
        );
    }

    #[test]
    fn test_extract_empty() {
        let map: HashMap<String, String> = HashMap::new();
        let propagator = Propagator::new();
        let context = propagator.extract(&map);
        assert_eq!(context.span().span_context(), &SpanContext::empty_context())
    }

    #[test]
    fn test_inject_extract_with_default() {
        let propagator = Propagator::default();
        for (span_context, header_value) in get_inject_data() {
            let mut injector = HashMap::new();
            propagator.inject_context(
                &Context::current_with_span(TestSpan(span_context)),
                &mut injector,
            );
            assert_eq!(injector.get(JAEGER_HEADER), Some(&header_value));
        }
        for (trace_id, span_id, flag, expected) in get_extract_data() {
            let mut map: HashMap<String, String> = HashMap::new();
            map.set(
                JAEGER_HEADER,
                format!("{}:{}:0:{}", trace_id, span_id, flag),
            );
            let context = propagator.extract(&map);
            assert_eq!(context.span().span_context(), &expected);
        }
    }

    #[test]
    fn test_extract_too_many_parts() {
        let mut map: HashMap<String, String> = HashMap::new();
        map.set(
            JAEGER_HEADER,
            format!("{}:{}:0:1:aa", LONG_TRACE_ID_STR, SPAN_ID_STR),
        );
        let propagator = Propagator::new();
        let context = propagator.extract(&map);
        assert_eq!(context.span().span_context(), &SpanContext::empty_context());
    }

    #[test]
    fn test_extract_invalid_flag() {
        let mut map: HashMap<String, String> = HashMap::new();
        map.set(
            JAEGER_HEADER,
            format!("{}:{}:0:aa", LONG_TRACE_ID_STR, SPAN_ID_STR),
        );
        let propagator = Propagator::new();
        let context = propagator.extract(&map);
        assert_eq!(context.span().span_context(), &SpanContext::empty_context());
    }

    #[test]
    fn test_extract_from_url() {
        let mut map: HashMap<String, String> = HashMap::new();
        map.set(
            JAEGER_HEADER,
            format!("{}%3A{}%3A0%3A1", LONG_TRACE_ID_STR, SPAN_ID_STR),
        );
        let propagator = Propagator::new();
        let context = propagator.extract(&map);
        assert_eq!(
            context.span().span_context(),
            &SpanContext::new(
                TraceId::from_u128(TRACE_ID),
                SpanId::from_u64(SPAN_ID),
                TraceFlags::SAMPLED,
                true,
                TraceState::default(),
            )
        );
    }

    #[test]
    fn test_extract() {
        _test_extract_with_header(JAEGER_HEADER, JAEGER_HEADER)
    }

    #[test]
    fn test_inject() {
        _test_inject_with_header(JAEGER_HEADER, JAEGER_HEADER)
    }

    #[test]
    fn test_extract_with_invalid_header() {
        for construct in &["", "   "] {
            _test_extract_with_header(construct, JAEGER_HEADER)
        }
    }

    #[test]
    fn test_extract_with_valid_header() {
        for construct in &["custom-header", "custom-header   ", "   custom-header   "] {
            _test_extract_with_header(construct, "custom-header")
        }
    }

    #[test]
    fn test_inject_with_invalid_header() {
        for construct in &["", "   "] {
            _test_inject_with_header(construct, JAEGER_HEADER)
        }
    }

    #[test]
    fn test_inject_with_valid_header() {
        for construct in &["custom-header", "custom-header   ", "   custom-header   "] {
            _test_inject_with_header(construct, "custom-header")
        }
    }

    #[test]
    fn test_fields() {
        let propagator = Propagator::new();
        let fields = propagator.fields().collect::<Vec<_>>();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields.first().unwrap(), &JAEGER_HEADER);
    }
}
