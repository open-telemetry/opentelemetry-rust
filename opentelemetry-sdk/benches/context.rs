use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use futures_util::future::BoxFuture;
use opentelemetry::{
    global::BoxedTracer,
    trace::{
        noop::NoopTracer, SpanContext, SpanId, TraceContextExt, TraceFlags, TraceId, TraceState,
        TracerProvider as _,
    },
    Context, ContextGuard,
};
use opentelemetry_sdk::{
    export::trace::{ExportResult, SpanData, SpanExporter},
    trace::{config, Sampler, TracerProvider},
};
#[cfg(not(target_os = "windows"))]
use pprof::criterion::{Output, PProfProfiler};
use std::fmt::Display;

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("context");
    group.throughput(Throughput::Elements(1));
    for env in [
        Environment::InContext,
        Environment::NoContext,
        Environment::NoSdk,
    ] {
        let (_provider, _tracer, _guard) = env.setup();

        for api in [Api::Alt, Api::Spec] {
            let param = format!("{env}/{api}");
            group.bench_function(
                BenchmarkId::new("has_active_span", param.clone()),
                |b| match api {
                    Api::Alt => b.iter(|| Context::map_current(TraceContextExt::has_active_span)),
                    Api::Spec => b.iter(|| Context::current().has_active_span()),
                },
            );
            group.bench_function(
                BenchmarkId::new("is_sampled", param.clone()),
                |b| match api {
                    Api::Alt => {
                        b.iter(|| Context::map_current(|cx| cx.span().span_context().is_sampled()))
                    }
                    Api::Spec => b.iter(|| Context::current().span().span_context().is_sampled()),
                },
            );
            group.bench_function(BenchmarkId::new("is_recording", param), |b| match api {
                Api::Alt => b.iter(|| Context::map_current(|cx| cx.span().is_recording())),
                Api::Spec => b.iter(|| Context::current().span().is_recording()),
            });
        }
    }
}

#[derive(Copy, Clone)]
enum Api {
    /// An alternative way which may be faster than what the spec recommends.
    Alt,
    /// The recommended way as proposed by the current opentelemetry specification.
    Spec,
}

impl Api {
    const fn as_str(self) -> &'static str {
        match self {
            Api::Alt => "alt",
            Api::Spec => "spec",
        }
    }
}

impl Display for Api {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Copy, Clone)]
enum Environment {
    /// There is an active span being sampled in the current context.
    InContext,
    /// There is no span in context (or there is not context).
    NoContext,
    /// An SDK has not been configured, so instrumentation should be noop.
    NoSdk,
}

impl Environment {
    const fn as_str(self) -> &'static str {
        match self {
            Environment::InContext => "in-cx",
            Environment::NoContext => "no-cx",
            Environment::NoSdk => "no-sdk",
        }
    }

    fn setup(&self) -> (Option<TracerProvider>, BoxedTracer, Option<ContextGuard>) {
        match self {
            Environment::InContext => {
                let guard = Context::current()
                    .with_remote_span_context(SpanContext::new(
                        TraceId::from(0x09251969),
                        SpanId::from(0x08171969),
                        TraceFlags::SAMPLED,
                        true,
                        TraceState::default(),
                    ))
                    .attach();
                let (provider, tracer) = parent_sampled_tracer(Sampler::AlwaysOff);
                (Some(provider), tracer, Some(guard))
            }
            Environment::NoContext => {
                let (provider, tracer) = parent_sampled_tracer(Sampler::AlwaysOff);
                (Some(provider), tracer, None)
            }
            Environment::NoSdk => (None, BoxedTracer::new(Box::new(NoopTracer::new())), None),
        }
    }
}

impl Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

fn parent_sampled_tracer(inner_sampler: Sampler) -> (TracerProvider, BoxedTracer) {
    let provider = TracerProvider::builder()
        .with_config(config().with_sampler(Sampler::ParentBased(Box::new(inner_sampler))))
        .with_simple_exporter(NoopExporter)
        .build();
    let tracer = provider.tracer(module_path!());
    (provider, BoxedTracer::new(Box::new(tracer)))
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
