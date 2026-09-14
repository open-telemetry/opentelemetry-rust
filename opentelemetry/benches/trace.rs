/*
    Benchmark Results:
    criterion = "0.5.1"
    OS: macOS 15
    Hardware: Apple M4 Pro, 14 cores, RAM: 24 GB
    | Test                                             | Average time|
    |--------------------------------------------------|-------------|
    | CreateSpan_NoopTracer                            |    29.2 ns  |
    | CreateSpan_NoopTracer_4Attributes                |    29.4 ns  |
    | CreateSpan_NoopTracer_AddEvent                   |    29.2 ns  |
    | CreateSpan_NoopTracer_AddLink                    |    29.2 ns  |
    | CreateSpan_NoopTracer_SetActive                  |    73.3 ns  |
    | CreateSpan_NoopTracer_WithActiveParent           |   102.9 ns  |
    | CreateSpan_NoopTracer_InSpan                     |    78.0 ns  |
    | SpanBuilder_Creation                             |    22.8 ns  |
    | SpanBuilder_WithAttributes                       |    51.7 ns  |
    | SpanBuilder_WithLinks                            |    69.0 ns  |
*/

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use opentelemetry::{
    global,
    trace::{
        noop::NoopTracerProvider, Span, SpanContext, SpanId, SpanKind, TraceFlags, TraceId,
        TraceState, Tracer,
    },
    KeyValue,
};

// Run this benchmark with:
// cargo bench --bench trace --features=trace

fn valid_span_context() -> SpanContext {
    SpanContext::new(
        TraceId::from(12345u128),
        SpanId::from(12345u64),
        TraceFlags::SAMPLED,
        true,
        TraceState::default(),
    )
}

fn criterion_benchmark(c: &mut Criterion) {
    global::set_tracer_provider(NoopTracerProvider::new());
    let tracer = global::tracer("bench");

    span_lifecycle(c, &tracer);
    span_builder(c, &tracer);
}

fn span_lifecycle(
    c: &mut Criterion,
    tracer: &impl Tracer<Span = impl Span + Send + Sync + 'static>,
) {
    c.bench_function("CreateSpan_NoopTracer", |b| {
        b.iter(|| {
            let mut span = tracer.start("test-span");
            span.end();
        });
    });

    c.bench_function("CreateSpan_NoopTracer_4Attributes", |b| {
        b.iter(|| {
            let mut span = tracer.start("test-span");
            if span.is_recording() {
                span.set_attribute(KeyValue::new("key1", false));
                span.set_attribute(KeyValue::new("key2", "hello"));
                span.set_attribute(KeyValue::new("key3", 123));
                span.set_attribute(KeyValue::new("key4", 123.456));
            }
            span.end();
        });
    });

    c.bench_function("CreateSpan_NoopTracer_AddEvent", |b| {
        b.iter(|| {
            let mut span = tracer.start("test-span");
            if span.is_recording() {
                span.add_event(
                    "my-event",
                    vec![
                        KeyValue::new("key1", "value1"),
                        KeyValue::new("key2", "value2"),
                    ],
                );
            }
            span.end();
        });
    });

    c.bench_function("CreateSpan_NoopTracer_AddLink", |b| {
        let link_context = valid_span_context();
        b.iter(|| {
            let mut span = tracer.start("test-span");
            if span.is_recording() {
                span.add_link(
                    link_context.clone(),
                    vec![
                        KeyValue::new("key1", "value1"),
                        KeyValue::new("key2", "value2"),
                    ],
                );
            }
            span.end();
        });
    });

    c.bench_function("CreateSpan_NoopTracer_SetActive", |b| {
        b.iter(|| {
            let span = tracer.start("test-span");
            let _guard = opentelemetry::trace::mark_span_as_active(span);
        });
    });

    c.bench_function("CreateSpan_NoopTracer_WithActiveParent", |b| {
        b.iter(|| {
            let parent = tracer.start("parent");
            let _guard = opentelemetry::trace::mark_span_as_active(parent);
            let mut child = tracer.start("child");
            child.end();
        });
    });

    c.bench_function("CreateSpan_NoopTracer_InSpan", |b| {
        b.iter(|| {
            tracer.in_span("test-span", |_cx| {});
        });
    });
}

fn span_builder(c: &mut Criterion, tracer: &impl Tracer<Span = impl Span + Send + Sync + 'static>) {
    c.bench_function("SpanBuilder_Creation", |b| {
        b.iter(|| {
            let span = tracer
                .span_builder("test-span")
                .with_kind(SpanKind::Client)
                .start(tracer);
            black_box(span);
        });
    });

    c.bench_function("SpanBuilder_WithAttributes", |b| {
        b.iter(|| {
            let span = tracer
                .span_builder("test-span")
                .with_kind(SpanKind::Client)
                .with_attributes(vec![
                    KeyValue::new("key1", false),
                    KeyValue::new("key2", "hello"),
                    KeyValue::new("key3", 123),
                    KeyValue::new("key4", 123.456),
                ])
                .start(tracer);
            black_box(span);
        });
    });

    c.bench_function("SpanBuilder_WithLinks", |b| {
        let link_context = valid_span_context();
        b.iter(|| {
            let span = tracer
                .span_builder("test-span")
                .with_links(vec![opentelemetry::trace::Link::new(
                    link_context.clone(),
                    vec![
                        KeyValue::new("key1", "value1"),
                        KeyValue::new("key2", "value2"),
                    ],
                    0,
                )])
                .start(tracer);
            black_box(span);
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(std::time::Duration::from_secs(1))
        .measurement_time(std::time::Duration::from_secs(2));
    targets = criterion_benchmark
}
criterion_main!(benches);
