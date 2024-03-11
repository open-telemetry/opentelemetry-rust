use criterion::{black_box, criterion_group, criterion_main, Criterion};
use opentelemetry::{
    metrics::{noop::NoopMeterProvider, Counter, MeterProvider as _},
    KeyValue,
};

fn create_counter() -> Counter<u64> {
    let meter_provider: NoopMeterProvider = NoopMeterProvider::default();
    let meter = meter_provider.meter("benchmarks");
    let counter = meter.u64_counter("counter_bench").init();
    counter
}

fn criterion_benchmark(c: &mut Criterion) {
    noop_counter_add(c);
}

fn noop_counter_add(c: &mut Criterion) {
    let noop_counter = create_counter();

    c.bench_function("NoopCounter_NoAttributes", |b| {
        b.iter(|| {
            noop_counter.add(1, &[]);
        });
    });

    c.bench_function("NoopCounter_AddWithInlineStaticAttributes", |b| {
        b.iter(|| {
            noop_counter.add(
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

    c.bench_function("NoopCounter_AddWithStaticArray", |b| {
        b.iter(|| {
            let kv = [
                KeyValue::new("attribute1", "value1"),
                KeyValue::new("attribute2", "value2"),
                KeyValue::new("attribute3", "value3"),
                KeyValue::new("attribute4", "value4"),
            ];

            noop_counter.add(1, &kv);
        });
    });

    c.bench_function("NoopCounter_AddWithDynamicAttributes", |b| {
        b.iter(|| {
            let kv = vec![
                KeyValue::new("attribute1", "value1"),
                KeyValue::new("attribute2", "value2"),
                KeyValue::new("attribute3", "value3"),
                KeyValue::new("attribute4", "value4"),
            ];

            noop_counter.add(1, &kv);
        });
    });

    #[allow(clippy::useless_vec)]
    c.bench_function("CreateVector_KeyValue", |b| {
        b.iter(|| {
            let _v1 = vec![
                KeyValue::new("attribute1", "value1"),
                KeyValue::new("attribute2", "value2"),
                KeyValue::new("attribute3", "value3"),
                KeyValue::new("attribute4", "value4"),
            ];
        });
    });

    #[allow(clippy::useless_vec)]
    c.bench_function("CreateDynamicVector_StringPair", |b| {
        b.iter(|| {
            let _v1 = vec![
                ("attribute1", "value1"),
                ("attribute2", "value2"),
                ("attribute3", "value3"),
                ("attribute4", "value4"),
            ];
        });
    });
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
