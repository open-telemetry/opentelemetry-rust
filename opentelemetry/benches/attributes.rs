use criterion::{black_box, criterion_group, criterion_main, Criterion};
use opentelemetry::{Key, KeyValue};
use std::sync::Arc;

// Run this benchmark with:
// cargo bench --bench attributes

fn criterion_benchmark(c: &mut Criterion) {
    attributes_creation(c);
}

fn use_dummy<'a>(_a: &'a KeyValue, _b: &'a KeyValue, _c: &'a KeyValue, _d: &'a KeyValue) {
    // Intentionally left blank, or perform some minimal operation that cannot be optimized out
}

fn attributes_creation(c: &mut Criterion) {
    c.bench_function("CreateOTelKey_Static", |b| {
        b.iter(|| {
            let v1 = black_box(Key::new("attribute1"));
            std::mem::forget(v1);
        });
    });

    c.bench_function("CreateOTelKey_Owned", |b| {
        b.iter(|| {
            let v1 = black_box(Key::new(String::from("attribute1")));
            std::mem::forget(v1);
        });
    });

    c.bench_function("CreateOTelKey_Arc", |b| {
        b.iter(|| {
            let v1 = black_box(Key::new(Arc::from("attribute1")));
            std::mem::forget(v1);
        });
    });

    c.bench_function("CreateOTelKeyValue", |b| {
        b.iter(|| {
            let v1 = black_box(KeyValue::new("attribute1", "value1"));
            std::mem::forget(v1);
        });
    });

    c.bench_function("CreateTupleKeyValue", |b| {
        b.iter(|| {
            let v1 = black_box(("attribute1", "value1"));
            let _ = v1.0.len() + v1.1.len();
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
    /*
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
    */
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
