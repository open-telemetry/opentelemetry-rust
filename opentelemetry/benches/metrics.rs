use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
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

    c.bench_function("Counter_NoAttributes", |b| {
        b.iter(|| {
            counter.add(1, &[]);
        });
    });

    c.bench_function("Counter_AddWithInlineStaticAttributes", |b| {
        b.iter(|| {
            counter.add(
                1,
                &[
                    black_box(KeyValue::new("attribute1", "value1")),
                    black_box(KeyValue::new("attribute2", "value2")),
                    black_box(KeyValue::new("attribute3", "value3")),
                    black_box(KeyValue::new("attribute4", "value4")),
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

    c.bench_function("Counter_AddWithStaticArray", |b| {
        b.iter(|| {
            counter.add(1, &kv);
        });
    });

    c.bench_function("Counter_AddWithDynamicAttributes", |b| {
        b.iter_batched(
            || {
                let value1 = black_box("a".repeat(6)); // Repeat character six times to match the length of value strings used in other benchmarks
                let value2 = black_box("b".repeat(6));
                let value3 = black_box("c".repeat(6));
                let value4 = black_box("d".repeat(6));

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
