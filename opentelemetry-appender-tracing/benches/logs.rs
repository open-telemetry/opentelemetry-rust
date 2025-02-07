/*
    The benchmark results:
    criterion = "0.5.1"
    OS: Ubuntu 22.04.3 LTS (5.15.146.1-microsoft-standard-WSL2)
    Hardware: AMD EPYC 7763 64-Core Processor - 2.44 GHz, 16vCPUs,
    RAM: 64.0 GB
    | Test                        | Average time|
    |-----------------------------|-------------|
    | log_no_subscriber           | 313 ps      |
    | noop_layer_disabled         | 12 ns       |
    | noop_layer_enabled          | 25 ns       |
    | ot_layer_disabled           | 19 ns       |
    | ot_layer_enabled            | 196 ns      |
*/

use criterion::{criterion_group, criterion_main, Criterion};
use opentelemetry::InstrumentationScope;
use opentelemetry_appender_tracing::layer as tracing_layer;
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::logs::{LogBatch, LogExporter};
use opentelemetry_sdk::logs::{LogProcessor, SdkLogRecord, SdkLoggerProvider};
use opentelemetry_sdk::Resource;
use pprof::criterion::{Output, PProfProfiler};
use tracing::error;
use tracing_subscriber::prelude::*;
use tracing_subscriber::Layer;
use tracing_subscriber::Registry;

#[derive(Debug, Clone)]
struct NoopExporter {
    enabled: bool,
}

impl LogExporter for NoopExporter {
    #[allow(clippy::manual_async_fn)]
    fn export(
        &self,
        _batch: LogBatch<'_>,
    ) -> impl std::future::Future<Output = OTelSdkResult> + Send {
        async { OTelSdkResult::Ok(()) }
    }

    fn event_enabled(&self, _: opentelemetry::logs::Severity, _: &str, _: &str) -> bool {
        self.enabled
    }
}

#[derive(Debug)]
struct NoopProcessor<E: LogExporter> {
    exporter: E,
}

impl<E: LogExporter> NoopProcessor<E> {
    fn new(exporter: E) -> Self {
        Self { exporter }
    }
}

impl<E: LogExporter> LogProcessor for NoopProcessor<E> {
    fn emit(&self, _: &mut SdkLogRecord, _: &InstrumentationScope) {
        // no-op
    }

    fn force_flush(&self) -> OTelSdkResult {
        Ok(())
    }

    fn shutdown(&self) -> OTelSdkResult {
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
                message = "Unable to process checkout."
            );
        });
    });
}

fn benchmark_with_ot_layer(c: &mut Criterion, enabled: bool, bench_name: &str) {
    let exporter = NoopExporter { enabled };
    let processor = NoopProcessor::new(exporter);
    let provider = SdkLoggerProvider::builder()
        .with_resource(
            Resource::builder_empty()
                .with_service_name("benchmark")
                .build(),
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
                    message = "Unable to process checkout."
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
