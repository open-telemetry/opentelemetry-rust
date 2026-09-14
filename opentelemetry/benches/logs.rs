/*
    Benchmark Results:
    criterion = "0.5.1"
    OS: macOS 15
    Hardware: Apple M4 Pro, 14 cores, RAM: 24 GB
    | Test                                             | Average time|
    |--------------------------------------------------|-------------|
    | CreateLogRecord_NoopLogger                       |    0.26 ns  |
    | CreateLogRecord_NoopLogger_WithAttributes        |    0.26 ns  |
    | CreateLogRecord_NoopLogger_WithBody              |    1.30 ns  |
    | CreateLogRecord_NoopLogger_Full                  |    1.30 ns  |
    | EventEnabled_NoopLogger                          |    0.26 ns  |
*/

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use opentelemetry::logs::{
    AnyValue, LogRecord, Logger, LoggerProvider, NoopLoggerProvider, Severity,
};

// Run this benchmark with:
// cargo bench --bench logs --features=logs

fn criterion_benchmark(c: &mut Criterion) {
    let provider = NoopLoggerProvider::new();
    let logger = provider.logger("bench");

    c.bench_function("CreateLogRecord_NoopLogger", |b| {
        b.iter(|| {
            let record = logger.create_log_record();
            logger.emit(record);
        });
    });

    c.bench_function("CreateLogRecord_NoopLogger_WithAttributes", |b| {
        b.iter(|| {
            let mut record = logger.create_log_record();
            record.add_attribute("key1", "value1");
            record.add_attribute("key2", 123);
            record.add_attribute("key3", true);
            record.add_attribute("key4", 123.456);
            logger.emit(record);
        });
    });

    c.bench_function("CreateLogRecord_NoopLogger_WithBody", |b| {
        b.iter(|| {
            let mut record = logger.create_log_record();
            record.set_body(AnyValue::String("This is a log message".into()));
            record.set_severity_number(Severity::Info);
            record.set_severity_text("INFO");
            logger.emit(record);
        });
    });

    c.bench_function("CreateLogRecord_NoopLogger_Full", |b| {
        b.iter(|| {
            let mut record = logger.create_log_record();
            record.set_body(AnyValue::String("This is a log message".into()));
            record.set_severity_number(Severity::Warn);
            record.set_severity_text("WARN");
            record.set_event_name("my.event");
            record.set_target("my_crate::my_module");
            record.add_attribute("key1", "value1");
            record.add_attribute("key2", 123);
            record.add_attribute("key3", true);
            record.add_attribute("key4", 123.456);
            logger.emit(record);
        });
    });

    c.bench_function("EventEnabled_NoopLogger", |b| {
        b.iter(|| {
            black_box(logger.event_enabled(Severity::Info, "my_target", Some("my_event")));
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(std::time::Duration::from_secs(1))
        .measurement_time(std::time::Duration::from_secs(2));
    targets = criterion_benchmark
}
criterion_main!(benches);
