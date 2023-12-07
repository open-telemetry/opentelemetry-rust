use async_trait::async_trait;
use criterion::{criterion_group, criterion_main, Criterion};

use opentelemetry::logs::{LogResult, Logger, LoggerProvider as _};
use opentelemetry::trace::Tracer;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::export::logs::{LogData, LogExporter};
use opentelemetry_sdk::logs::LoggerProvider;
use opentelemetry_sdk::trace::{config, Sampler, TracerProvider};

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

        let logger = provider.logger("no-context");

        b.iter(|| f(&logger));
    });

    group.bench_function("with-context", |b| {
        let provider = LoggerProvider::builder()
            .with_simple_exporter(VoidExporter)
            .build();

        let logger = provider.logger("with-context");

        // setup tracing as well.
        let tracer_provider = TracerProvider::builder()
            .with_config(config().with_sampler(Sampler::AlwaysOn))
            .build();
        let tracer = tracer_provider.tracer("bench-tracer");

        // Act
        tracer.in_span("bench-span", |_cx| {
            b.iter(|| f(&logger));
        });
    });

    group.finish();
}

fn criterion_benchmark(c: &mut Criterion) {}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
