use criterion::{black_box, criterion_group, criterion_main, Criterion};
use opentelemetry::KeyValue;

// Run this benchmark with:
// cargo bench --bench attributes

fn criterion_benchmark(c: &mut Criterion) {
    attributes_creation(c);
}

fn attributes_creation(c: &mut Criterion) {
    c.bench_function("CreateOTelKeyValue", |b| {
        b.iter(|| {
            let _v1 = black_box(KeyValue::new("attribute1", "value1"));
        });
    });

    c.bench_function("CreateKeyValueTuple", |b| {
        b.iter(|| {
            let _v1 = black_box(("attribute1", "value1"));
        });
    });

    #[allow(clippy::useless_vec)]
    c.bench_function("CreateVector_KeyValue", |b| {
        b.iter(|| {
            let _v1 = black_box(vec![
                KeyValue::new("attribute1", "value1"),
                KeyValue::new("attribute2", "value2"),
                KeyValue::new("attribute3", "value3"),
                KeyValue::new("attribute4", "value4"),
            ]);
        });
    });

    #[allow(clippy::useless_vec)]
    c.bench_function("CreateVector_StringPairs", |b| {
        b.iter(|| {
            let _v1 = black_box(vec![
                ("attribute1", "value1"),
                ("attribute2", "value2"),
                ("attribute3", "value3"),
                ("attribute4", "value4"),
            ]);
        });
    });
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
