use criterion::{black_box, criterion_group, criterion_main, Criterion};
use opentelemetry::{Key, KeyValue};
use std::sync::Arc;

// Run this benchmark with:
// cargo bench --bench attributes

fn criterion_benchmark(c: &mut Criterion) {
    attributes_creation(c);
}

fn attributes_creation(c: &mut Criterion) {
    c.bench_function("CreateOTelKey_Static", |b| {
        b.iter(|| {
            let _v1 = black_box(Key::new("attribute1"));
        });
    });

    c.bench_function("CreateOTelKey_Owned", |b| {
        b.iter(|| {
            let _v1 = black_box(Key::new(String::from("attribute1")));
        });
    });

    c.bench_function("CreateOTelKey_Arc", |b| {
        b.iter(|| {
            let _v1 = black_box(Key::new(Arc::from("attribute1")));
        });
    });

    c.bench_function("CreateOTelKeyValue", |b| {
        b.iter(|| {
            let _v1 = black_box(KeyValue::new("attribute1", "value1"));
        });
    });

    c.bench_function("CreateTupleKeyValue", |b| {
        b.iter(|| {
            let _v1 = black_box(("attribute1", "value1"));
        });
    });

    c.bench_function("CreateTupleKeyValueUsingGenerics", |b| {
        b.iter(|| {
            let _v1 = black_box(no_op("attribute1", "value1"));
        });
    });

    #[allow(clippy::useless_vec)]
    c.bench_function("CreateOtelKeyValueVector", |b| {
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
    c.bench_function("CreateTupleKeyValueVector", |b| {
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


trait OTelValueType {}

impl OTelValueType for u32 {}

impl  OTelValueType for &str {}

fn no_op<T : OTelValueType>(key: &'static str, value: T) {
    black_box("test");
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
