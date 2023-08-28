use std::fmt::Display;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use futures_util::future::BoxFuture;
use opentelemetry::{
    trace::{TraceContextExt, Tracer, TracerProvider},
    Context,
};
use opentelemetry_sdk::{
    export::trace::{ExportResult, SpanData, SpanExporter},
    trace as sdktrace,
};
#[cfg(not(target_os = "windows"))]
use pprof::criterion::{Output, PProfProfiler};

fn criterion_benchmark(c: &mut Criterion) {
    benchmark_group(c, BenchmarkParameter::NoActiveSpan);
    benchmark_group(c, BenchmarkParameter::WithActiveSpan);
}

fn benchmark_group(c: &mut Criterion, p: BenchmarkParameter) {
    let _guard = match p {
        BenchmarkParameter::NoActiveSpan => None,
        BenchmarkParameter::WithActiveSpan => {
            let (provider, tracer) = tracer();
            let guard = Context::current_with_span(tracer.start("span")).attach();
            Some((guard, provider))
        }
    };

    let mut group = c.benchmark_group("context");

    group.bench_function(BenchmarkId::new("baseline current()", p), |b| {
        b.iter(|| {
            black_box(Context::current());
        })
    });

    group.bench_function(BenchmarkId::new("current().has_active_span()", p), |b| {
        b.iter(|| {
            black_box(Context::current().has_active_span());
        })
    });

    group.bench_function(
        BenchmarkId::new("map_current(|cx| cx.has_active_span())", p),
        |b| {
            b.iter(|| {
                black_box(Context::map_current(|cx| cx.has_active_span()));
            })
        },
    );

    group.finish();
}

#[derive(Copy, Clone)]
enum BenchmarkParameter {
    NoActiveSpan,
    WithActiveSpan,
}

impl Display for BenchmarkParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            BenchmarkParameter::NoActiveSpan => write!(f, "no-active-span"),
            BenchmarkParameter::WithActiveSpan => write!(f, "with-active-span"),
        }
    }
}

fn tracer() -> (sdktrace::TracerProvider, sdktrace::Tracer) {
    let provider = sdktrace::TracerProvider::builder()
        .with_config(sdktrace::config().with_sampler(sdktrace::Sampler::AlwaysOn))
        .with_simple_exporter(NoopExporter)
        .build();
    let tracer = provider.tracer(module_path!());
    (provider, tracer)
}

#[derive(Debug)]
struct NoopExporter;

impl SpanExporter for NoopExporter {
    fn export(&mut self, _spans: Vec<SpanData>) -> BoxFuture<'static, ExportResult> {
        Box::pin(futures_util::future::ready(Ok(())))
    }
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
