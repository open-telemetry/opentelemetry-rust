use criterion::{black_box, criterion_group, criterion_main, Criterion};
use opentelemetry::{
    trace::{Span, Tracer, TracerProvider},
    Context, KeyValue,
};
use opentelemetry_sdk::trace as sdktrace;

#[cfg(not(target_os = "windows"))]
use pprof::criterion::{Output, PProfProfiler};

/*
Adding results in comments for a quick reference.
  Chip:	Apple M1 Max
  Total Number of Cores:	10 (8 performance and 2 efficiency)

SpanProcessorApi/0_processors
    time:   [385.15 ns 386.14 ns 387.25 ns]
SpanProcessorApi/1_processors
    time:   [385.73 ns 387.17 ns 388.85 ns]
SpanProcessorApi/2_processors
    time:   [384.84 ns 385.66 ns 386.50 ns]
SpanProcessorApi/4_processors
    time:   [386.78 ns 388.17 ns 389.58 ns]
*/

#[derive(Debug)]
struct NoopSpanProcessor;

impl sdktrace::SpanProcessor for NoopSpanProcessor {
    fn on_start(&self, _span: &mut sdktrace::Span, _parent_cx: &Context) {}
    fn on_end(&self, _span: &mut sdktrace::FinishedSpan) {}
    fn force_flush(&self) -> opentelemetry_sdk::error::OTelSdkResult {
        Ok(())
    }
    fn shutdown(&self) -> opentelemetry_sdk::error::OTelSdkResult {
        Ok(())
    }
}

fn create_tracer(span_processors_count: usize) -> sdktrace::SdkTracer {
    let mut builder = sdktrace::SdkTracerProvider::builder();
    for _ in 0..span_processors_count {
        builder = builder.with_span_processor(NoopSpanProcessor);
    }
    builder.build().tracer("tracer")
}

fn create_span(tracer: &sdktrace::Tracer) -> sdktrace::Span {
    let mut span = tracer.start("foo");
    span.set_attribute(KeyValue::new("key1", false));
    span.set_attribute(KeyValue::new("key2", "hello"));
    span.set_attribute(KeyValue::new("key4", 123.456));
    span.add_event("my_event", vec![KeyValue::new("key1", "value1")]);
    span
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("SpanProcessorApi");
    for i in [0, 1, 2, 4] {
        group.bench_function(format!("{}_processors", i), |b| {
            let tracer = create_tracer(i);
            b.iter(|| {
                black_box(create_span(&tracer));
            });
        });
    }
}

#[cfg(not(target_os = "windows"))]
criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)))
                               .warm_up_time(std::time::Duration::from_secs(1))
                               .measurement_time(std::time::Duration::from_secs(2));
    targets = criterion_benchmark
}

#[cfg(target_os = "windows")]
criterion_group! {
    name = benches;
    config = Criterion::default().warm_up_time(std::time::Duration::from_secs(1))
                               .measurement_time(std::time::Duration::from_secs(2));
    targets = criterion_benchmark
}

criterion_main!(benches);
