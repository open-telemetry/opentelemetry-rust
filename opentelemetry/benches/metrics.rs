/*
    Benchmark Results:
    criterion = "0.5.1"
    OS: Ubuntu 22.04.4 LTS (5.15.153.1-microsoft-standard-WSL2)
    Hardware: Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz, 16vCPUs,
    RAM: 64.0 GB
    | Test                           | Average time|
    |--------------------------------|-------------|
    | NoAttributes                   | 1.1616 ns   |
    | AddWithInlineStaticAttributes  | 13.296 ns   |
    | AddWithStaticArray             | 1.1612 ns   |
    | AddWithDynamicAttributes       | 110.40 ns   |
*/

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use opentelemetry::{global, metrics::Counter, KeyValue};

// Run this benchmark with:
// cargo bench --bench metrics

fn create_counter() -> Counter<u64> {
    let meter = global::meter("benchmarks");

    meter.u64_counter("counter_bench").init()
}

fn criterion_benchmark(c: &mut Criterion) {
    counter_add(c);
}

fn counter_add(c: &mut Criterion) {
    let counter = create_counter();

    c.bench_function("NoAttributes", |b| {
        b.iter(|| {
            counter.add(1, &[]);
        });
    });

    c.bench_function("AddWithInlineStaticAttributes", |b| {
        b.iter(|| {
            counter.add(
                1,
                &[
                    KeyValue::new("attribute1", "value1"),
                    KeyValue::new("attribute2", "value2"),
                    KeyValue::new("attribute3", "value3"),
                    KeyValue::new("attribute4", "value4"),
                ],
            );
        });
    });

    let kv = [
        KeyValue::new("attribute1", "value1"),
        KeyValue::new("attribute2", "value2"),
        KeyValue::new("attribute3", "value3"),
        KeyValue::new("attribute4", "value4"),
    ];

    c.bench_function("AddWithStaticArray", |b| {
        b.iter(|| {
            counter.add(1, &kv);
        });
    });

    c.bench_function("AddWithDynamicAttributes", |b| {
        b.iter_batched(
            || {
                let value1 = "value1".to_string(); // Repeat character six times to match the length of value strings used in other benchmarks
                let value2 = "value2".to_string();
                let value3 = "value3".to_string();
                let value4 = "value4".to_string();

                (value1, value2, value3, value4)
            },
            |values| {
                let kv = &[
                    KeyValue::new("attribute1", values.0),
                    KeyValue::new("attribute2", values.1),
                    KeyValue::new("attribute3", values.2),
                    KeyValue::new("attribute4", values.3),
                ];

                counter.add(1, kv);
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
