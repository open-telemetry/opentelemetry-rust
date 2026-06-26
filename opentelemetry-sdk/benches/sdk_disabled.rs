/*
    The benchmark results:
    criterion = "0.5.1"
    rustc 1.98.0-nightly (8b6558a02 2026-06-20)
    OS: macOS 26.5.1 (arm64)
    Hardware: Apple M1

    | Test    | api-only | sdk-disabled | sdk-shutdown | sdk-enabled |
    |---------|----------|--------------|--------------|-------------|
    | trace   |  20.4 ns |  38.9 ns     |  38.5 ns     |  178 ns     |
    | log     |  0.32 ns |  1.28 ns     |  1.27 ns     |  37 ns      |
    | metrics |  6.11 ns |  6.11 ns     |  6.10 ns     |  66 ns      |
*/

use criterion::{criterion_group, criterion_main, Criterion};
use opentelemetry::InstrumentationScope;
use opentelemetry::{
    logs::{LogRecord, Logger, LoggerProvider, Severity},
    metrics::{Counter, MeterProvider},
    trace::{Span, Tracer, TracerProvider},
    KeyValue,
};
use opentelemetry_sdk::{
    error::OTelSdkResult,
    logs::{LogProcessor, SdkLogRecord, SdkLoggerProvider},
    metrics::{
        data::ResourceMetrics, exporter::PushMetricExporter, PeriodicReader, SdkMeterProvider,
        Temporality,
    },
    trace::{Sampler, SdkTracerProvider, SpanData, SpanExporter},
};
use std::time::Duration;

// Run the given closure with the `OTEL_SDK_DISABLED` environment variable set to "true".
fn with_sdk_disabled<T>(f: impl FnOnce() -> T) -> T {
    temp_env::with_var("OTEL_SDK_DISABLED", Some("true"), f)
}

#[derive(Debug)]
struct NoOpSpanExporter;

impl SpanExporter for NoOpSpanExporter {
    async fn export(&self, _spans: Vec<SpanData>) -> OTelSdkResult {
        Ok(())
    }
}

#[derive(Debug)]
struct NoOpLogProcessor;

impl LogProcessor for NoOpLogProcessor {
    fn emit(&self, _data: &mut SdkLogRecord, _scope: &InstrumentationScope) {}

    fn force_flush(&self) -> OTelSdkResult {
        Ok(())
    }

    fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
        Ok(())
    }
}

#[derive(Debug)]
struct NoOpMetricExporter;

impl PushMetricExporter for NoOpMetricExporter {
    async fn export(&self, _metrics: &ResourceMetrics) -> OTelSdkResult {
        Ok(())
    }

    fn force_flush(&self) -> OTelSdkResult {
        Ok(())
    }

    fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
        Ok(())
    }

    fn temporality(&self) -> Temporality {
        Temporality::Cumulative
    }
}

fn emit_span<T: Tracer>(tracer: &T) {
    let mut span = tracer.start("span-name");
    if span.is_recording() {
        span.set_attribute(KeyValue::new("key1", "value1"));
        span.set_attribute(KeyValue::new("key2", 123));
    }
    span.end();
}

fn emit_log<L: Logger>(logger: &L) {
    if logger.event_enabled(Severity::Info, "benchmark", None) {
        let mut record = logger.create_log_record();
        record.set_body("a log message".into());
        record.set_severity_number(Severity::Info);
        record.add_attribute("key1", "value1");
        record.add_attribute("key2", 123);
        logger.emit(record);
    }
}

fn add_to_counter(counter: &Counter<u64>) {
    counter.add(
        1,
        &[KeyValue::new("key1", "value1"), KeyValue::new("key2", 123)],
    );
}

fn trace_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("trace");

    group.bench_function("api-only", |b| {
        let provider = opentelemetry::trace::noop::NoopTracerProvider::new();
        let tracer = provider.tracer("bench");
        b.iter(|| emit_span(&tracer));
    });

    group.bench_function("sdk-disabled", |b| {
        let provider = with_sdk_disabled(|| {
            SdkTracerProvider::builder()
                .with_sampler(Sampler::AlwaysOn)
                .build()
        });
        let tracer = provider.tracer("bench");
        b.iter(|| emit_span(&tracer));
    });

    group.bench_function("sdk-shutdown", |b| {
        let provider = SdkTracerProvider::builder()
            .with_sampler(Sampler::AlwaysOn)
            .with_simple_exporter(NoOpSpanExporter)
            .build();
        let _ = provider.shutdown();
        let tracer = provider.tracer("bench");
        b.iter(|| emit_span(&tracer));
    });

    group.bench_function("sdk-enabled", |b| {
        let provider = SdkTracerProvider::builder()
            .with_sampler(Sampler::AlwaysOn)
            .with_simple_exporter(NoOpSpanExporter)
            .build();
        let tracer = provider.tracer("bench");
        b.iter(|| emit_span(&tracer));
    });

    group.finish();
}

fn log_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("log");

    group.bench_function("api-only", |b| {
        let provider = opentelemetry::logs::NoopLoggerProvider::new();
        let logger = provider.logger("bench");
        b.iter(|| emit_log(&logger));
    });

    group.bench_function("sdk-disabled", |b| {
        let provider = with_sdk_disabled(|| {
            SdkLoggerProvider::builder()
                .with_log_processor(NoOpLogProcessor)
                .build()
        });
        let logger = provider.logger("bench");
        b.iter(|| emit_log(&logger));
    });

    group.bench_function("sdk-shutdown", |b| {
        let provider = SdkLoggerProvider::builder()
            .with_log_processor(NoOpLogProcessor)
            .build();
        let _ = provider.shutdown();
        let logger = provider.logger("bench");
        b.iter(|| emit_log(&logger));
    });

    group.bench_function("sdk-enabled", |b| {
        let provider = SdkLoggerProvider::builder()
            .with_log_processor(NoOpLogProcessor)
            .build();
        let logger = provider.logger("bench");
        b.iter(|| emit_log(&logger));
    });

    group.finish();
}

fn metrics_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("metrics");

    group.bench_function("api-only", |b| {
        let provider = opentelemetry::metrics::NoopMeterProvider::new();
        let counter = provider.meter("bench").u64_counter("counter").build();
        b.iter(|| add_to_counter(&counter));
    });

    group.bench_function("sdk-disabled", |b| {
        let provider = with_sdk_disabled(|| SdkMeterProvider::builder().build());
        let counter = provider.meter("bench").u64_counter("counter").build();
        b.iter(|| add_to_counter(&counter));
    });

    group.bench_function("sdk-shutdown", |b| {
        let provider = SdkMeterProvider::builder()
            .with_reader(PeriodicReader::builder(NoOpMetricExporter).build())
            .build();
        let _ = provider.shutdown();
        let counter = provider.meter("bench").u64_counter("counter").build();
        b.iter(|| add_to_counter(&counter));
    });

    group.bench_function("sdk-enabled", |b| {
        let provider = SdkMeterProvider::builder()
            .with_reader(PeriodicReader::builder(NoOpMetricExporter).build())
            .build();
        let counter = provider.meter("bench").u64_counter("counter").build();
        b.iter(|| add_to_counter(&counter));
    });

    group.finish();
}

fn criterion_benchmark(c: &mut Criterion) {
    trace_benchmark(c);
    log_benchmark(c);
    metrics_benchmark(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(std::time::Duration::from_secs(1))
        .measurement_time(std::time::Duration::from_secs(2));
    targets = criterion_benchmark
}
criterion_main!(benches);
