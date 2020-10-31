use criterion::{criterion_group, criterion_main, Criterion};
use opentelemetry::{
    sdk::trace as sdktrace,
    trace::{Span, Tracer, TracerProvider},
    Key,
};

fn criterion_benchmark(c: &mut Criterion) {
    trace_benchmark_group(c, "start-end-span", |tracer| tracer.start("foo").end());

    trace_benchmark_group(c, "start-end-span-4-attrs", |tracer| {
        let span = tracer.start("foo");
        span.set_attribute(Key::new("key1").bool(false));
        span.set_attribute(Key::new("key2").string("hello"));
        span.set_attribute(Key::new("key4").f64(123.456));
        span.end();
    });

    trace_benchmark_group(c, "start-end-span-8-attrs", |tracer| {
        let span = tracer.start("foo");
        span.set_attribute(Key::new("key1").bool(false));
        span.set_attribute(Key::new("key2").string("hello"));
        span.set_attribute(Key::new("key4").f64(123.456));
        span.set_attribute(Key::new("key11").bool(false));
        span.set_attribute(Key::new("key12").string("hello"));
        span.set_attribute(Key::new("key14").f64(123.456));
        span.end();
    });

    trace_benchmark_group(c, "start-end-span-all-attr-types", |tracer| {
        let span = tracer.start("foo");
        span.set_attribute(Key::new("key1").bool(false));
        span.set_attribute(Key::new("key2").string("hello"));
        span.set_attribute(Key::new("key3").i64(123));
        span.set_attribute(Key::new("key5").f64(123.456));
        span.end();
    });

    trace_benchmark_group(c, "start-end-span-all-attr-types-2x", |tracer| {
        let span = tracer.start("foo");
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

fn trace_benchmark_group<F: Fn(&sdktrace::Tracer)>(c: &mut Criterion, name: &str, f: F) {
    let mut group = c.benchmark_group(name);

    group.bench_function("always-sample", |b| {
        let always_sample = sdktrace::TracerProvider::builder()
            .with_config(sdktrace::Config {
                default_sampler: Box::new(sdktrace::Sampler::AlwaysOn),
                ..Default::default()
            })
            .build()
            .get_tracer("always-sample", None);

        b.iter(|| f(&always_sample));
    });

    group.bench_function("never-sample", |b| {
        let never_sample = sdktrace::TracerProvider::builder()
            .with_config(sdktrace::Config {
                default_sampler: Box::new(sdktrace::Sampler::AlwaysOff),
                ..Default::default()
            })
            .build()
            .get_tracer("never-sample", None);
        b.iter(|| f(&never_sample));
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
