/*
    The benchmark results:
    criterion = "0.5.1"
    rustc 1.98.0 (88d9e12ae 2026-08-18)
    OS: macOS 27.0 (arm64)
    Hardware: Apple M1

    | Test       | api-only | sdk-disabled | sdk-shutdown | sdk-enabled |
    |------------|----------|--------------|--------------|-------------|
    | trace      | 20.6 ns  |  32.4 ns     |  33.5 ns     |  179 ns     |
    | log        | 0.32 ns  |   0.32 ns    |   0.32 ns    |   42.3 ns   |
    | log (emit) | 1.63 ns  |  32.8 ns     |  32.1 ns     |   45.1 ns   |
    | metrics    | 7.07 ns  |   7.12 ns    |   7.09 ns    |   58.3 ns   |

    `log` filters on `event_enabled` first, `log (emit)` emits unconditionally.
    The gap in the latter is the caller building an `SdkLogRecord`, not `emit`,
    which returns immediately when the SDK is disabled.
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
// black_box is required, else the no-op paths get elided entirely and measure 0 ns
use std::hint::black_box;
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
        emit_log_direct(logger);
    }
}

// emits without filtering on `event_enabled` first
fn emit_log_direct<L: Logger>(logger: &L) {
    let mut record = logger.create_log_record();
    record.set_body("a log message".into());
    record.set_severity_number(Severity::Info);
    record.add_attribute("key1", "value1");
    record.add_attribute("key2", 123);
    logger.emit(record);
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
        b.iter(|| emit_span(black_box(&tracer)));
    });

    group.bench_function("sdk-disabled", |b| {
        let provider = with_sdk_disabled(|| {
            SdkTracerProvider::builder()
                .with_sampler(Sampler::AlwaysOn)
                .build()
        });
        let tracer = provider.tracer("bench");
        b.iter(|| emit_span(black_box(&tracer)));
    });

    group.bench_function("sdk-shutdown", |b| {
        let provider = SdkTracerProvider::builder()
            .with_sampler(Sampler::AlwaysOn)
            .with_simple_exporter(NoOpSpanExporter)
            .build();
        let _ = provider.shutdown();
        let tracer = provider.tracer("bench");
        b.iter(|| emit_span(black_box(&tracer)));
    });

    group.bench_function("sdk-enabled", |b| {
        let provider = SdkTracerProvider::builder()
            .with_sampler(Sampler::AlwaysOn)
            .with_simple_exporter(NoOpSpanExporter)
            .build();
        let tracer = provider.tracer("bench");
        b.iter(|| emit_span(black_box(&tracer)));
    });

    group.finish();
}

fn log_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("log");

    group.bench_function("api-only", |b| {
        let provider = opentelemetry::logs::NoopLoggerProvider::new();
        let logger = provider.logger("bench");
        b.iter(|| emit_log(black_box(&logger)));
    });

    group.bench_function("sdk-disabled", |b| {
        let provider = with_sdk_disabled(|| {
            SdkLoggerProvider::builder()
                .with_log_processor(NoOpLogProcessor)
                .build()
        });
        let logger = provider.logger("bench");
        b.iter(|| emit_log(black_box(&logger)));
    });

    group.bench_function("sdk-shutdown", |b| {
        let provider = SdkLoggerProvider::builder()
            .with_log_processor(NoOpLogProcessor)
            .build();
        let _ = provider.shutdown();
        let logger = provider.logger("bench");
        b.iter(|| emit_log(black_box(&logger)));
    });

    group.bench_function("sdk-enabled", |b| {
        let provider = SdkLoggerProvider::builder()
            .with_log_processor(NoOpLogProcessor)
            .build();
        let logger = provider.logger("bench");
        b.iter(|| emit_log(black_box(&logger)));
    });

    group.finish();
}

fn log_emit_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("log-emit");

    group.bench_function("api-only", |b| {
        let provider = opentelemetry::logs::NoopLoggerProvider::new();
        let logger = provider.logger("bench");
        b.iter(|| emit_log_direct(black_box(&logger)));
    });

    group.bench_function("sdk-disabled", |b| {
        let provider = with_sdk_disabled(|| {
            SdkLoggerProvider::builder()
                .with_log_processor(NoOpLogProcessor)
                .build()
        });
        let logger = provider.logger("bench");
        b.iter(|| emit_log_direct(black_box(&logger)));
    });

    group.bench_function("sdk-shutdown", |b| {
        let provider = SdkLoggerProvider::builder()
            .with_log_processor(NoOpLogProcessor)
            .build();
        let _ = provider.shutdown();
        let logger = provider.logger("bench");
        b.iter(|| emit_log_direct(black_box(&logger)));
    });

    group.bench_function("sdk-enabled", |b| {
        let provider = SdkLoggerProvider::builder()
            .with_log_processor(NoOpLogProcessor)
            .build();
        let logger = provider.logger("bench");
        b.iter(|| emit_log_direct(black_box(&logger)));
    });

    group.finish();
}

fn metrics_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("metrics");

    group.bench_function("api-only", |b| {
        let provider = opentelemetry::metrics::NoopMeterProvider::new();
        let counter = provider.meter("bench").u64_counter("counter").build();
        b.iter(|| add_to_counter(black_box(&counter)));
    });

    group.bench_function("sdk-disabled", |b| {
        let provider = with_sdk_disabled(|| SdkMeterProvider::builder().build());
        let counter = provider.meter("bench").u64_counter("counter").build();
        b.iter(|| add_to_counter(black_box(&counter)));
    });

    group.bench_function("sdk-shutdown", |b| {
        let provider = SdkMeterProvider::builder()
            .with_reader(PeriodicReader::builder(NoOpMetricExporter).build())
            .build();
        let _ = provider.shutdown();
        let counter = provider.meter("bench").u64_counter("counter").build();
        b.iter(|| add_to_counter(black_box(&counter)));
    });

    group.bench_function("sdk-enabled", |b| {
        let provider = SdkMeterProvider::builder()
            .with_reader(PeriodicReader::builder(NoOpMetricExporter).build())
            .build();
        let counter = provider.meter("bench").u64_counter("counter").build();
        b.iter(|| add_to_counter(black_box(&counter)));
    });

    group.finish();
}

fn criterion_benchmark(c: &mut Criterion) {
    trace_benchmark(c);
    log_benchmark(c);
    log_emit_benchmark(c);
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
