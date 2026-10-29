//! Measures steady-state metric overflow throughput under contention.
//!
//! The counter is filled to its cardinality limit before each measurement. All
//! benchmark threads then record an attribute set that routes to the existing
//! overflow bucket.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use opentelemetry::{
    metrics::{Counter, MeterProvider as _},
    KeyValue,
};
use opentelemetry_sdk::metrics::{ManualReader, SdkMeterProvider};
use std::{
    sync::{Arc, Barrier},
    thread,
    time::{Duration, Instant},
};

const CARDINALITY_LIMIT: u64 = 2_000;

fn build_overflowed_counter() -> Counter<u64> {
    let provider = SdkMeterProvider::builder()
        .with_reader(ManualReader::builder().build())
        .build();
    let meter = provider.meter("overflow_mt_bench");
    let counter = meter.u64_counter("overflow_mt").build();

    for value in 0..=CARDINALITY_LIMIT {
        counter.add(1, &[KeyValue::new("prefill", value.to_string())]);
    }

    counter
}

fn record_overflow(counter: &Arc<Counter<u64>>, thread_count: usize, iterations: u64) -> Duration {
    let barrier = Arc::new(Barrier::new(thread_count));
    let handles: Vec<_> = (0..thread_count)
        .map(|thread_id| {
            let counter = Arc::clone(counter);
            let barrier = Arc::clone(&barrier);
            let thread_iterations = iterations / thread_count as u64
                + u64::from((thread_id as u64) < iterations % thread_count as u64);

            thread::spawn(move || {
                let attributes = [
                    KeyValue::new("attribute2", "beta"),
                    KeyValue::new("attribute1", "alpha"),
                    KeyValue::new("thread", thread_id as i64),
                ];
                barrier.wait();
                let start = Instant::now();
                for _ in 0..thread_iterations {
                    counter.add(1, &attributes);
                }
                start.elapsed()
            })
        })
        .collect();

    handles
        .into_iter()
        .map(|handle| handle.join().expect("benchmark thread panicked"))
        .max()
        .unwrap_or_default()
}

fn bench_overflow_threads(c: &mut Criterion) {
    let mut group = c.benchmark_group("Counter_Overflow_MT");
    group.sample_size(30);

    for thread_count in [1, 2, 4, 8] {
        group.bench_with_input(
            BenchmarkId::new("threads", thread_count),
            &thread_count,
            |b, &thread_count| {
                b.iter_custom(|iterations| {
                    let counter = Arc::new(build_overflowed_counter());
                    record_overflow(&counter, thread_count, iterations)
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_overflow_threads);
criterion_main!(benches);
