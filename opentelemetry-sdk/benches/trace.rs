use criterion::{criterion_group, criterion_main, Criterion};
use futures_util::future::BoxFuture;
use opentelemetry_api::{
    trace::{Span, Tracer, TracerProvider},
    Key,
};
use opentelemetry_sdk::{
    export::trace::{ExportResult, SpanData, SpanExporter},
    trace as sdktrace,
};
#[cfg(not(target_os = "windows"))]
use pprof::criterion::{Output, PProfProfiler};

fn criterion_benchmark(c: &mut Criterion) {
    trace_benchmark_group(c, "start-end-span", |tracer| tracer.start("foo").end());

    trace_benchmark_group(c, "start-end-span-4-attrs", |tracer| {
        let mut span = tracer.start("foo");
        span.set_attribute(Key::new("key1").bool(false));
        span.set_attribute(Key::new("key2").string("hello"));
        span.set_attribute(Key::new("key4").f64(123.456));
        span.end();
    });

    trace_benchmark_group(c, "start-end-span-8-attrs", |tracer| {
        let mut span = tracer.start("foo");
        span.set_attribute(Key::new("key1").bool(false));
        span.set_attribute(Key::new("key2").string("hello"));
        span.set_attribute(Key::new("key4").f64(123.456));
        span.set_attribute(Key::new("key11").bool(false));
        span.set_attribute(Key::new("key12").string("hello"));
        span.set_attribute(Key::new("key14").f64(123.456));
        span.end();
    });

    trace_benchmark_group(c, "start-end-span-all-attr-types", |tracer| {
        let mut span = tracer.start("foo");
        span.set_attribute(Key::new("key1").bool(false));
        span.set_attribute(Key::new("key2").string("hello"));
        span.set_attribute(Key::new("key3").i64(123));
        span.set_attribute(Key::new("key5").f64(123.456));
        span.end();
    });

    trace_benchmark_group(c, "start-end-span-all-attr-types-2x", |tracer| {
        let mut span = tracer.start("foo");
        span.set_attribute(Key::new("key1").bool(false));
        span.set_attribute(Key::new("key2").string("hello"));
        span.set_attribute(Key::new("key3").i64(123));
        span.set_attribute(Key::new("key5").f64(123.456));
        span.set_attribute(Key::new("key11").bool(false));
        span.set_attribute(Key::new("key12").string("hello"));
        span.set_attribute(Key::new("key13").i64(123));
        span.set_attribute(Key::new("key15").f64(123.456));
        span.end();
    });
}

#[derive(Debug)]
struct VoidExporter;

impl SpanExporter for VoidExporter {
    fn export(&mut self, _spans: Vec<SpanData>) -> BoxFuture<'static, ExportResult> {
        Box::pin(futures_util::future::ready(Ok(())))
    }
}

fn trace_benchmark_group<F: Fn(&sdktrace::Tracer)>(c: &mut Criterion, name: &str, f: F) {
    let mut group = c.benchmark_group(name);

    group.bench_function("always-sample", |b| {
        let provider = sdktrace::TracerProvider::builder()
            .with_config(sdktrace::config().with_sampler(sdktrace::Sampler::AlwaysOn))
            .with_simple_exporter(VoidExporter)
            .build();
        let always_sample = provider.tracer("always-sample");

        b.iter(|| f(&always_sample));
    });

    group.bench_function("never-sample", |b| {
        let provider = sdktrace::TracerProvider::builder()
            .with_config(sdktrace::config().with_sampler(sdktrace::Sampler::AlwaysOff))
            .with_simple_exporter(VoidExporter)
            .build();
        let never_sample = provider.tracer("never-sample");
        b.iter(|| f(&never_sample));
    });

    group.finish();
}

#[cfg(not(target_os = "windows"))]
criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = criterion_benchmark
}
#[cfg(target_os = "windows")]
criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = criterion_benchmark
}
criterion_main!(benches);
