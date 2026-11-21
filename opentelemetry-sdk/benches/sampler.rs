use criterion::{criterion_group, criterion_main, Criterion};
use opentelemetry::{
    trace::{SpanContext, SpanId, SpanKind, TraceContextExt, TraceFlags, TraceId, TraceState},
    Context,
};
use opentelemetry_sdk::trace::{ProbabilitySampler, ShouldSample};

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("probability-sampler");

    let sampler_half = ProbabilitySampler::new(0.5);
    let sampler_one = ProbabilitySampler::new(1.0);

    let trace_id_sample = TraceId::from(0x0000000000000000_0080000000000000u128);
    let trace_id_drop = TraceId::from(0x0000000000000000_007fffffffffffffu128);
    let span_id = SpanId::from(1);
    let parent_with_vendor = Context::new().with_remote_span_context(SpanContext::new(
        trace_id_drop,
        span_id,
        TraceFlags::RANDOM,
        true,
        TraceState::from_key_value([("vendor", "value")]).unwrap(),
    ));
    let parent_with_th = Context::new().with_remote_span_context(SpanContext::new(
        trace_id_drop,
        span_id,
        TraceFlags::RANDOM,
        true,
        TraceState::from_key_value([("ot", "th:0;other:value"), ("vendor", "value")]).unwrap(),
    ));

    let state_with_rv =
        TraceState::from_key_value([("ot", "rv:80000000000000;other:value"), ("vendor", "value")])
            .unwrap();
    let state_with_low_rv =
        TraceState::from_key_value([("ot", "rv:00000000000000;other:value"), ("vendor", "value")])
            .unwrap();
    let state_with_existing_th = TraceState::from_key_value([
        ("ot", "th:0;rv:80000000000000;other:value"),
        ("vendor", "value"),
    ])
    .unwrap();

    let parent_with_rv = Context::current().with_remote_span_context(SpanContext::new(
        trace_id_sample,
        span_id,
        TraceFlags::default(),
        true,
        state_with_rv,
    ));
    let parent_with_low_rv = Context::current().with_remote_span_context(SpanContext::new(
        trace_id_drop,
        span_id,
        TraceFlags::default(),
        true,
        state_with_low_rv,
    ));
    let parent_with_existing_th = Context::current().with_remote_span_context(SpanContext::new(
        trace_id_sample,
        span_id,
        TraceFlags::default(),
        true,
        state_with_existing_th,
    ));

    group.bench_function("sampling-without-tracestate", |b| {
        b.iter(|| {
            sampler_half.should_sample(
                None,
                trace_id_sample,
                "bench",
                &SpanKind::Internal,
                &[],
                &[],
            )
        });
    });

    group.bench_function("dropping-without-tracestate", |b| {
        b.iter(|| {
            sampler_half.should_sample(None, trace_id_drop, "bench", &SpanKind::Internal, &[], &[])
        });
    });

    group.bench_function("sampling-with-explicit-rv", |b| {
        b.iter(|| {
            sampler_half.should_sample(
                Some(&parent_with_rv),
                trace_id_sample,
                "bench",
                &SpanKind::Internal,
                &[],
                &[],
            )
        });
    });

    for (name, parent) in [
        ("dropping-with-vendor", &parent_with_vendor),
        ("dropping-with-existing-th", &parent_with_th),
    ] {
        group.bench_function(name, |b| {
            b.iter(|| {
                sampler_half.should_sample(
                    Some(parent),
                    trace_id_drop,
                    "bench",
                    &SpanKind::Internal,
                    &[],
                    &[],
                )
            });
        });
    }

    group.bench_function("dropping-with-explicit-rv", |b| {
        b.iter(|| {
            sampler_half.should_sample(
                Some(&parent_with_low_rv),
                trace_id_drop,
                "bench",
                &SpanKind::Internal,
                &[],
                &[],
            )
        });
    });

    group.bench_function("replacing-existing-th", |b| {
        b.iter(|| {
            sampler_half.should_sample(
                Some(&parent_with_existing_th),
                trace_id_sample,
                "bench",
                &SpanKind::Internal,
                &[],
                &[],
            )
        });
    });

    group.bench_function("sampling-probability-one", |b| {
        b.iter(|| {
            sampler_one.should_sample(
                None,
                trace_id_sample,
                "bench",
                &SpanKind::Internal,
                &[],
                &[],
            )
        });
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(std::time::Duration::from_millis(500))
        .measurement_time(std::time::Duration::from_millis(1000));
    targets = criterion_benchmark
}
criterion_main!(benches);
