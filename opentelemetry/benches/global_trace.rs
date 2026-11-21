/*
    Benchmark for global tracer provider acquisition under contention.

    `global::tracer_provider()` hands every caller an owned handle (a clone
    of the provider's Arc), on hot paths and from many threads at once, so
    the cost of reading the global provider slot matters. This compares
    two read paths for that slot at thread counts 1, 2 and 8 plus
    `available_parallelism()`:

    - `rwlock`:  the slot as `OnceLock<RwLock<GlobalTracerProvider>>`, read
                 by taking the read lock and cloning the provider. This is
                 how the global provider used to be stored; readers contend
                 on the lock's shared reader count.
    - `current`: the current `global::tracer_provider()` entry point, which
                 reads the slot through an `ArcSwap` guard and clones the
                 provider out of it, without taking a lock.

    Run this benchmark with:
    cargo bench -p opentelemetry --bench global_trace
*/

use std::hint::black_box;
use std::sync::{OnceLock, RwLock};
use std::thread;
use std::thread::available_parallelism;
use std::time::{Duration, Instant};

use criterion::{
    criterion_group, criterion_main, BenchmarkGroup, BenchmarkId, Criterion, Throughput,
};
use opentelemetry::global::{self, GlobalTracerProvider};

/// RwLock-based global slot, lazily initialized with the default (no-op)
/// global provider.
static RWLOCK_GLOBAL_PROVIDER: OnceLock<RwLock<GlobalTracerProvider>> = OnceLock::new();

/// Thread counts: 1 (uncontended), contended 2 and 8, plus
/// `available_parallelism()` when it exceeds 8.
fn thread_counts() -> Vec<usize> {
    let mut counts = vec![1, 2, 8];
    let parallelism = available_parallelism().map(|n| n.get()).unwrap_or(1);
    if parallelism > 8 {
        counts.push(parallelism);
    }
    counts
}

/// RwLock read path: take the read lock and clone the provider out from
/// under it.
fn rwlock_acquire() -> GlobalTracerProvider {
    RWLOCK_GLOBAL_PROVIDER
        .get_or_init(|| RwLock::new(global::tracer_provider()))
        .read()
        .unwrap()
        .clone()
}

/// One measured operation; non-capturing closures coerce to this.
type Op = fn();

/// (variant name, acquisition op)
type Variant = (&'static str, Op);

const VARIANTS: [Variant; 2] = [
    ("rwlock", || {
        black_box(rwlock_acquire());
    }),
    ("current", || {
        black_box(global::tracer_provider());
    }),
];

/// Bench one operation across all thread counts. One Criterion iteration =
/// one operation per worker thread, so the reported time per iteration is
/// the per-op latency (under contention), and
/// `Throughput::Elements(threads)` makes Criterion report aggregate
/// ops/second.
fn bench_variant(
    group: &mut BenchmarkGroup<'_, criterion::measurement::WallTime>,
    variant: &str,
    op: Op,
) {
    for &threads in &thread_counts() {
        group.throughput(Throughput::Elements(threads as u64));
        group.bench_with_input(
            BenchmarkId::new(variant, threads),
            &threads,
            |b, threads| {
                let threads = *threads;
                b.iter_custom(|iters| {
                    let start = Instant::now();
                    thread::scope(|scope| {
                        for _ in 0..threads {
                            scope.spawn(|| {
                                for _ in 0..iters {
                                    op();
                                }
                            });
                        }
                    });
                    start.elapsed()
                });
            },
        );
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    // Initialize both slots so every measured path stays on its
    // steady-state fast path.
    rwlock_acquire();

    let mut group = c.benchmark_group("tracer_provider_acquisition");
    for (name, op) in VARIANTS {
        bench_variant(&mut group, name, op);
    }
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(3))
        .sample_size(30);
    targets = criterion_benchmark
}
criterion_main! { benches }
