use criterion::{
    criterion_group, criterion_main, measurement::Measurement, BenchmarkGroup, BenchmarkId,
    Criterion,
};
use opentelemetry_api::metrics::MeterProvider as _;
use opentelemetry_api::{Context, Key, KeyValue};
use opentelemetry_sdk::metrics::MeterProvider;
use rand::{rngs, Rng};
use std::cell::RefCell;

pub fn counters(c: &mut Criterion) {
    let meter = MeterProvider::builder().build().meter("bench");

    let mut g = c.benchmark_group("Counter");
    let cx = Context::new();

    let counter = meter.u64_counter("u64.sum").init();
    benchmark_unbound_metric("u64", &mut g, |attributes| counter.add(&cx, 1, attributes));

    let counter = meter.f64_counter("f64.sum").init();
    benchmark_unbound_metric("f64", &mut g, |attributes| {
        counter.add(&cx, 1.0, attributes)
    });

    g.finish();
}

fn benchmark_unbound_metric<M: Measurement, F: Fn(&[KeyValue])>(
    name: &str,
    g: &mut BenchmarkGroup<M>,
    f: F,
) {
    for (num, kvs) in [
        ("1", build_kv(1)),
        ("2", build_kv(2)),
        ("4", build_kv(4)),
        ("8", build_kv(8)),
        ("16", build_kv(16)),
    ]
    .iter()
    {
        g.bench_with_input(BenchmarkId::new(name, num), kvs, |b, kvs| b.iter(|| f(kvs)));
    }
}

fn build_kv(n: u8) -> Vec<KeyValue> {
    let mut res = Vec::new();

    CURRENT_RNG.with(|rng| {
        let mut rng = rng.borrow_mut();
        for _ in 0..n {
            let k = Key::new(format!("k_{}", rng.gen::<f64>() * 1_000_000_000.0));
            res.push(k.string(format!("v_{}", rng.gen::<f64>() * 1_000_000_000.0)));
        }
    });

    res
}
thread_local! {
    static CURRENT_RNG: RefCell<rngs::ThreadRng> = RefCell::new(rngs::ThreadRng::default());
}

criterion_group!(benches, counters);
criterion_main!(benches);
