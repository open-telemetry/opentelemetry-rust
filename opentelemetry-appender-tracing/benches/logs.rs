/*
    The benchmark results:
    criterion = "0.5.1"
    OS: Ubuntu 22.04.2 LTS (5.10.102.1-microsoft-standard-WSL2)
    Hardware: AMD EPYC 7763 64-Core Processor - 2.44 GHz, 16vCPUs,
    RAM: 64.0 GB
    | Test                        | Average time|
    |-----------------------------|-------------|
    | log_no_subscriber           | 313 ps      |
    | noop_layer_disabled         | 12 ns       |
    | noop_layer_enabled          | 25 ns       |
    | ot_layer_disabled           | 19 ns       |
    | ot_layer_enabled           | 561 ns       |
*/

use async_trait::async_trait;
use criterion::{criterion_group, criterion_main, Criterion};
use opentelemetry::logs::LogResult;
use opentelemetry::KeyValue;
use opentelemetry_appender_tracing::layer as tracing_layer;
use opentelemetry_sdk::export::logs::{LogData, LogExporter};
use opentelemetry_sdk::logs::{Config, LogProcessor, LoggerProvider};
use opentelemetry_sdk::Resource;
use tracing::error;
use tracing_subscriber::prelude::*;
use tracing_subscriber::Layer;
use tracing_subscriber::Registry;

#[derive(Debug, Clone)]
struct NoopExporter {
    enabled: bool,
}

#[async_trait]
impl LogExporter for NoopExporter {
    async fn export(&mut self, _: Vec<LogData>) -> LogResult<()> {
        LogResult::Ok(())
    }

    fn event_enabled(&self, _: opentelemetry::logs::Severity, _: &str, _: &str) -> bool {
        self.enabled
    }
}

#[derive(Debug)]
struct NoopProcessor {
    exporter: Box<dyn LogExporter>,
}

impl NoopProcessor {
    fn new(exporter: Box<dyn LogExporter>) -> Self {
        Self { exporter }
    }
}

impl LogProcessor for NoopProcessor {
    fn emit(&self, _: LogData) {
        // no-op
    }

    fn force_flush(&self) -> LogResult<()> {
        Ok(())
    }

    fn shutdown(&self) -> LogResult<()> {
        Ok(())
    }

    fn event_enabled(
        &self,
        level: opentelemetry::logs::Severity,
        target: &str,
        name: &str,
    ) -> bool {
        self.exporter.event_enabled(level, target, name)
    }
}

struct NoOpLogLayer {
    enabled: bool,
}

impl<S> Layer<S> for NoOpLogLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = NoopEventVisitor;
        event.record(&mut visitor);
    }

    fn event_enabled(
        &self,
        _event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) -> bool {
        self.enabled
    }
}

struct NoopEventVisitor;

impl tracing::field::Visit for NoopEventVisitor {
    fn record_debug(&mut self, _field: &tracing::field::Field, _value: &dyn std::fmt::Debug) {}
}

fn benchmark_no_subscriber(c: &mut Criterion) {
    c.bench_function("log_no_subscriber", |b| {
        b.iter(|| {
            error!(
                name = "CheckoutFailed",
                book_id = "12345",
                book_title = "Rust Programming Adventures",
                "Unable to process checkout."
            );
        });
    });
}

fn benchmark_with_ot_layer(c: &mut Criterion, enabled: bool, bench_name: &str) {
    let exporter = NoopExporter { enabled };
    let processor = NoopProcessor::new(Box::new(exporter));
    let provider = LoggerProvider::builder()
        .with_config(
            Config::default().with_resource(Resource::new(vec![KeyValue::new(
                "service.name",
                "benchmark",
            )])),
        )
        .with_log_processor(processor)
        .build();
    let ot_layer = tracing_layer::OpenTelemetryTracingBridge::new(&provider);
    let subscriber = Registry::default().with(ot_layer);

    tracing::subscriber::with_default(subscriber, || {
        c.bench_function(bench_name, |b| {
            b.iter(|| {
                error!(
                    name = "CheckoutFailed",
                    book_id = "12345",
                    book_title = "Rust Programming Adventures",
                    "Unable to process checkout."
                );
            });
        });
    });
}

fn benchmark_with_noop_layer(c: &mut Criterion, enabled: bool, bench_name: &str) {
    let subscriber = Registry::default().with(NoOpLogLayer { enabled });

    tracing::subscriber::with_default(subscriber, || {
        c.bench_function(bench_name, |b| {
            b.iter(|| {
                error!(
                    name = "CheckoutFailed",
                    book_id = "12345",
                    book_title = "Rust Programming Adventures",
                    "Unable to process checkout."
                );
            });
        });
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    benchmark_no_subscriber(c);
    benchmark_with_ot_layer(c, true, "ot_layer_enabled");
    benchmark_with_ot_layer(c, false, "ot_layer_disabled");
    benchmark_with_noop_layer(c, true, "noop_layer_enabled");
    benchmark_with_noop_layer(c, false, "noop_layer_disabled");
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
