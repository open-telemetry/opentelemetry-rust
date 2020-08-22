use opentelemetry::api;
use opentelemetry::api::{
    Context, HttpTextCompositePropagator, HttpTextFormat, SpanContext, SpanId, TraceContextExt,
    TraceContextPropagator, TraceId,
};
use opentelemetry_contrib::trace_propagator::{B3Encoding, B3Propagator};
use std::collections::HashMap;

fn test_data() -> Vec<(&'static str, &'static str)> {
    vec![
        ("b3", "00000000000000000000000000000001-0000000000000001-0"),
        (
            "traceparent",
            "00-00000000000000000000000000000001-0000000000000001-00",
        ),
    ]
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
fn inject_multiple_propagators() {
    let b3 = B3Propagator::with_encoding(B3Encoding::SingleHeader);
    let trace_context = TraceContextPropagator::new();
    let composite_propagator =
        HttpTextCompositePropagator::new(vec![Box::new(b3), Box::new(trace_context)]);

    let cx = Context::default().with_span(TestSpan(SpanContext::new(
        TraceId::from_u128(1),
        SpanId::from_u64(1),
        0,
        false,
    )));
    let mut injector = HashMap::new();
    composite_propagator.inject_context(&cx, &mut injector);

    for (header_name, header_value) in test_data() {
        assert_eq!(injector.get(header_name), Some(&header_value.to_string()));
    }
}

#[test]
fn extract_multiple_propagators() {
    let b3 = B3Propagator::with_encoding(B3Encoding::SingleHeader);
    let trace_context = TraceContextPropagator::new();
    let composite_propagator =
        HttpTextCompositePropagator::new(vec![Box::new(b3), Box::new(trace_context)]);

    for (header_name, header_value) in test_data() {
        let mut extractor = HashMap::new();
        extractor.insert(header_name.to_string(), header_value.to_string());
        assert_eq!(
            composite_propagator
                .extract(&extractor)
                .remote_span_context(),
            Some(&SpanContext::new(
                TraceId::from_u128(1),
                SpanId::from_u64(1),
                0,
                true,
            ))
        );
    }
}

#[test]
fn test_get_fields() {
    let b3 = B3Propagator::with_encoding(B3Encoding::SingleHeader);
    let b3_fields = b3.fields().map(|s| s.to_string()).collect::<Vec<String>>();

    let trace_context = TraceContextPropagator::new();
    let trace_context_fields = trace_context
        .fields()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    let composite_propagator =
        HttpTextCompositePropagator::new(vec![Box::new(b3), Box::new(trace_context)]);

    let mut fields = composite_propagator
        .fields()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    fields.sort();

    let mut expected = vec![b3_fields, trace_context_fields]
        .into_iter()
        .flatten()
        .collect::<Vec<String>>();
    expected.sort();
    expected.dedup();

    assert_eq!(fields, expected);
}
