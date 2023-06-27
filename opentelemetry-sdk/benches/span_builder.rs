use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use futures_util::future::BoxFuture;
use opentelemetry_api::{
    trace::{OrderMap, Span, Tracer, TracerProvider},
    KeyValue,
};
use opentelemetry_sdk::{
    export::trace::{ExportResult, SpanData, SpanExporter},
    trace as sdktrace,
};
use pprof::criterion::{Output, PProfProfiler};

fn criterion_benchmark(c: &mut Criterion) {
    span_builder_benchmark_group(c)
}

fn span_builder_benchmark_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("span_builder");
    group.bench_function("simplest", |b| {
        let (_provider, tracer) = not_sampled_provider();
        b.iter(|| {
            let mut span = tracer.span_builder("span").start(&tracer);
            span.end();
        })
    });
    group.bench_function(BenchmarkId::new("with_attributes", "1"), |b| {
        let (_provider, tracer) = not_sampled_provider();
        b.iter(|| {
            let mut span = tracer
                .span_builder("span")
                .with_attributes([KeyValue::new(MAP_KEYS[0], "value")])
                .start(&tracer);
            span.end();
        })
    });
    group.bench_function(BenchmarkId::new("with_attributes", "4"), |b| {
        let (_provider, tracer) = not_sampled_provider();
        b.iter(|| {
            let mut span = tracer
                .span_builder("span")
                .with_attributes([
                    KeyValue::new(MAP_KEYS[0], "value"),
                    KeyValue::new(MAP_KEYS[1], "value"),
                    KeyValue::new(MAP_KEYS[2], "value"),
                    KeyValue::new(MAP_KEYS[3], "value"),
                ])
                .start(&tracer);
            span.end();
        })
    });
    group.bench_function(BenchmarkId::new("with_attributes_map", "1"), |b| {
        let (_provider, tracer) = not_sampled_provider();
        b.iter(|| {
            let mut span = tracer
                .span_builder("span")
                .with_attributes_map(OrderMap::from_iter([KeyValue::new(MAP_KEYS[0], "value")]))
                .start(&tracer);
            span.end();
        })
    });
    group.bench_function(BenchmarkId::new("with_attributes_map", "4"), |b| {
        let (_provider, tracer) = not_sampled_provider();
        b.iter(|| {
            let mut span = tracer
                .span_builder("span")
                .with_attributes_map(OrderMap::from_iter([KeyValue::new(MAP_KEYS[0], "value")]))
                .start(&tracer);
            span.end();
        })
    });
    group.finish();
}

fn not_sampled_provider() -> (sdktrace::TracerProvider, sdktrace::Tracer) {
    let provider = sdktrace::TracerProvider::builder()
        .with_config(sdktrace::config().with_sampler(sdktrace::Sampler::AlwaysOff))
        .with_simple_exporter(NoopExporter)
        .build();
    let tracer = provider.tracer("not-sampled");
    (provider, tracer)
}

#[derive(Debug)]
struct NoopExporter;

impl SpanExporter for NoopExporter {
    fn export(&mut self, _spans: Vec<SpanData>) -> BoxFuture<'static, ExportResult> {
        Box::pin(futures_util::future::ready(Ok(())))
    }
}

const MAP_KEYS: [&str; 64] = [
    "key.1", "key.2", "key.3", "key.4", "key.5", "key.6", "key.7", "key.8", "key.9", "key.10",
    "key.11", "key.12", "key.13", "key.14", "key.15", "key.16", "key.17", "key.18", "key.19",
    "key.20", "key.21", "key.22", "key.23", "key.24", "key.25", "key.26", "key.27", "key.28",
    "key.29", "key.30", "key.31", "key.32", "key.33", "key.34", "key.35", "key.36", "key.37",
    "key.38", "key.39", "key.40", "key.41", "key.42", "key.43", "key.44", "key.45", "key.46",
    "key.47", "key.48", "key.49", "key.50", "key.51", "key.52", "key.53", "key.54", "key.55",
    "key.56", "key.57", "key.58", "key.59", "key.60", "key.61", "key.62", "key.63", "key.64",
];

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = criterion_benchmark
}
criterion_main!(benches);
