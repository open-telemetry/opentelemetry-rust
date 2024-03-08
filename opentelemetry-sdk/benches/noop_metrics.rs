use criterion::{criterion_group, criterion_main, Criterion};
use opentelemetry::{
    metrics::{noop::NoopMeterProvider, Counter, MeterProvider as _},
    KeyValue,
};

use rand::{rngs::SmallRng, Rng, SeedableRng};

fn create_counter() -> Counter<u64> {
    let meter_provider: NoopMeterProvider = NoopMeterProvider::default();
    let meter = meter_provider.meter("benchmarks");
    let counter = meter.u64_counter("counter_bench").init();
    counter
}

fn criterion_benchmark(c: &mut Criterion) {
    noop_counter_add(c);
    create_keyvalue_vector(c);
}

fn noop_counter_add(c: &mut Criterion) {
    let attribute_values = [
        "value1", "value2", "value3", "value4", "value5", "value6", "value7", "value8", "value9",
        "value10",
    ];

    let mut rng = SmallRng::from_entropy();
    let num_samples = 1000; // Arbitrary number of samples for the benchmark
    let random_indices: Vec<(usize, usize, usize, usize)> = (0..num_samples)
        .map(|_| {
            (
                rng.gen_range(0..4),
                rng.gen_range(0..4),
                rng.gen_range(0..10),
                rng.gen_range(0..10),
            )
        })
        .collect();

    let noop_counter = create_counter();
    c.bench_function("Noop_Counter", |b| {
        // Use an iterator to cycle through the pre-generated indices.
        // This ensures that the benchmark does not exhaust the indices and each iteration gets a "random" set.
        let mut indices_iter = random_indices.iter().cycle();
        b.iter(|| {
            let (
                index_first_attribute,
                index_second_attribute,
                index_third_attribute,
                index_forth_attribute,
            ) = indices_iter.next().unwrap();
            noop_counter.add(
                1,
                &[
                    KeyValue::new("attribute1", attribute_values[*index_first_attribute]),
                    KeyValue::new("attribute2", attribute_values[*index_second_attribute]),
                    KeyValue::new("attribute3", attribute_values[*index_third_attribute]),
                    KeyValue::new("attribute4", attribute_values[*index_forth_attribute]),
                ],
            );
        });
    });

    c.bench_function("Create_KeyValue", |b| {
        let mut indices_iter = random_indices.iter().cycle();
        b.iter(|| {
            let (
                index_first_attribute,
                index_second_attribute,
                index_third_attribute,
                index_forth_attribute,
            ) = indices_iter.next().unwrap();
            let _ = vec![
                KeyValue::new("attribute1", attribute_values[*index_first_attribute]),
                KeyValue::new("attribute2", attribute_values[*index_second_attribute]),
                KeyValue::new("attribute3", attribute_values[*index_third_attribute]),
                KeyValue::new("attribute4", attribute_values[*index_forth_attribute]),
            ];
        });
    });
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
