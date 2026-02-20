/*
    Span Creation scenarios.
    This benchmark measures the performance cost of different
    span creation patterns in OpenTelemetry when using On/Off
    sampling strategies.

    TODO: Cover the impact of the presence of ActiveSpan in the context.

    The benchmark results:
    criterion = "0.5.1"
    cargo 1.91.1 (ea2d97820 2025-10-10)
    Hardware: M4Pro
    | Test                                                  | Always Sample | Never Sample |
    |-------------------------------------------------------|---------------|--------------|
    | span-creation-simple                                  | 203.67 ns     | 51.316 ns    |
    | span-creation-span-builder                            | 209.55 ns     | 88.032 ns    |
    | span-creation-tracer-in-span                          | 339.52 ns     | 150.55 ns    |
    | span-creation-tracer-in-span-with-builder             | 317.44 ns     | 170.38 ns    |
    | span-creation-simple-context-activation               | 329.41 ns     | 47.681 ns    |
    | span-creation-span-builder-context-activation         | 328.84 ns     | 78.021 ns    |
*/

use criterion::{criterion_group, criterion_main, Criterion};
use opentelemetry::{
    trace::{mark_span_as_active, Span, SpanBuilder, TraceContextExt, Tracer, TracerProvider},
    Context, KeyValue,
};
use opentelemetry_sdk::{
    error::OTelSdkResult,
    trace::{self as sdktrace, SpanBatch, SpanExporter},
};
#[cfg(all(not(target_os = "windows"), feature = "bench_profiling"))]
use pprof::criterion::{Output, PProfProfiler};

fn criterion_benchmark(c: &mut Criterion) {
    trace_benchmark_group(c, "span-creation-simple", |tracer| {
        // Simple span creation
        // There is not ability to specify anything other than the name.
        // Attributes are set after creation, and automatically gets
        // ignored if sampling is unfavorable.
        let mut span = tracer.start("span-name");
        if span.is_recording() {
            span.set_attribute(KeyValue::new("key1", false));
            span.set_attribute(KeyValue::new("key2", "hello"));
            span.set_attribute(KeyValue::new("key3", 123.456));
            span.set_attribute(KeyValue::new("key4", "world"));
            span.set_attribute(KeyValue::new("key5", 123));
        }
        span.end();
    });

    trace_benchmark_group(c, "span-creation-span-builder", |tracer| {
        // This is similar to the simple span creation, but allows
        // attributes and other properties to be set during creation.
        // It is slightly slower than the simple span creation due to the fact that
        // attributes are collected into a vec! and allocated, even before sampling
        // decision is made.
        let mut span = tracer
            .span_builder("span-name")
            .with_attributes([
                KeyValue::new("key1", false),
                KeyValue::new("key2", "hello"),
                KeyValue::new("key3", 123.456),
            ])
            .start(tracer);
        if span.is_recording() {
            span.set_attribute(KeyValue::new("key4", "world"));
            span.set_attribute(KeyValue::new("key5", 123));
        }
        span.end();
    });

    trace_benchmark_group(c, "span-creation-tracer-in-span", |tracer| {
        // This is similar to the simple span creation, but also does the job of activating
        // the span in the current context.
        // It is slower than other approaches of activation due to the fact that
        // context activation is done, irrespective of sampling decision.
        tracer.in_span("span-name", |ctx| {
            let span = ctx.span();
            if span.is_recording() {
                span.set_attribute(KeyValue::new("key1", false));
                span.set_attribute(KeyValue::new("key2", "hello"));
                span.set_attribute(KeyValue::new("key3", 123.456));
                span.set_attribute(KeyValue::new("key4", "world"));
                span.set_attribute(KeyValue::new("key5", 123));
            }
        });
    });

    trace_benchmark_group(c, "span-creation-tracer-in-span-with-builder", |tracer| {
        // This is similar to the simple span creation, but also does the job of activating
        // the span in the current context.
        tracer.in_span_with_builder(
            SpanBuilder::from_name("span-name").with_attributes([
                KeyValue::new("key1", false),
                KeyValue::new("key2", "hello"),
                KeyValue::new("key3", 123.456),
            ]),
            |ctx| {
                let span = ctx.span();
                if span.is_recording() {
                    span.set_attribute(KeyValue::new("key4", "world"));
                    span.set_attribute(KeyValue::new("key5", 123));
                }
            },
        );
    });

    trace_benchmark_group(c, "span-creation-simple-context-activation", |tracer| {
        // This optimizes by bypassing the context activation
        // based on sampling decision, and hence it is faster than the
        // tracer.in_span approach.
        let mut span = tracer.start("span-name");
        if span.is_recording() {
            span.set_attribute(KeyValue::new("key1", false));
            span.set_attribute(KeyValue::new("key2", "hello"));
            span.set_attribute(KeyValue::new("key3", 123.456));
            let _guard = mark_span_as_active(span);
            Context::map_current(|cx| {
                let span_from_context = cx.span();
                span_from_context.set_attribute(KeyValue::new("key4", "world"));
                span_from_context.set_attribute(KeyValue::new("key5", 123));
            });
        }
    });

    trace_benchmark_group(
        c,
        "span-creation-span-builder-context-activation",
        |tracer| {
            // This optimizes by bypassing the context activation
            // based on sampling decision, and hence it is faster than the
            // tracer.in_span approach.
            let span = tracer
                .span_builder("span-name")
                .with_attributes([
                    KeyValue::new("key1", false),
                    KeyValue::new("key2", "hello"),
                    KeyValue::new("key3", 123.456),
                ])
                .start(tracer);
            if span.is_recording() {
                let _guard = mark_span_as_active(span);
                Context::map_current(|cx| {
                    let span_from_context = cx.span();
                    span_from_context.set_attribute(KeyValue::new("key4", "world"));
                    span_from_context.set_attribute(KeyValue::new("key5", 123));
                });
            }
        },
    );
}

#[derive(Debug)]
struct VoidExporter;

impl SpanExporter for VoidExporter {
    async fn export(&self, _spans: SpanBatch<'_>) -> OTelSdkResult {
        Ok(())
    }
}

fn trace_benchmark_group<F: Fn(&sdktrace::SdkTracer)>(c: &mut Criterion, name: &str, f: F) {
    let mut group = c.benchmark_group(name);

    group.bench_function("always-sample", |b| {
        let provider = sdktrace::SdkTracerProvider::builder()
            .with_sampler(sdktrace::Sampler::AlwaysOn)
            .with_simple_exporter(VoidExporter)
            .build();
        let always_sample = provider.tracer("always-sample");

        b.iter(|| f(&always_sample));
    });

    group.bench_function("never-sample", |b| {
        let provider = sdktrace::SdkTracerProvider::builder()
            .with_sampler(sdktrace::Sampler::AlwaysOff)
            .with_simple_exporter(VoidExporter)
            .build();
        let never_sample = provider.tracer("never-sample");
        b.iter(|| f(&never_sample));
    });

    group.finish();
}

#[cfg(all(not(target_os = "windows"), feature = "bench_profiling"))]
criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(std::time::Duration::from_secs(1))
        .measurement_time(std::time::Duration::from_secs(2))
        .with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = criterion_benchmark
}

#[cfg(any(target_os = "windows", not(feature = "bench_profiling")))]
criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(std::time::Duration::from_secs(1))
        .measurement_time(std::time::Duration::from_secs(2));
    targets = criterion_benchmark
}
criterion_main!(benches);
