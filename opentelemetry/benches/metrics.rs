/*
    Benchmark Results:
    criterion = "0.5.1"
    OS: macOS 15
    Hardware: Apple M4 Pro, 14 cores, RAM: 24 GB
    | Test                                                | Average time|
    |-----------------------------------------------------|-------------|
    | NoAttributes                                        |   0.77 ns   |
    | AddWithInlineStaticAttributes                       |   8.10 ns   |
    | AddWithStaticArray                                  |   0.77 ns   |
    | AddWithDynamicAttributes                            |  44.84 ns   |
    | AddWithDynamicAttributes_WithStringAllocation       |  85.92 ns   |
*/

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use opentelemetry::{global, metrics::Counter, KeyValue};
use std::hint::black_box;

// Run this benchmark with:
// cargo bench --bench metrics

fn create_counter() -> Counter<u64> {
    let meter = global::meter("benchmarks");

    meter.u64_counter("counter_bench").build()
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
                let value1 = "value1".to_string();
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

    c.bench_function("AddWithDynamicAttributes_WithStringAllocation", |b| {
        b.iter(|| {
            let kv = &[
                KeyValue::new("attribute1", black_box("value1".to_string())),
                KeyValue::new("attribute2", black_box("value2".to_string())),
                KeyValue::new("attribute3", black_box("value3".to_string())),
                KeyValue::new("attribute4", black_box("value4".to_string())),
            ];

            counter.add(1, kv);
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
