use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use opentelemetry::baggage::Baggage;
use rand::distr::{Alphanumeric, SampleString};
use rand::Rng;

const MAX_KEY_VALUE_PAIRS: usize = 64;

fn criterion_benchmark(c: &mut Criterion) {
    set_baggage_value(c);
    set_baggage_value_with_metadata(c);
}

fn set_baggage_value(c: &mut Criterion) {
    let mut baggage = Baggage::new();

    let mut rng = rand::rng();
    let key_value = (0..MAX_KEY_VALUE_PAIRS)
        .map(|_| {
            (
                Alphanumeric.sample_string(&mut rng, 4),
                Alphanumeric.sample_string(&mut rng, 4),
            )
        })
        .collect::<Vec<(String, String)>>();

    c.bench_function("set_baggage_value", move |b| {
        b.iter_batched(
            || rng.random_range(0..MAX_KEY_VALUE_PAIRS),
            |idx| {
                let (key, value) = key_value[idx].clone();
                baggage.insert(key, value);
            },
            BatchSize::SmallInput,
        )
    });
}

fn set_baggage_value_with_metadata(c: &mut Criterion) {
    let mut baggage = Baggage::new();

    let mut rng = rand::rng();
    let key_value_metadata = (0..MAX_KEY_VALUE_PAIRS)
        .map(|_| {
            (
                Alphanumeric.sample_string(&mut rng, 4),
                Alphanumeric.sample_string(&mut rng, 4),
                Alphanumeric.sample_string(&mut rng, 4),
            )
        })
        .collect::<Vec<(String, String, String)>>();

    c.bench_function("set_baggage_value_with_metadata", move |b| {
        b.iter_batched(
            || rng.random_range(0..MAX_KEY_VALUE_PAIRS),
            |idx| {
                let (key, value, metadata) = key_value_metadata[idx].clone();
                baggage.insert_with_metadata(key, value, metadata);
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
