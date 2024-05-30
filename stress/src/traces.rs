/*
    Stress test results:
    OS: Ubuntu 22.04.3 LTS (5.15.146.1-microsoft-standard-WSL2)
    Hardware: AMD EPYC 7763 64-Core Processor - 2.44 GHz, 16vCPUs,
    RAM: 64.0 GB
    9 M/sec
*/

use lazy_static::lazy_static;
use opentelemetry::{
    trace::{Span, SpanBuilder, TraceResult, Tracer, TracerProvider as _},
    Context, KeyValue,
};
use opentelemetry_sdk::{
    export::trace::SpanData,
    trace::{self as sdktrace, SpanProcessor},
};

mod throughput;

lazy_static! {
    static ref PROVIDER: sdktrace::TracerProvider = sdktrace::TracerProvider::builder()
        .with_config(sdktrace::Config::default().with_sampler(sdktrace::Sampler::AlwaysOn))
        .with_span_processor(NoOpSpanProcessor {})
        .build();
    static ref TRACER: sdktrace::Tracer = PROVIDER.tracer("stress");
}

#[derive(Debug)]
pub struct NoOpSpanProcessor;

impl SpanProcessor for NoOpSpanProcessor {
    fn on_start(&self, _span: &mut opentelemetry_sdk::trace::Span, _cx: &Context) {
        // No-op
    }

    fn on_end(&self, _span: SpanData) {
        // No-op
    }

    fn force_flush(&self) -> TraceResult<()> {
        Ok(())
    }

    fn shutdown(&self) -> TraceResult<()> {
        Ok(())
    }
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
