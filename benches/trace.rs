use criterion::{criterion_group, criterion_main, Criterion};
use opentelemetry::{
    api::{Key, Provider, Span, Tracer},
    sdk,
};

fn criterion_benchmark(c: &mut Criterion) {
    trace_benchmark_group(c, "start-end-span", |tracer| {
        tracer.start("foo", None).end()
    });

    trace_benchmark_group(c, "start-end-span-4-attrs", |tracer| {
        let mut span = tracer.start("foo", None);
        span.set_attribute(Key::new("key1").bool(false));
        span.set_attribute(Key::new("key2").string("hello"));
        span.set_attribute(Key::new("key3").u64(123));
        span.set_attribute(Key::new("key4").f64(123.456));
        span.end();
    });

    trace_benchmark_group(c, "start-end-span-8-attrs", |tracer| {
        let mut span = tracer.start("foo", None);
        span.set_attribute(Key::new("key1").bool(false));
        span.set_attribute(Key::new("key2").string("hello"));
        span.set_attribute(Key::new("key3").u64(123));
        span.set_attribute(Key::new("key4").f64(123.456));
        span.set_attribute(Key::new("key11").bool(false));
        span.set_attribute(Key::new("key12").string("hello"));
        span.set_attribute(Key::new("key13").u64(123));
        span.set_attribute(Key::new("key14").f64(123.456));
        span.end();
    });

    trace_benchmark_group(c, "start-end-span-all-attr-types", |tracer| {
        let mut span = tracer.start("foo", None);
        span.set_attribute(Key::new("key1").bool(false));
        span.set_attribute(Key::new("key2").string("hello"));
        span.set_attribute(Key::new("key3").i64(123));
        span.set_attribute(Key::new("key4").u64(123));
        span.set_attribute(Key::new("key5").f64(123.456));
        span.set_attribute(
            Key::new("key6").bytes(vec![104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100]),
        );
        span.end();
    });

    trace_benchmark_group(c, "start-end-span-all-attr-types-2x", |tracer| {
        let mut span = tracer.start("foo", None);
        span.set_attribute(Key::new("key1").bool(false));
        span.set_attribute(Key::new("key2").string("hello"));
        span.set_attribute(Key::new("key3").i64(123));
        span.set_attribute(Key::new("key4").u64(123));
        span.set_attribute(Key::new("key5").f64(123.456));
        span.set_attribute(
            Key::new("key6").bytes(vec![104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100]),
        );
        span.set_attribute(Key::new("key11").bool(false));
        span.set_attribute(Key::new("key12").string("hello"));
        span.set_attribute(Key::new("key13").i64(123));
        span.set_attribute(Key::new("key14").u64(123));
        span.set_attribute(Key::new("key15").f64(123.456));
        span.set_attribute(
            Key::new("key16").bytes(vec![104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100]),
        );
        span.end();
    });
}

fn trace_benchmark_group<F: Fn(&sdk::Tracer) -> ()>(c: &mut Criterion, name: &str, f: F) {
    let mut group = c.benchmark_group(name);

    group.bench_function("always-sample", |b| {
        let always_sample = sdk::Provider::builder()
            .with_config(sdk::Config {
                default_sampler: Box::new(sdk::Sampler::Always),
                ..Default::default()
            })
            .build()
            .get_tracer("always-sample");

        b.iter(|| f(&always_sample));
    });

    group.bench_function("never-sample", |b| {
        let never_sample = sdk::Provider::builder()
            .with_config(sdk::Config {
                default_sampler: Box::new(sdk::Sampler::Never),
                ..Default::default()
            })
            .build()
            .get_tracer("never-sample");
        b.iter(|| f(&never_sample));
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
