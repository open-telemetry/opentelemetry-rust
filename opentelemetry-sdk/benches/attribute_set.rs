use criterion::{criterion_group, criterion_main, Criterion};
use opentelemetry::attributes::AttributeSet;
use opentelemetry::KeyValue;

// Run this benchmark with:
// cargo bench --bench metric_counter

fn criterion_benchmark(c: &mut Criterion) {
    attribute_set(c);
}

fn attribute_set(c: &mut Criterion) {
    c.bench_function("AttributeSet_without_duplicates", |b| {
        b.iter(|| {
            let attributes = [
                KeyValue::new("attribute1", "value1"),
                KeyValue::new("attribute2", "value2"),
                KeyValue::new("attribute3", "value3"),
                KeyValue::new("attribute4", "value4"),
            ];
            let _attribute_set: AttributeSet = AttributeSet::from(&attributes);
        });
    });

    c.bench_function("AttributeSet_with_duplicates", |b| {
        b.iter(|| {
            let attributes = [
                KeyValue::new("attribute1", "value1"),
                KeyValue::new("attribute3", "value3"),
                KeyValue::new("attribute3", "value3"),
                KeyValue::new("attribute4", "value4"),
            ];
            let _attribute_set: AttributeSet = AttributeSet::from(&attributes);
        });
    });
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
