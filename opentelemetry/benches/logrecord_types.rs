use criterion::{black_box, criterion_group, criterion_main, Criterion};
use opentelemetry::{logs::AnyValue, Key, KeyValue};

// Run this benchmark with:
// cargo bench --bench logrecord_types
// Results:
// CreateOTelKeyValue 2-3 ns
// CreateOTelKeyAnyValue 30 ns
// CreateTupleKeyValue < 1 ns

fn criterion_benchmark(c: &mut Criterion) {
    attributes_creation(c);
}

fn attributes_creation(c: &mut Criterion) {
    c.bench_function("CreateOTelKeyValue", |b| {
        b.iter(|| {
            let _v1 = black_box(KeyValue::new("attribute1", "value1"));
        });
    });

    c.bench_function("CreateOTelKeyAnyValue", |b| {
        b.iter(|| {            
            let _k= black_box(Key::new("attribute1"));
            let _v1 = black_box(AnyValue::String("value1".to_string().into()));
        });
    });

    c.bench_function("CreateTupleKeyValue", |b| {
        b.iter(|| {
            let _v1 = black_box(("attribute1", "value1"));
        });
    });
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
