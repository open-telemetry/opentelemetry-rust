use async_trait::async_trait;
use criterion::{criterion_group, criterion_main, Criterion};
use opentelemetry::logs::LogResult;
use opentelemetry::KeyValue;
use opentelemetry_appender_tracing::layer as tracing_layer;
use opentelemetry_sdk::export::logs::{LogData, LogExporter};
use opentelemetry_sdk::logs::{Config, LogProcessor, LoggerProvider};
use opentelemetry_sdk::Resource;
use tracing::info;
use tracing_subscriber::prelude::*;
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

    fn shutdown(&mut self) -> LogResult<()> {
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

fn benchmark_no_subscriber(c: &mut Criterion) {
    c.bench_function("log_no_subscriber", |b| {
        b.iter(|| {
            info!("my-event-name");
        });
    });
}

fn benchmark_with_subscriber(c: &mut Criterion, enabled: bool, bench_name: &str) {
    let exporter = NoopExporter { enabled };
    let processor = NoopProcessor::new(Box::new(exporter));
    let provider = LoggerProvider::builder()
        .with_config(
            Config::default().with_resource(Resource::new(vec![KeyValue::new(
                "service.name",
                "log-appender-tracing-example",
            )])),
        )
        .with_log_processor(processor)
        .build();
    let tracing_layer = tracing_layer::OpenTelemetryTracingBridge::new(&provider);
    let subscriber = Registry::default().with(tracing_layer);

    tracing::subscriber::with_default(subscriber, || {
        c.bench_function(bench_name, |b| {
            b.iter(|| {
                info!("my-event-name");
            });
        });
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    benchmark_no_subscriber(c);
    benchmark_with_subscriber(c, false, "log_subscriber_disabled");
    benchmark_with_subscriber(c, true, "log_subscriber_enabled");
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
