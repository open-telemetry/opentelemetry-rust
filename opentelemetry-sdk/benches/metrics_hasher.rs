//! Compares the metrics hot path (`Counter::add`) using the default
//! DoS-resistant `RandomState` (SipHash-1-3) against a user-supplied
//! non-cryptographic hasher (`foldhash`) installed via
//! [`MeterProviderBuilder::with_hasher`].
//!
//! A single fixed attribute set is recorded repeatedly, so the only difference
//! between the two benchmarks is the hash builder threaded into the internal
//! `ValueMap` trackers map. This isolates the per-measurement hashing cost.
//!
//! Run with:
//! ```text
//! cargo bench --bench metrics_hasher --features metrics,experimental_metrics_custom_reader
//! ```
//!
//! Results (Criterion median per `Counter::add`, AMD Ryzen Threadripper 1900X):
//!
//! ```text
//! Counter_Add_SipHash_Default   ~153 ns
//! Counter_Add_Foldhash          ~108 ns   (~30% faster)
//! ```

use criterion::{criterion_group, criterion_main, Criterion};
use opentelemetry::{
    metrics::{Counter, MeterProvider as _},
    KeyValue,
};
use opentelemetry_sdk::metrics::{ManualReader, SdkMeterProvider};
use std::hash::BuildHasher;

fn counter_with_hasher<S>(name: &'static str, hash_builder: S) -> Counter<u64>
where
    S: BuildHasher + Clone + Send + Sync + 'static,
{
    let meter_provider = SdkMeterProvider::builder()
        .with_hasher(hash_builder)
        .with_reader(ManualReader::builder().build())
        .build();
    meter_provider.meter("benchmarks").u64_counter(name).build()
}

fn bench_counter(c: &mut Criterion, label: &str, counter: Counter<u64>) {
    // A single fixed attribute set: every iteration hits the same tracker, so
    // the measured delta is purely the hasher cost on a steady-state lookup.
    let attributes = [
        KeyValue::new("attribute1", "value1"),
        KeyValue::new("attribute2", "value2"),
        KeyValue::new("attribute3", "value3"),
        KeyValue::new("attribute4", "value4"),
    ];
    c.bench_function(label, |b| {
        b.iter(|| {
            counter.add(1, &attributes);
        });
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    // Default: standard library `RandomState` (SipHash-1-3, DoS-resistant).
    bench_counter(
        c,
        "Counter_Add_SipHash_Default",
        counter_with_hasher(
            "Counter_Add_SipHash_Default",
            std::collections::hash_map::RandomState::new(),
        ),
    );

    // Bring-your-own: foldhash (fast, non-DoS-resistant).
    bench_counter(
        c,
        "Counter_Add_Foldhash",
        counter_with_hasher(
            "Counter_Add_Foldhash",
            foldhash::fast::RandomState::default(),
        ),
    );
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(std::time::Duration::from_secs(1))
        .measurement_time(std::time::Duration::from_secs(2));
    targets = criterion_benchmark
}
criterion_main!(benches);
