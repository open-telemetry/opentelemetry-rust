use async_trait::async_trait;
use criterion::{criterion_group, criterion_main, Criterion};
use opentelemetry::logs::LogResult;
use opentelemetry::KeyValue;
use opentelemetry_appender_tracing::layer;
use opentelemetry_sdk::export::logs::LogData;
use opentelemetry_sdk::{
    logs::{Config, LoggerProvider},
    Resource,
};
use tracing::info;
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;

#[derive(Debug, Clone)]
struct VoidExporter {
    is_enabled: bool,
}

#[async_trait]
impl opentelemetry_sdk::export::logs::LogExporter for VoidExporter {
    async fn export(&mut self, _batch: Vec<LogData>) -> LogResult<()> {
        LogResult::Ok(())
    }

    fn event_enabled(
        &self,
        _level: opentelemetry::logs::Severity,
        _target: &str,
        _name: &str,
    ) -> bool {
        self.is_enabled
    }
}

// disabled log processor
#[derive(Debug)]
struct VoidProcessor {
    exporter: Box<dyn opentelemetry_sdk::export::logs::LogExporter>,
}

impl VoidProcessor {
    fn new(exporter: Box<dyn opentelemetry_sdk::export::logs::LogExporter>) -> Self {
        VoidProcessor { exporter: exporter }
    }
}

impl opentelemetry_sdk::logs::LogProcessor for VoidProcessor {
    fn emit(&self, _data: LogData) {
        //  nop
    }
    fn force_flush(&self) -> LogResult<()> {
        Ok(())
    }
    fn shutdown(&mut self) -> LogResult<()> {
        Ok(())
    }
    fn event_enabled(
        &self,
        _level: opentelemetry::logs::Severity,
        _target: &str,
        _name: &str,
    ) -> bool {
        self.exporter.event_enabled(_level, _target, _name)
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let exporter1 = VoidExporter { is_enabled: false };
    let processor1 = VoidProcessor::new(Box::new(exporter1));
    let provider1: LoggerProvider = LoggerProvider::builder()
        .with_config(
            Config::default().with_resource(Resource::new(vec![KeyValue::new(
                "service.name",
                "log-appender-tracing-example",
            )])),
        )
        .with_log_processor(processor1)
        .build();

    c.bench_function("log_no_subscriber", |b| {
        b.iter(|| {
            info!("my-event-name");
        });
    });
    let layer1 = layer::OpenTelemetryTracingBridge::new(&provider1);
    let subscriber1 = Registry::default().with(layer1);
    tracing::subscriber::with_default(subscriber1, || {
        c.bench_function("log_subscriber_log_level_disabled", |b| {
            b.iter(|| {
                info!("my-event-name");
            });
        });
    });
    drop(provider1);

    let exporter2 = VoidExporter { is_enabled: true };
    let processor2 = VoidProcessor::new(Box::new(exporter2));
    let provider2 = LoggerProvider::builder()
        .with_config(
            Config::default().with_resource(Resource::new(vec![KeyValue::new(
                "service.name",
                "log-appender-tracing-example",
            )])),
        )
        .with_log_processor(processor2)
        .build();
    let layer2 = layer::OpenTelemetryTracingBridge::new(&provider2);
    let subscriber2 = Registry::default().with(layer2);
    tracing::subscriber::with_default(subscriber2, || {
        c.bench_function("log_subscriber_log_level_enabled", |b| {
            b.iter(|| {
                info!("my-event-name");
            });
        });
    });
    drop(provider2);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
