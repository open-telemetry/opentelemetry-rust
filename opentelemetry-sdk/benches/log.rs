use std::time::SystemTime;

use async_trait::async_trait;
use criterion::{criterion_group, criterion_main, Criterion};

use opentelemetry::logs::{LogRecord, LogResult, Logger, LoggerProvider as _, Severity};
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

fn criterion_benchmark(c: &mut Criterion) {
    log_benchmark_group(c, "simple-log", |logger| {
        logger.emit(LogRecord::builder().with_body("simple log".into()).build())
    });

    log_benchmark_group(c, "long-log", |logger| {
        logger.emit(LogRecord::builder().with_body("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Gravida in fermentum et sollicitudin ac orci phasellus. Ullamcorper dignissim cras tincidunt lobortis feugiat vivamus at augue. Magna etiam tempor orci eu. Sed tempus urna et pharetra pharetra massa.".into()).build())
    });

    let now = SystemTime::now();
    log_benchmark_group(c, "full-log", |logger| {
        logger.emit(
            LogRecord::builder()
                .with_body("full log".into())
                .with_timestamp(now)
                .with_observed_timestamp(now)
                .with_severity_number(Severity::Warn)
                .with_severity_text(Severity::Warn.name())
                .build(),
        )
    });

    log_benchmark_group(c, "full-log-with-4-attributes", |logger| {
        logger.emit(
            LogRecord::builder()
                .with_body("full log".into())
                .with_timestamp(now)
                .with_observed_timestamp(now)
                .with_severity_number(Severity::Warn)
                .with_severity_text(Severity::Warn.name())
                .with_attribute("name", "my-event-name")
                .with_attribute("event.id", 20)
                .with_attribute("user.name", "otel")
                .with_attribute("user.email", "otel@opentelemetry.io")
                .build(),
        )
    });

    log_benchmark_group(c, "full-log-with-9-attributes", |logger| {
        logger.emit(
            LogRecord::builder()
                .with_body("full log".into())
                .with_timestamp(now)
                .with_observed_timestamp(now)
                .with_severity_number(Severity::Warn)
                .with_severity_text(Severity::Warn.name())
                .with_attribute("name", "my-event-name")
                .with_attribute("event.id", 20)
                .with_attribute("user.name", "otel")
                .with_attribute("user.email", "otel@opentelemetry.io")
                .with_attribute("log.source.file.name", "log.rs")
                .with_attribute("log.source.file.path", "opentelemetry_sdk/benches/log.rs")
                .with_attribute("log.source.file.line", 96)
                .with_attribute("log.module.path", "opentelemetry_sdk::benches::log")
                .with_attribute("log.target", "opentelemetry_sdk::benches::log")
                .build(),
        )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
