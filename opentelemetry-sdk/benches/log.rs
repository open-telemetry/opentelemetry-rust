use async_trait::async_trait;
use criterion::{criterion_group, criterion_main, Criterion};

use opentelemetry::logs::{LogResult, Logger};
use opentelemetry_sdk::export::logs::{LogData, LogExporter};
use opentelemetry_sdk::logs::LoggerProvider;

#[derive(Debug)]
struct VoidExporter;

#[async_trait]
impl LogExporter for VoidExporter {
    async fn export(&mut self, _batch: Vec<LogData>) -> LogResult<()> {
        LogResult::Ok(())
    }
}

fn log_benchmark_group<F: Fn(&dyn Logger)>(c: &mut Criterion, name: &str, f: F) {
    let mut group = c.benchmark_group(name);

    group.bench_function("no-context", |b| {
        let provider = LoggerProvider::builder()
            .with_simple_exporter(VoidExporter)
            .build();
    });

    //group.bench_function("always-sample", |b| {
    //let provider = sdktrace::TracerProvider::builder()
    //.with_config(sdktrace::config().with_sampler(sdktrace::Sampler::AlwaysOn))
    //.with_simple_exporter(VoidExporter)
    //.build();
    //let always_sample = provider.tracer("always-sample");

    //b.iter(|| f(&always_sample));
    //});

    //group.bench_function("never-sample", |b| {
    //let provider = sdktrace::TracerProvider::builder()
    //.with_config(sdktrace::config().with_sampler(sdktrace::Sampler::AlwaysOff))
    //.with_simple_exporter(VoidExporter)
    //.build();
    //let never_sample = provider.tracer("never-sample");
    //b.iter(|| f(&never_sample));
    //});

    group.finish();
}

fn criterion_benchmark(c: &mut Criterion) {}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
