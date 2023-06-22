use criterion::{
    black_box, criterion_group, criterion_main, BatchSize::SmallInput, BenchmarkId, Criterion,
};
use indexmap::IndexMap;
use opentelemetry_api::{Key, KeyValue, Value};
use opentelemetry_sdk::trace::EvictedHashMap;
use pprof::criterion::{Output, PProfProfiler};
use std::iter::Iterator;

fn criterion_benchmark(c: &mut Criterion) {
    let cap = 32;
    let input = [(2, 32, cap), (8, 32, cap), (32, 32, cap)];
    populate_benchmark(c, &input);
    lookup_benchmark(c, &input);
    populate_and_lookup_benchmark(c, &input);
}

fn populate_benchmark(c: &mut Criterion, input: &[(usize, u32, usize)]) {
    let mut group = c.benchmark_group("populate");
    for &(n, max, capacity) in input {
        let parameter_string = format!("{n:02}/{max:02}/{capacity:02}");

        group.bench_function(
            BenchmarkId::new("EvictedHashMap", parameter_string.clone()),
            |b| {
                b.iter(|| populate_evicted_hashmap(n, max, capacity));
            },
        );
        group.bench_function(
            BenchmarkId::new("IndexMap", parameter_string.clone()),
            |b| {
                b.iter(|| populate_indexmap(n, max, capacity));
            },
        );
        group.bench_function(BenchmarkId::new("TwoVecs", parameter_string.clone()), |b| {
            b.iter(|| populate_twovecs(n, max, capacity));
        });
        group.bench_function(BenchmarkId::new("OneVec", parameter_string.clone()), |b| {
            b.iter(|| populate_onevec(n, max, capacity));
        });
    }
    group.finish();
}

fn lookup_benchmark(c: &mut Criterion, input: &[(usize, u32, usize)]) {
    let mut group = c.benchmark_group("lookup");
    for &(n, max, capacity) in input {
        let lookup_keys = &MAP_KEYS[n - 2..n];
        let parameter_string = format!("{n:02}/{max:02}/{capacity:02}");
        group.bench_function(
            BenchmarkId::new("EvictedHashMap", parameter_string.clone()),
            |b| {
                b.iter_batched(
                    || populate_evicted_hashmap(n, max, capacity),
                    |map| lookup_evicted_hashmap(&map, lookup_keys),
                    SmallInput,
                );
            },
        );
        group.bench_function(
            BenchmarkId::new("IndexMap", parameter_string.clone()),
            |b| {
                b.iter_batched(
                    || populate_indexmap(n, max, capacity),
                    |map| lookup_indexmap(&map, lookup_keys),
                    SmallInput,
                );
            },
        );
        group.bench_function(BenchmarkId::new("OneVec", parameter_string.clone()), |b| {
            b.iter_batched(
                || populate_onevec(n, max, capacity),
                |vec| lookup_onevec(&vec, lookup_keys),
                SmallInput,
            );
        });
        group.bench_function(BenchmarkId::new("TwoVecs", parameter_string.clone()), |b| {
            b.iter_batched(
                || populate_twovecs(n, max, capacity),
                |(keys, vals)| lookup_twovec(&keys, &vals, lookup_keys),
                SmallInput,
            );
        });
    }
    group.finish();
}

fn populate_and_lookup_benchmark(c: &mut Criterion, input: &[(usize, u32, usize)]) {
    let mut group = c.benchmark_group("populate_and_lookup");
    for &(n, max, capacity) in input {
        let lookup_keys = &MAP_KEYS[n - 2..n];
        let parameter_string = format!("{n:02}/{max:02}/{capacity:02}");
        group.bench_function(
            BenchmarkId::new("EvictedHashMap", parameter_string.clone()),
            |b| {
                b.iter(|| {
                    let map = populate_evicted_hashmap(n, max, capacity);
                    lookup_evicted_hashmap(&map, lookup_keys);
                });
            },
        );
        group.bench_function(
            BenchmarkId::new("IndexMap", parameter_string.clone()),
            |b| {
                b.iter(|| {
                    let map = populate_indexmap(n, max, capacity);
                    lookup_indexmap(&map, lookup_keys);
                });
            },
        );
        group.bench_function(BenchmarkId::new("OneVec", parameter_string.clone()), |b| {
            b.iter(|| {
                let vec = populate_onevec(n, max, capacity);
                lookup_onevec(&vec, lookup_keys);
            });
        });
        group.bench_function(BenchmarkId::new("TwoVecs", parameter_string.clone()), |b| {
            b.iter(|| {
                let (keys, vals) = populate_twovecs(n, max, capacity);
                lookup_twovec(&keys, &vals, lookup_keys);
            });
        });
    }
    group.finish();
}

fn populate_evicted_hashmap(n: usize, max: u32, capacity: usize) -> EvictedHashMap {
    let mut map = EvictedHashMap::new(max, capacity);
    for (idx, key) in MAP_KEYS.iter().enumerate().take(n) {
        map.insert(KeyValue::new(*key, idx as i64));
    }
    map
}

fn lookup_evicted_hashmap(map: &EvictedHashMap, keys: &[&'static str]) {
    for key in keys {
        black_box(map.get(&Key::new(*key)));
    }
}

fn populate_indexmap(n: usize, max: u32, _capacity: usize) -> IndexMap<Key, Value> {
    let mut map = IndexMap::with_capacity(max as usize);
    for (idx, key) in MAP_KEYS.iter().enumerate().take(n) {
        map.insert(Key::new(*key), Value::I64(idx as i64));
    }
    map
}

fn lookup_indexmap(map: &IndexMap<Key, Value>, keys: &[&'static str]) {
    for key in keys {
        black_box(map.get(&Key::new(*key)));
    }
}

fn populate_onevec(n: usize, max: u32, _capacity: usize) -> Vec<(Key, Value)> {
    let mut tuples = Vec::with_capacity(max as usize);
    for (idx, key) in MAP_KEYS.iter().enumerate().take(n) {
        tuples.push((Key::new(*key), Value::I64(idx as i64)));
    }
    tuples
}

fn lookup_onevec(vec: &[(Key, Value)], keys: &[&'static str]) {
    for key in keys {
        black_box(
            vec.iter()
                .position(|(k, _v)| *k == Key::new(*key))
                .map(|idx| vec.get(idx)),
        );
    }
}

fn populate_twovecs(n: usize, max: u32, _capacity: usize) -> (Vec<Key>, Vec<Value>) {
    let mut keys = Vec::with_capacity(max as usize);
    let mut vals = Vec::with_capacity(max as usize);
    for (idx, key) in MAP_KEYS.iter().enumerate().take(n) {
        keys.push(Key::new(*key));
        vals.push(Value::I64(idx as i64));
    }
    (keys, vals)
}

fn lookup_twovec(keys: &[Key], vals: &[Value], lookup_keys: &[&'static str]) {
    for key in lookup_keys {
        black_box(
            keys.iter()
                .position(|k| *k == Key::new(*key))
                .map(|idx| vals.get(idx)),
        );
    }
}

const MAP_KEYS: [&str; 64] = [
    "key.1", "key.2", "key.3", "key.4", "key.5", "key.6", "key.7", "key.8", "key.9", "key.10",
    "key.11", "key.12", "key.13", "key.14", "key.15", "key.16", "key.17", "key.18", "key.19",
    "key.20", "key.21", "key.22", "key.23", "key.24", "key.25", "key.26", "key.27", "key.28",
    "key.29", "key.30", "key.31", "key.32", "key.33", "key.34", "key.35", "key.36", "key.37",
    "key.38", "key.39", "key.40", "key.41", "key.42", "key.43", "key.44", "key.45", "key.46",
    "key.47", "key.48", "key.49", "key.50", "key.51", "key.52", "key.53", "key.54", "key.55",
    "key.56", "key.57", "key.58", "key.59", "key.60", "key.61", "key.62", "key.63", "key.64",
];

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = criterion_benchmark
}
criterion_main!(benches);
