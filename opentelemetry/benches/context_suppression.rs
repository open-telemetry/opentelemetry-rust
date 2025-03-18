use criterion::{black_box, criterion_group, criterion_main, Criterion};
use opentelemetry::Context;

// Run this benchmark with:
// cargo bench --bench context_suppression

// The benchmark results:
// criterion = "0.5.1"
// Hardware: Apple M4 Pro
// Total Number of Cores:   14 (10 performance and 4 efficiency)
// | Benchmark                  | Time (ns) |
// |----------------------------|-----------|
// | enter_suppressed           | 9.0       |
// | normal_attach              | 9.0       |
// | is_current_suppressed_false| 1.2       |
// | is_current_suppressed_true | 1.2       |

fn criterion_benchmark(c: &mut Criterion) {
    // Original benchmarks...

    // New benchmarks for telemetry suppression
    suppression_benchmarks(c);
}

fn suppression_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("telemetry_suppression");

    // Benchmark the cost of entering a suppressed scope
    group.bench_function("enter_suppressed", |b| {
        b.iter(|| {
            let _guard = black_box(Context::enter_suppressed());
            let _ = black_box(dummy_work());
        });
    });

    // For comparison - normal context attach
    group.bench_function("normal_attach", |b| {
        b.iter(|| {
            let _guard = black_box(Context::current().attach());
            let _ = black_box(dummy_work());
        });
    });

    // Benchmark checking if current is suppressed (when not suppressed)
    group.bench_function("is_current_suppressed_false", |b| {
        // Make sure we're in a non-suppressed context
        let _restore_ctx = Context::current().attach();
        b.iter(|| {
            let is_suppressed = black_box(Context::is_current_suppressed());
            black_box(is_suppressed);
        });
    });

    // Benchmark checking if current is suppressed (when suppressed)
    group.bench_function("is_current_suppressed_true", |b| {
        // Enter suppressed context for the duration of the benchmark
        let _suppressed_guard = Context::enter_suppressed();
        b.iter(|| {
            let is_suppressed = black_box(Context::is_current_suppressed());
            black_box(is_suppressed);
        });
    });

    group.finish();
}

#[inline(never)]
fn dummy_work() -> i32 {
    black_box(1 + 1)
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
