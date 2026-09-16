use std::time::Duration;

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
  Chip:  Apple M4 Max
  Total Number of Cores: 16 (12 performance and 4 efficiency)

SpanProcessorApi/0_processors
    time:   [190.51 ns 190.82 ns 191.13 ns]
SpanProcessorApi/1_processors
    time:   [191.60 ns 192.53 ns 193.42 ns]
SpanProcessorApi/2_processors
    time:   [191.24 ns 191.67 ns 192.15 ns]
SpanProcessorApi/4_processors
    time:   [192.94 ns 193.48 ns 194.12 ns]
*/

#[derive(Debug)]
struct NoopSpanProcessor;

impl sdktrace::SpanProcessor for NoopSpanProcessor {
    fn on_start(&self, _span: &mut sdktrace::Span, _parent_cx: &Context) {}
    fn on_end(&self, _span: sdktrace::FinishedSpan<'_>) {}
    fn force_flush(&self) -> opentelemetry_sdk::error::OTelSdkResult {
        Ok(())
    }
    fn shutdown_with_timeout(&self, _timeout: Duration) -> opentelemetry_sdk::error::OTelSdkResult {
        Ok(())
    }
}

#[derive(Debug)]
struct ReadOnlySpanProcessor;

impl sdktrace::SpanProcessor for ReadOnlySpanProcessor {
    fn on_start(&self, _span: &mut sdktrace::Span, _parent_cx: &Context) {}

    fn on_end(&self, span: sdktrace::FinishedSpan<'_>) {
        black_box(span.span_data().attributes.len());
    }

    fn force_flush(&self) -> opentelemetry_sdk::error::OTelSdkResult {
        Ok(())
    }

    fn shutdown_with_timeout(&self, _timeout: Duration) -> opentelemetry_sdk::error::OTelSdkResult {
        Ok(())
    }
}

#[derive(Debug)]
struct OwningSpanProcessor;

impl sdktrace::SpanProcessor for OwningSpanProcessor {
    fn on_start(&self, _span: &mut sdktrace::Span, _parent_cx: &Context) {}

    fn on_end(&self, span: sdktrace::FinishedSpan<'_>) {
        black_box(span.into_owned());
    }

    fn force_flush(&self) -> opentelemetry_sdk::error::OTelSdkResult {
        Ok(())
    }

    fn shutdown_with_timeout(&self, _timeout: Duration) -> opentelemetry_sdk::error::OTelSdkResult {
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

fn create_populated_span(tracer: &sdktrace::Tracer) -> sdktrace::Span {
    let mut span = tracer.start("ownership-benchmark");
    for (index, key) in [
        "benchmark.attribute.00",
        "benchmark.attribute.01",
        "benchmark.attribute.02",
        "benchmark.attribute.03",
        "benchmark.attribute.04",
        "benchmark.attribute.05",
        "benchmark.attribute.06",
        "benchmark.attribute.07",
        "benchmark.attribute.08",
        "benchmark.attribute.09",
        "benchmark.attribute.10",
        "benchmark.attribute.11",
        "benchmark.attribute.12",
        "benchmark.attribute.13",
        "benchmark.attribute.14",
        "benchmark.attribute.15",
        "benchmark.attribute.16",
        "benchmark.attribute.17",
        "benchmark.attribute.18",
        "benchmark.attribute.19",
        "benchmark.attribute.20",
        "benchmark.attribute.21",
        "benchmark.attribute.22",
        "benchmark.attribute.23",
    ]
    .iter()
    .enumerate()
    {
        span.set_attribute(KeyValue::new(*key, index as i64));
    }
    for event_index in 0..4 {
        span.add_event(
            "benchmark.event",
            vec![
                KeyValue::new("benchmark.event.index", event_index as i64),
                KeyValue::new("benchmark.event.kind", "ownership"),
                KeyValue::new("benchmark.event.enabled", true),
            ],
        );
    }
    span
}

#[derive(Clone, Copy)]
enum OwnershipConfiguration {
    OneReadOnly,
    OneOwning,
    TwoReadOnly,
    ReadOnlyThenOwning,
    OwningThenReadOnly,
    TwoOwning,
}

impl OwnershipConfiguration {
    fn name(self) -> &'static str {
        match self {
            Self::OneReadOnly => "one_readonly",
            Self::OneOwning => "one_owning",
            Self::TwoReadOnly => "two_readonly",
            Self::ReadOnlyThenOwning => "readonly_then_owning",
            Self::OwningThenReadOnly => "owning_then_readonly",
            Self::TwoOwning => "two_owning",
        }
    }

    fn tracer(self) -> sdktrace::SdkTracer {
        let builder = sdktrace::SdkTracerProvider::builder();
        match self {
            Self::OneReadOnly => builder.with_span_processor(ReadOnlySpanProcessor).build(),
            Self::OneOwning => builder.with_span_processor(OwningSpanProcessor).build(),
            Self::TwoReadOnly => builder
                .with_span_processor(ReadOnlySpanProcessor)
                .with_span_processor(ReadOnlySpanProcessor)
                .build(),
            Self::ReadOnlyThenOwning => builder
                .with_span_processor(ReadOnlySpanProcessor)
                .with_span_processor(OwningSpanProcessor)
                .build(),
            Self::OwningThenReadOnly => builder
                .with_span_processor(OwningSpanProcessor)
                .with_span_processor(ReadOnlySpanProcessor)
                .build(),
            Self::TwoOwning => builder
                .with_span_processor(OwningSpanProcessor)
                .with_span_processor(OwningSpanProcessor)
                .build(),
        }
        .tracer("ownership-benchmark")
    }
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

    group.finish();

    let mut ownership_group = c.benchmark_group("FinishedSpanOwnership");
    for configuration in [
        OwnershipConfiguration::OneReadOnly,
        OwnershipConfiguration::OneOwning,
        OwnershipConfiguration::TwoReadOnly,
        OwnershipConfiguration::ReadOnlyThenOwning,
        OwnershipConfiguration::OwningThenReadOnly,
        OwnershipConfiguration::TwoOwning,
    ] {
        ownership_group.bench_function(configuration.name(), |b| {
            let tracer = configuration.tracer();
            b.iter(|| drop(black_box(create_populated_span(&tracer))));
        });
    }
    ownership_group.finish();
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
