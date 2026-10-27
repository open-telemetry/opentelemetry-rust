use criterion::{criterion_group, criterion_main, Criterion};
use opentelemetry::{
    propagation::TextMapPropagator,
    trace::{SpanContext, SpanId, TraceContextExt, TraceFlags, TraceId, TraceState},
    Context,
};
use opentelemetry_sdk::propagation::TraceContextPropagator;
#[cfg(all(not(target_os = "windows"), feature = "bench_profiling"))]
use pprof::criterion::{Output, PProfProfiler};
use std::collections::HashMap;
use std::hint::black_box;

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("trace_context_propagator");
    let propagator = TraceContextPropagator::new();

    for (name, traceparent) in [
        (
            "valid",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
        ),
        (
            "invalid",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-qw",
        ),
    ] {
        let extractor = HashMap::from([("traceparent".to_string(), traceparent.to_string())]);
        group.bench_function(format!("extract/{name}"), |b| {
            b.iter(|| black_box(propagator.extract(black_box(&extractor))))
        });
    }

    let cx = Context::new().with_remote_span_context(SpanContext::new(
        TraceId::from(0x4bf9_2f35_77b3_4da6_a3ce_929d_0e0e_4736),
        SpanId::from(0x00f0_67aa_0ba9_02b7),
        TraceFlags::SAMPLED,
        true,
        TraceState::default(),
    ));
    group.bench_function("inject", |b| {
        b.iter(|| {
            let mut injector = HashMap::with_capacity(1);
            propagator.inject_context(black_box(&cx), &mut injector);
            black_box(injector)
        })
    });
}

#[cfg(all(not(target_os = "windows"), feature = "bench_profiling"))]
criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(std::time::Duration::from_secs(1))
        .measurement_time(std::time::Duration::from_secs(2))
        .with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = criterion_benchmark
}

#[cfg(any(target_os = "windows", not(feature = "bench_profiling")))]
criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(std::time::Duration::from_secs(1))
        .measurement_time(std::time::Duration::from_secs(2));
    targets = criterion_benchmark
}

criterion_main!(benches);
