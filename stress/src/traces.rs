use lazy_static::lazy_static;
use opentelemetry::{
    trace::{Span, SpanBuilder, Tracer, TracerProvider as _},
    KeyValue,
};
use opentelemetry_sdk::trace as sdktrace;

mod throughput;

lazy_static! {
    static ref PROVIDER: sdktrace::TracerProvider = sdktrace::TracerProvider::builder()
        .with_config(sdktrace::config().with_sampler(sdktrace::Sampler::AlwaysOn))
        .build();
    static ref TRACER: sdktrace::Tracer = PROVIDER.tracer("stress");
}

fn main() {
    throughput::test_throughput(test_span);
}

fn test_span() {
    let span_builder = SpanBuilder::from_name("test_span").with_attributes(vec![
        KeyValue::new("attribute_at_span_start1", "value1"),
        KeyValue::new("attribute_at_span_start2", "value2"),
    ]);

    let mut span = TRACER.build(span_builder);
    span.set_attribute(KeyValue::new("key3", "value3"));
    span.set_attribute(KeyValue::new("key4", "value4"));
    span.end();
}
