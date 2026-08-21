use criterion::{criterion_group, criterion_main, Criterion};
use opentelemetry::logs::{LogRecord, Logger, LoggerProvider, Severity};
use opentelemetry::InstrumentationScope;
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::logs::{
    FlightRecorderLogProcessor, LogProcessor, ScopedFlightRecorderLogProcessor, SdkLogRecord,
    SdkLogger, SdkLoggerProvider,
};
use std::time::Duration;

#[derive(Debug)]
struct NoopProcessor;

impl LogProcessor for NoopProcessor {
    fn emit(&self, _record: &mut SdkLogRecord, _scope: &InstrumentationScope) {}

    fn force_flush(&self) -> OTelSdkResult {
        Ok(())
    }

    fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
        Ok(())
    }
}

fn record(logger: &SdkLogger, severity: Severity) -> SdkLogRecord {
    let mut record = logger.create_log_record();
    record.set_severity_number(severity);
    record.set_body("flight recorder benchmark".into());
    record.add_attribute("request.id", "benchmark");
    record
}

fn benchmark(c: &mut Criterion) {
    let instrumentation = InstrumentationScope::builder("benchmark").build();
    let provider = SdkLoggerProvider::builder().build();
    let logger = provider.logger("benchmark");
    let (global, _trigger) = FlightRecorderLogProcessor::builder(NoopProcessor).build();
    c.bench_function("flight_recorder/global_buffered_emit", |b| {
        b.iter(|| global.emit(&mut record(&logger, Severity::Info), &instrumentation));
    });

    let (global, _trigger) = FlightRecorderLogProcessor::builder(NoopProcessor).build();
    c.bench_function("flight_recorder/global_bypass_emit", |b| {
        b.iter(|| global.emit(&mut record(&logger, Severity::Warn), &instrumentation));
    });

    let (scoped, recorder) = ScopedFlightRecorderLogProcessor::builder(NoopProcessor).build();
    let scope = recorder.try_start().unwrap();
    c.bench_function("flight_recorder/scoped_context_and_buffered_emit", |b| {
        b.iter(|| {
            futures_executor::block_on(scope.with_context(async {
                scoped.emit(&mut record(&logger, Severity::Info), &instrumentation);
            }));
        });
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
