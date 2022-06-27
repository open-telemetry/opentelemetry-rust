//! This crate provides unofficial integration with AWS services.
//!
//! # Components
//! As for now, the only components provided in this crate is AWS X-Ray propagator.
//!
//! ### AWS X-Ray Propagator
//! This propagator helps propagate tracing information from upstream services to downstream services.
//!
//! ### Quick start
//! ```no_run
//! use opentelemetry::global;
//! use opentelemetry_aws::trace::XrayPropagator;
//! use opentelemetry::{sdk::export::trace::stdout, trace::Tracer};
//! use opentelemetry_http::HeaderInjector;
//!
//! #[tokio::main]
//! async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
//!     // Set the global propagator to X-Ray propagator
//!     global::set_text_map_propagator(XrayPropagator::default());
//!     let tracer = stdout::new_pipeline().install_simple();
//!
//!     let mut req = hyper::Request::builder().uri("http://127.0.0.1:3000");
//!     tracer.in_span("doing_work", |cx| {
//!         // Send request to downstream services.
//!         // Build request
//!         global::get_text_map_propagator(|propagator| {
//!             // Set X-Ray tracing header in request object `req`
//!             propagator.inject_context(&cx, &mut HeaderInjector(req.headers_mut().unwrap()));
//!             println!("Headers: {:?}", req.headers_ref());
//!         })
//!     });
//!
//!     Ok(())
//! }
//! ```
//! A more detailed example can be found in [opentelemetry-rust](https://github.com/open-telemetry/opentelemetry-rust/tree/main/examples/aws-xray) repo
#[cfg(feature = "trace")]
pub mod trace {
    use once_cell::sync::Lazy;
    use opentelemetry::{
        global::{self, Error},
        propagation::{text_map_propagator::FieldIter, Extractor, Injector, TextMapPropagator},
        trace::{
            SpanContext, SpanId, TraceContextExt, TraceError, TraceFlags, TraceId, TraceState,
        },
        Context,
    };
    use std::convert::{TryFrom, TryInto};

    const AWS_XRAY_TRACE_HEADER: &str = "x-amzn-trace-id";
    const AWS_XRAY_VERSION_KEY: &str = "1";
    const HEADER_PARENT_KEY: &str = "Parent";
    const HEADER_ROOT_KEY: &str = "Root";
    const HEADER_SAMPLED_KEY: &str = "Sampled";

    const SAMPLED: &str = "1";
    const NOT_SAMPLED: &str = "0";
    const REQUESTED_SAMPLE_DECISION: &str = "?";

    const TRACE_FLAG_DEFERRED: TraceFlags = TraceFlags::new(0x02);

    static AWS_XRAY_HEADER_FIELD: Lazy<[String; 1]> =
        Lazy::new(|| [AWS_XRAY_TRACE_HEADER.to_owned()]);

    /// Extracts and injects `SpanContext`s into `Extractor`s or `Injector`s using AWS X-Ray header format.
    ///
    /// Extracts and injects values to/from the `x-amzn-trace-id` header. Converting between
    /// OpenTelemetry [SpanContext][otel-spec] and [X-Ray Trace format][xray-trace-id].
    ///
    /// For details on the [`x-amzn-trace-id` header][xray-header] see the AWS X-Ray Docs.
    ///
    /// ## Example
    ///
    /// ```
    /// use opentelemetry::global;
    /// use opentelemetry_aws::trace::XrayPropagator;
    ///
    /// global::set_text_map_propagator(XrayPropagator::default());
    /// ```
    ///
    /// [otel-spec]: https://github.com/open-telemetry/opentelemetry-specification/blob/master/specification/trace/api.md#SpanContext
    /// [xray-trace-id]: https://docs.aws.amazon.com/xray/latest/devguide/xray-api-sendingdata.html#xray-api-traceids
    /// [xray-header]: https://docs.aws.amazon.com/xray/latest/devguide/xray-concepts.html#xray-concepts-tracingheader
    #[derive(Clone, Debug, Default)]
    pub struct XrayPropagator {
        _private: (),
    }

    impl XrayPropagator {
        /// Creates a new `XrayTraceContextPropagator`.
        pub fn new() -> Self {
            XrayPropagator::default()
        }

        fn extract_span_context(&self, extractor: &dyn Extractor) -> Result<SpanContext, ()> {
            let header_value: &str = extractor.get(AWS_XRAY_TRACE_HEADER).unwrap_or("").trim();

            let parts: Vec<(&str, &str)> = header_value
                .split_terminator(';')
                .filter_map(from_key_value_pair)
                .collect();

            let mut trace_id: TraceId = TraceId::INVALID;
            let mut parent_segment_id: SpanId = SpanId::INVALID;
            let mut sampling_decision = TRACE_FLAG_DEFERRED;
            let mut kv_vec: Vec<(String, String)> = Vec::with_capacity(parts.len());

            for (key, value) in parts {
                match key {
                    HEADER_ROOT_KEY => {
                        let converted_trace_id: Result<TraceId, ()> =
                            XrayTraceId(value.to_string()).try_into();
                        match converted_trace_id {
                            Err(_) => return Err(()),
                            Ok(parsed) => trace_id = parsed,
                        }
                    }
                    HEADER_PARENT_KEY => {
                        parent_segment_id = SpanId::from_hex(value).unwrap_or(SpanId::INVALID)
                    }
                    HEADER_SAMPLED_KEY => {
                        sampling_decision = match value {
                            NOT_SAMPLED => TraceFlags::default(),
                            SAMPLED => TraceFlags::SAMPLED,
                            REQUESTED_SAMPLE_DECISION => TRACE_FLAG_DEFERRED,
                            _ => TRACE_FLAG_DEFERRED,
                        }
                    }
                    _ => kv_vec.push((key.to_ascii_lowercase(), value.to_string())),
                }
            }

            match TraceState::from_key_value(kv_vec) {
                Ok(trace_state) => {
                    if trace_id == TraceId::INVALID {
                        return Err(());
                    }

                    let context: SpanContext = SpanContext::new(
                        trace_id,
                        parent_segment_id,
                        sampling_decision,
                        true,
                        trace_state,
                    );

                    Ok(context)
                }
                Err(trace_state_err) => {
                    global::handle_error(Error::Trace(TraceError::Other(Box::new(
                        trace_state_err,
                    ))));
                    Err(()) //todo: assign an error type instead of using ()
                }
            }
        }
    }

    impl TextMapPropagator for XrayPropagator {
        fn inject_context(&self, cx: &Context, injector: &mut dyn Injector) {
            let span = cx.span();
            let span_context = span.span_context();
            if span_context.is_valid() {
                let xray_trace_id: XrayTraceId = span_context.trace_id().into();

                let sampling_decision: &str =
                    if span_context.trace_flags() & TRACE_FLAG_DEFERRED == TRACE_FLAG_DEFERRED {
                        REQUESTED_SAMPLE_DECISION
                    } else if span_context.is_sampled() {
                        SAMPLED
                    } else {
                        NOT_SAMPLED
                    };

                let trace_state_header: String = span_context
                    .trace_state()
                    .header_delimited("=", ";")
                    .split_terminator(';')
                    .map(title_case)
                    .collect::<Vec<String>>()
                    .join(";");
                let trace_state_prefix = if trace_state_header.is_empty() {
                    ""
                } else {
                    ";"
                };

                injector.set(
                    AWS_XRAY_TRACE_HEADER,
                    format!(
                        "{}={};{}={:016x};{}={}{}{}",
                        HEADER_ROOT_KEY,
                        xray_trace_id.0,
                        HEADER_PARENT_KEY,
                        span_context.span_id(),
                        HEADER_SAMPLED_KEY,
                        sampling_decision,
                        trace_state_prefix,
                        trace_state_header
                    ),
                );
            }
        }

        fn extract_with_context(&self, cx: &Context, extractor: &dyn Extractor) -> Context {
            let extracted = self
                .extract_span_context(extractor)
                .unwrap_or_else(|_| SpanContext::empty_context());

            cx.with_remote_span_context(extracted)
        }

        fn fields(&self) -> FieldIter<'_> {
            FieldIter::new(AWS_XRAY_HEADER_FIELD.as_ref())
        }
    }

    /// Holds an X-Ray formatted Trace ID
    ///
    /// A `trace_id` consists of three numbers separated by hyphens. For example, `1-58406520-a006649127e371903a2de979`.
    /// This includes:
    ///
    /// * The version number, that is, 1.
    /// * The time of the original request, in Unix epoch time, in 8 hexadecimal digits.
    /// * For example, 10:00AM December 1st, 2016 PST in epoch time is 1480615200 seconds, or 58406520 in hexadecimal digits.
    /// * A 96-bit identifier for the trace, globally unique, in 24 hexadecimal digits.
    ///
    /// See the [AWS X-Ray Documentation][xray-trace-id] for more details.
    ///
    /// [xray-trace-id]: https://docs.aws.amazon.com/xray/latest/devguide/xray-api-sendingdata.html#xray-api-traceids
    #[derive(Clone, Debug, PartialEq)]
    struct XrayTraceId(String);

    impl TryFrom<XrayTraceId> for TraceId {
        type Error = ();

        fn try_from(id: XrayTraceId) -> Result<Self, Self::Error> {
            let parts: Vec<&str> = id.0.split_terminator('-').collect();

            if parts.len() != 3 {
                return Err(());
            }

            let trace_id: TraceId =
                TraceId::from_hex(format!("{}{}", parts[1], parts[2]).as_str()).map_err(|_| ())?;

            if trace_id == TraceId::INVALID {
                Err(())
            } else {
                Ok(trace_id)
            }
        }
    }

    impl From<TraceId> for XrayTraceId {
        fn from(trace_id: TraceId) -> Self {
            let trace_id_as_hex = trace_id.to_string();
            let (timestamp, xray_id) = trace_id_as_hex.split_at(8_usize);

            XrayTraceId(format!(
                "{}-{}-{}",
                AWS_XRAY_VERSION_KEY, timestamp, xray_id
            ))
        }
    }

    fn from_key_value_pair(pair: &str) -> Option<(&str, &str)> {
        let mut key_value_pair: Option<(&str, &str)> = None;

        if let Some(index) = pair.find('=') {
            let (key, value) = pair.split_at(index);
            key_value_pair = Some((key, value.trim_start_matches('=')));
        }
        key_value_pair
    }

    fn title_case(s: &str) -> String {
        let mut capitalized: String = String::with_capacity(s.len());

        if !s.is_empty() {
            let mut characters = s.chars();

            if let Some(first) = characters.next() {
                capitalized.push(first.to_ascii_uppercase())
            }
            capitalized.extend(characters);
        }

        capitalized
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use opentelemetry::testing::trace::TestSpan;
        use opentelemetry::trace::TraceState;
        use std::collections::HashMap;
        use std::str::FromStr;

        #[rustfmt::skip]
        fn extract_test_data() -> Vec<(&'static str, SpanContext)> {
            vec![
                ("", SpanContext::empty_context()),
                ("Sampled=1;Self=foo", SpanContext::empty_context()),
                ("Root=1-bogus-bad", SpanContext::empty_context()),
                ("Root=1-too-many-parts", SpanContext::empty_context()),
                ("Root=1-58406520-a006649127e371903a2de979;Parent=garbage", SpanContext::new(TraceId::from_hex("58406520a006649127e371903a2de979").unwrap(), SpanId::INVALID, TRACE_FLAG_DEFERRED, true, TraceState::default())),
                ("Root=1-58406520-a006649127e371903a2de979;Sampled=1", SpanContext::new(TraceId::from_hex("58406520a006649127e371903a2de979").unwrap(), SpanId::INVALID, TraceFlags::SAMPLED, true, TraceState::default())),
                ("Root=1-58406520-a006649127e371903a2de979;Parent=4c721bf33e3caf8f;Sampled=0", SpanContext::new(TraceId::from_hex("58406520a006649127e371903a2de979").unwrap(), SpanId::from_hex("4c721bf33e3caf8f").unwrap(), TraceFlags::default(), true, TraceState::default())),
                ("Root=1-58406520-a006649127e371903a2de979;Parent=4c721bf33e3caf8f;Sampled=1", SpanContext::new(TraceId::from_hex("58406520a006649127e371903a2de979").unwrap(), SpanId::from_hex("4c721bf33e3caf8f").unwrap(), TraceFlags::SAMPLED, true, TraceState::default())),
                ("Root=1-58406520-a006649127e371903a2de979;Parent=4c721bf33e3caf8f", SpanContext::new(TraceId::from_hex("58406520a006649127e371903a2de979").unwrap(), SpanId::from_hex("4c721bf33e3caf8f").unwrap(), TRACE_FLAG_DEFERRED, true, TraceState::default())),
                ("Root=1-58406520-a006649127e371903a2de979;Parent=4c721bf33e3caf8f;Sampled=?", SpanContext::new(TraceId::from_hex("58406520a006649127e371903a2de979").unwrap(), SpanId::from_hex("4c721bf33e3caf8f").unwrap(), TRACE_FLAG_DEFERRED, true, TraceState::default())),
                ("Root=1-58406520-a006649127e371903a2de979;Self=1-58406520-bf42676c05e20ba4a90e448e;Parent=4c721bf33e3caf8f;Sampled=1", SpanContext::new(TraceId::from_hex("58406520a006649127e371903a2de979").unwrap(), SpanId::from_hex("4c721bf33e3caf8f").unwrap(), TraceFlags::SAMPLED, true, TraceState::from_str("self=1-58406520-bf42676c05e20ba4a90e448e").unwrap())),
                ("Root=1-58406520-a006649127e371903a2de979;Self=1-58406520-bf42676c05e20ba4a90e448e;Parent=4c721bf33e3caf8f;Sampled=1;RandomKey=RandomValue", SpanContext::new(TraceId::from_hex("58406520a006649127e371903a2de979").unwrap(), SpanId::from_hex("4c721bf33e3caf8f").unwrap(), TraceFlags::SAMPLED, true, TraceState::from_str("self=1-58406520-bf42676c05e20ba4a90e448e,randomkey=RandomValue").unwrap())),
            ]
        }

        #[rustfmt::skip]
        fn inject_test_data() -> Vec<(&'static str, SpanContext)> {
            vec![
                ("", SpanContext::empty_context()),
                ("", SpanContext::new(TraceId::INVALID, SpanId::INVALID, TRACE_FLAG_DEFERRED, true, TraceState::default())),
                ("", SpanContext::new(TraceId::from_hex("58406520a006649127e371903a2de979").unwrap(), SpanId::INVALID, TRACE_FLAG_DEFERRED, true, TraceState::default())),
                ("", SpanContext::new(TraceId::from_hex("58406520a006649127e371903a2de979").unwrap(), SpanId::INVALID, TraceFlags::SAMPLED, true, TraceState::default())),
                ("Root=1-58406520-a006649127e371903a2de979;Parent=4c721bf33e3caf8f;Sampled=0", SpanContext::new(TraceId::from_hex("58406520a006649127e371903a2de979").unwrap(), SpanId::from_hex("4c721bf33e3caf8f").unwrap(), TraceFlags::default(), true, TraceState::default())),
                ("Root=1-58406520-a006649127e371903a2de979;Parent=4c721bf33e3caf8f;Sampled=1", SpanContext::new(TraceId::from_hex("58406520a006649127e371903a2de979").unwrap(), SpanId::from_hex("4c721bf33e3caf8f").unwrap(), TraceFlags::SAMPLED, true, TraceState::default())),
                ("Root=1-58406520-a006649127e371903a2de979;Parent=4c721bf33e3caf8f;Sampled=?;Self=1-58406520-bf42676c05e20ba4a90e448e;Randomkey=RandomValue", SpanContext::new(TraceId::from_hex("58406520a006649127e371903a2de979").unwrap(), SpanId::from_hex("4c721bf33e3caf8f").unwrap(), TRACE_FLAG_DEFERRED, true, TraceState::from_str("self=1-58406520-bf42676c05e20ba4a90e448e,randomkey=RandomValue").unwrap())),
            ]
        }

        #[test]
        fn test_extract() {
            for (header, expected) in extract_test_data() {
                let map: HashMap<String, String> =
                    vec![(AWS_XRAY_TRACE_HEADER.to_string(), header.to_string())]
                        .into_iter()
                        .collect();

                let propagator = XrayPropagator::default();
                let context = propagator.extract(&map);
                assert_eq!(context.span().span_context(), &expected);
            }
        }

        #[test]
        fn test_extract_empty() {
            let map: HashMap<String, String> = HashMap::new();
            let propagator = XrayPropagator::default();
            let context = propagator.extract(&map);
            assert_eq!(context.span().span_context(), &SpanContext::empty_context())
        }

        #[test]
        fn test_inject() {
            let propagator = XrayPropagator::default();
            for (header_value, span_context) in inject_test_data() {
                let mut injector: HashMap<String, String> = HashMap::new();
                propagator.inject_context(
                    &Context::current_with_span(TestSpan(span_context)),
                    &mut injector,
                );

                let injected_value: Option<&String> = injector.get(AWS_XRAY_TRACE_HEADER);

                if header_value.is_empty() {
                    assert!(injected_value.is_none());
                } else {
                    assert_eq!(injected_value, Some(&header_value.to_string()));
                }
            }
        }
    }
}

pub use trace::XrayPropagator;
