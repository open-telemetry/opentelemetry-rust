use criterion::{criterion_group, criterion_main, Criterion};
use opentelemetry::logs::AnyValue;
use opentelemetry::Key;
use opentelemetry_sdk::logs::HybridVec;

#[derive(Clone, Debug, PartialEq)]
pub struct KeyValuePair(Key, AnyValue);

impl Default for KeyValuePair {
    fn default() -> Self {
        KeyValuePair(Key::from_static_str(""), AnyValue::Int(0))
    }
}

fn hybridvec_insertion_benchmark(c: &mut Criterion) {
    c.bench_function("HybridVec Insertion", |b| {
        b.iter(|| {
            let mut collection = HybridVec::<KeyValuePair>::new();
            for i in 0..8 {
                let key = Key::from(format!("key{}", i));
                let value = AnyValue::Int(i as i64);
                collection.push(KeyValuePair(key, value));
            }
        })
    });
}

fn vec_insertion_benchmark(c: &mut Criterion) {
    c.bench_function("Vec Insertion", |b| {
        b.iter(|| {
            let mut collection = Vec::new();
            for i in 0..8 {
                let key = Key::from(format!("key{}", i));
                let value = AnyValue::Int(i as i64);
                collection.push(KeyValuePair(key, value));
            }
        })
    });
}

fn hybridvec_iteration_benchmark(c: &mut Criterion) {
    c.bench_function("HybridVec Iteration", |b| {
        let mut collection = HybridVec::<KeyValuePair>::new();
        for i in 0..8 {
            let key = Key::from(format!("key{}", i));
            let value = AnyValue::Int(i as i64);
            collection.push(KeyValuePair(key, value));
        }
        b.iter(|| {
            for item in &collection {
                criterion::black_box(item);
            }
        })
    });
}

fn hybridvec_get_benchmark(c: &mut Criterion) {
    c.bench_function("HybridVec Get Loop", |b| {
        let mut collection = HybridVec::<KeyValuePair>::new();
        for i in 0..8 {
            let key = Key::from(format!("key{}", i));
            let value = AnyValue::Int(i as i64);
            collection.push(KeyValuePair(key, value));
        }
        b.iter(|| {
            for i in 0..collection.len() {
                criterion::black_box(collection.get(i));
            }
        })
    });
}

fn vec_iteration_benchmark(c: &mut Criterion) {
    c.bench_function("Vec Iteration", |b| {
        let mut collection = Vec::new();
        for i in 0..8 {
            let key = Key::from(format!("key{}", i));
            let value = AnyValue::Int(i as i64);
            collection.push(KeyValuePair(key, value));
        }
        b.iter(|| {
            for item in &collection {
                criterion::black_box(item);
            }
        })
    });
}

criterion_group!(
    benches,
    hybridvec_insertion_benchmark,
    vec_insertion_benchmark,
    hybridvec_iteration_benchmark,
    hybridvec_get_benchmark,
    vec_iteration_benchmark
);
criterion_main!(benches);
