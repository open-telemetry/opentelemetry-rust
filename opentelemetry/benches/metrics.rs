use criterion::{criterion_group, criterion_main, Criterion};
use opentelemetry::{global, metrics::Counter, KeyValue};

// Run this benchmark with:
// cargo bench --bench metrics --features=metrics

fn create_counter() -> Counter<u64> {
    let meter = global::meter("benchmarks");
    let counter = meter.u64_counter("counter_bench").init();
    counter
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

    c.bench_function("Counter_AddWithStaticArray", |b| {
        b.iter(|| {
            counter.add(1, &kv);
        });
    });

    c.bench_function("Counter_AddWithDynamicAttributes", |b| {
        b.iter(|| {
            let kv = vec![
                KeyValue::new("attribute1", "value1"),
                KeyValue::new("attribute2", "value2"),
                KeyValue::new("attribute3", "value3"),
                KeyValue::new("attribute4", "value4"),
            ];

            counter.add(1, &kv);
        });
    });
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
