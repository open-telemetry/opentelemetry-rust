use criterion::{black_box, criterion_group, criterion_main, Criterion};
use opentelemetry::{logs::AnyValue, Value};

// Run this benchmark with:
// cargo bench --bench anyvalue
// Results:
// CreateOTelValue 1-2 ns
// CreateOTelAnyValue 15 ns

fn criterion_benchmark(c: &mut Criterion) {
    attributes_creation(c);
}

fn attributes_creation(c: &mut Criterion) {
    c.bench_function("CreateOTelValue", |b| {
        b.iter(|| {
            let _v = black_box(Value::String("value1".into()));
        });
    });

    c.bench_function("CreateOTelAnyValue", |b| {
        b.iter(|| {
            let _v = black_box(AnyValue::String("value1".into()));
        });
    });
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
