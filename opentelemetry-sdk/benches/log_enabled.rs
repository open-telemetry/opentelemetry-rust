/*
    The benchmark results:
    criterion = "0.5.1"
    Hardware: Apple M4 Pro
    Total Number of Cores:	14 (10 performance and 4 efficiency)
    | Test                        | Average time|
    |-----------------------------|-------------|
    | exporter_enabled_false      |  1.8 ns     |
*/

use criterion::{criterion_group, criterion_main, Criterion};
use opentelemetry::logs::{LoggerProvider,Logger};
use opentelemetry::InstrumentationScope;
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::logs::{LogBatch, LogExporter, LogProcessor, SdkLogRecord, SdkLoggerProvider};
use opentelemetry_sdk::Resource;
#[cfg(not(target_os = "windows"))]
use pprof::criterion::{Output, PProfProfiler};

#[derive(Debug)]
struct NoopExporter;
impl LogExporter for NoopExporter {
    async fn export(&self, _: LogBatch<'_>) -> OTelSdkResult {
        Ok(())
    }

    fn shutdown(&mut self) -> OTelSdkResult {
        Ok(())
    }

    fn event_enabled(&self, _level: opentelemetry::logs::Severity, _target: &str, _name: Option<&str>) -> bool {
        false
    }

    fn set_resource(&mut self, _: &Resource) {}
}



fn benchmark_exporter_enabled_false(c: &mut Criterion) {
    let processor = ConcurrentExportProcessor::new(NoopExporter);
    let provider = SdkLoggerProvider::builder()
        .with_log_processor(processor)
        .build();
    let logger = provider.logger("test_logger");

    c.bench_function("exporter_enabled_false", |b| {
        b.iter(|| {
            logger.event_enabled(opentelemetry::logs::Severity::Debug, "target", Some("name"));
        });
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    benchmark_exporter_enabled_false(c);
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
