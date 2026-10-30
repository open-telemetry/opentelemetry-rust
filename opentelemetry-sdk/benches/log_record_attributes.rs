//! Measures the cost of reading log record attributes, which is where duplicate
//! keys are resolved (last-write-wins). Adding attributes is covered by the
//! `log` benchmark.
//!
//! run with `$ cargo bench --bench log_record_attributes --features logs`
/*
The benchmark results (median of 3 runs):
criterion = "0.5.1"
OS: macOS 26.6.2
Hardware: Apple M4
| Test                              | Average time |
|-----------------------------------|--------------|
| attributes_iter/1-attributes      |   2.9 ns     |
| attributes_iter/4-attributes      |  15.7 ns     |
| attributes_iter/5-attributes      |  20.9 ns     |
| attributes_iter/9-attributes      |  38.6 ns     |
| attributes_to_owned/1-attributes  |  49.4 ns     |
| attributes_to_owned/4-attributes  | 142.5 ns     |
| attributes_to_owned/5-attributes  | 208.4 ns     |
| attributes_to_owned/9-attributes  | 388.1 ns     |
*/

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use opentelemetry::logs::{LogRecord as _, Logger, LoggerProvider};
use opentelemetry_sdk::logs::{SdkLogRecord, SdkLoggerProvider};

// Same keys as the `full-log-with-9-attributes` case in the `log` benchmark.
const KEYS: [&str; 9] = [
    "name",
    "event.id",
    "user.name",
    "user.email",
    "code.filename",
    "code.filepath",
    "code.lineno",
    "code.namespace",
    "log.target",
];

fn log_record_with_attributes(count: usize) -> SdkLogRecord {
    let provider = SdkLoggerProvider::builder().build();
    let mut log_record = provider.logger("bench").create_log_record();
    for key in &KEYS[..count] {
        log_record.add_attribute(*key, "value");
    }
    log_record
}

fn attributes_iter(c: &mut Criterion) {
    let mut group = c.benchmark_group("attributes_iter");
    for count in [1, 4, 5, 9] {
        let log_record = log_record_with_attributes(count);
        group.bench_function(format!("{count}-attributes"), |b| {
            b.iter(|| {
                for attribute in black_box(&log_record).attributes_iter() {
                    black_box(attribute);
                }
            })
        });
    }
    group.finish();
}

// Roughly what the OTLP exporter does per attribute (it also converts the
// value), to put the cost of `attributes_iter` in context.
fn attributes_to_owned(c: &mut Criterion) {
    let mut group = c.benchmark_group("attributes_to_owned");
    for count in [1, 4, 5, 9] {
        let log_record = log_record_with_attributes(count);
        group.bench_function(format!("{count}-attributes"), |b| {
            b.iter(|| {
                black_box(&log_record)
                    .attributes_iter()
                    .map(|(key, value)| (key.to_string(), value.clone()))
                    .collect::<Vec<_>>()
            })
        });
    }
    group.finish();
}

criterion_group!(benches, attributes_iter, attributes_to_owned);
criterion_main!(benches);
