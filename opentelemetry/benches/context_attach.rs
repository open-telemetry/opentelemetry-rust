use criterion::{
    black_box, criterion_group, criterion_main, measurement::WallTime, BenchmarkGroup, BenchmarkId,
    Criterion, Throughput,
};
use opentelemetry::{
    trace::{SpanContext, TraceContextExt},
    Context,
};

// Run this benchmark with:
// cargo bench --bench current_context

fn criterion_benchmark(c: &mut Criterion) {
    let span_context = Context::new().with_remote_span_context(SpanContext::empty_context());
    let contexts = vec![
        ("empty_cx", Context::new()),
        ("single_value_cx", Context::new().with_value(Value(4711))),
        ("span_cx", span_context),
    ];
    for (name, cx) in contexts {
        single_cx_scope(&mut group(c), name, &cx);
        nested_cx_scope(&mut group(c), name, &cx);
        overlapping_cx_scope(&mut group(c), name, &cx);
    }
}

fn single_cx_scope(
    group: &mut BenchmarkGroup<'_, WallTime>,
    context_type: &str,
    context: &Context,
) {
    let _restore = Context::current().attach();
    group.throughput(Throughput::Elements(1)).bench_function(
        BenchmarkId::new("single_cx_scope", context_type),
        |b| {
            b.iter_batched(
                || context.clone(),
                |cx| {
                    single_cx(cx);
                },
                criterion::BatchSize::SmallInput,
            );
        },
    );
}

#[inline(never)]
fn single_cx(cx: Context) {
    let cx = black_box(cx.attach());
    let _ = black_box(dummy_work());
    drop(cx);
}

fn nested_cx_scope(group: &mut BenchmarkGroup<'_, WallTime>, cx_type: &str, context: &Context) {
    let _restore = Context::current().attach();
    group.throughput(Throughput::Elements(1)).bench_function(
        BenchmarkId::new("nested_cx_scope", cx_type),
        |b| {
            b.iter_batched(
                || (context.clone(), context.clone()),
                |(cx1, cx2)| {
                    nested_cx(cx1, cx2);
                },
                criterion::BatchSize::SmallInput,
            );
        },
    );
}

#[inline(never)]
fn nested_cx(cx1: Context, cx2: Context) {
    let outer = black_box(cx1.attach());
    let inner = black_box(cx2.attach());
    let _ = black_box(dummy_work());
    drop(inner);
    drop(outer);
}

fn overlapping_cx_scope(
    group: &mut BenchmarkGroup<'_, WallTime>,
    cx_type: &str,
    context: &Context,
) {
    let _restore = Context::current().attach();
    group.throughput(Throughput::Elements(1)).bench_function(
        BenchmarkId::new("overlapping_cx_scope", cx_type),
        |b| {
            b.iter_batched(
                || (context.clone(), context.clone()),
                |(cx1, cx2)| {
                    overlapping_cx(cx1, cx2);
                },
                criterion::BatchSize::SmallInput,
            );
        },
    );
}

#[inline(never)]
fn overlapping_cx(cx1: Context, cx2: Context) {
    let outer = cx1.attach();
    let inner = cx2.attach();
    let _ = black_box(dummy_work());
    drop(outer);
    drop(inner);
}

#[inline(never)]
fn dummy_work() -> i32 {
    black_box(1 + 1)
}

fn group(c: &mut Criterion) -> BenchmarkGroup<WallTime> {
    c.benchmark_group("context_attach")
}

#[derive(Debug, PartialEq)]
struct Value(i32);

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
