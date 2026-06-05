//! Benchmarks for `SdkLogRecord::add_attribute` deduplication overhead.
//!
//! Run with:
//! `cargo bench --bench log_record_attribute_dedup --features logs`
//!
//! Apple M4 Max, macOS 25.3.0 (2026-06-05)
//!
//! Results (unique keys — typical case, no duplicates in the batch):
//! | Scenario                | Dedup ON | Dedup OFF (before) | Overhead |
//! |-------------------------|----------|--------------------|----------|
//! | add_1_unique_attribute  | 25.6 ns  | 25.9 ns            |  ~1.0x   |
//! | add_5_unique_attributes | 129.3 ns | 127.5 ns           |  ~1.0x   |
//! | add_9_unique_attributes | 292.0 ns | 223.6 ns           |  ~1.3x   |
//!
//! Results (repeated key — duplicate writes to the same key):
//! | Scenario       | Dedup ON | Dedup OFF (before) |
//! |----------------|----------|--------------------|
//! | add_5_same_key | 32.1 ns  | 38.5 ns            |
//!
//! Note: criterion does not fail CI on regression by itself. These numbers are
//! reference values for human review in PR #3537 / issue #3497.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use opentelemetry::logs::{AnyValue, LogRecord as _, Logger, LoggerProvider};
use opentelemetry::Key;
use opentelemetry_sdk::logs::SdkLoggerProvider;

struct BenchLoggers {
    dedup_on: <SdkLoggerProvider as LoggerProvider>::Logger,
    dedup_off: <SdkLoggerProvider as LoggerProvider>::Logger,
}

impl BenchLoggers {
    fn new() -> Self {
        let dedup_on = SdkLoggerProvider::builder().build().logger("bench");
        let dedup_off = SdkLoggerProvider::builder()
            .with_log_record_attribute_deduplication(false)
            .build()
            .logger("bench");
        Self {
            dedup_on,
            dedup_off,
        }
    }
}

fn bench_add_unique_attributes(c: &mut Criterion) {
    let loggers = BenchLoggers::new();
    let mut group = c.benchmark_group("LogRecord_AddUniqueAttributes");

    for count in [1usize, 5, 9] {
        let keys: Vec<Key> = (0..count).map(|i| Key::new(format!("key{i}"))).collect();

        group.bench_with_input(BenchmarkId::new("dedup_on", count), &keys, |b, keys| {
            b.iter(|| {
                let mut record = loggers.dedup_on.create_log_record();
                for (i, key) in keys.iter().enumerate() {
                    record.add_attribute(key.clone(), AnyValue::Int(i as i64));
                }
                black_box(record.attributes_iter().count());
            });
        });

        group.bench_with_input(BenchmarkId::new("dedup_off", count), &keys, |b, keys| {
            b.iter(|| {
                let mut record = loggers.dedup_off.create_log_record();
                for (i, key) in keys.iter().enumerate() {
                    record.add_attribute(key.clone(), AnyValue::Int(i as i64));
                }
                black_box(record.attributes_iter().count());
            });
        });
    }

    group.finish();
}

fn bench_add_repeated_key(c: &mut Criterion) {
    let loggers = BenchLoggers::new();
    let mut group = c.benchmark_group("LogRecord_AddRepeatedKey");
    let key = Key::new("key");

    group.bench_function("dedup_on_5_writes", |b| {
        b.iter(|| {
            let mut record = loggers.dedup_on.create_log_record();
            for i in 0..5 {
                record.add_attribute(key.clone(), AnyValue::Int(i));
            }
            black_box(record.attributes_iter().count());
        });
    });

    group.bench_function("dedup_off_5_writes", |b| {
        b.iter(|| {
            let mut record = loggers.dedup_off.create_log_record();
            for i in 0..5 {
                record.add_attribute(key.clone(), AnyValue::Int(i));
            }
            black_box(record.attributes_iter().count());
        });
    });

    group.finish();
}

fn criterion_benchmark(c: &mut Criterion) {
    bench_add_unique_attributes(c);
    bench_add_repeated_key(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(std::time::Duration::from_secs(1))
        .measurement_time(std::time::Duration::from_secs(2));
    targets = criterion_benchmark
}
criterion_main!(benches);
