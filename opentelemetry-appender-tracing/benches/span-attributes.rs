/*
// Run this benchmark with:
// cargo bench --bench span-attributes
// The benchmark results:
// Hardware: Mac M4 Pro
// Total Number of Cores: 14 (10 performance and 4 efficiency)
// rustc 1.95.0 (59807616e 2026-04-14)
// cargo 1.95.0 (f2d3ce0bd 2026-03-21)
//
// Attribute counts are chosen to stay within the SDK's inline-array optimisation
// threshold (5 attributes). With tracing-span attribute enrichment enabled (via
// `OpenTelemetryTracingBridgeBuilder::with_tracing_span_attributes`), tracing-span
// attributes are copied onto the log record, so total = log attrs + span attrs.
//
// Log + span benchmarks (showing incremental cost of span attributes on logging):
// | Test                                      | Total attrs | Average time | Increment vs baseline        |
// |-------------------------------------------|-------------|--------------|------------------------------|
// | log_1_attr_no_span                        | 1           |  85 ns       | -                            |
// | log_1_attr_in_span_2_attr                 | 3           | 382 ns       | +297 ns                      |
// | log_1_attr_in_nested_spans_2plus2_attr    | 5           | 590 ns       | +505 ns (+208 ns vs 1 span)  |
//
// Span-only benchmarks (no log emission, kept for reference):
// | Test                  | Average time | Increment |
// |-----------------------|--------------|-----------|
// | span_4_attributes     | 194 ns       | -         |
// | span_8_attributes     | 314 ns       | +120 ns   |
// | nested_spans_1_levels | 194 ns       | -         |
// | nested_spans_2_levels | 413 ns       | +219 ns   |
// | nested_spans_3_levels | 630 ns       | +217 ns   |

*/

use criterion::{criterion_group, criterion_main, Criterion};
use opentelemetry::InstrumentationScope;
use opentelemetry_appender_tracing::layer as tracing_layer;
use opentelemetry_appender_tracing::layer::TracingSpanAttributes;
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::logs::{LogProcessor, SdkLogRecord, SdkLoggerProvider};
use opentelemetry_sdk::Resource;
#[cfg(all(not(target_os = "windows"), feature = "bench_profiling"))]
use pprof::criterion::{Output, PProfProfiler};
use tracing::{error, info_span};
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;

#[derive(Debug)]
struct NoopProcessor;

impl LogProcessor for NoopProcessor {
    fn emit(&self, _: &mut SdkLogRecord, _: &InstrumentationScope) {}

    fn force_flush(&self) -> OTelSdkResult {
        Ok(())
    }

    fn shutdown_with_timeout(&self, _timeout: std::time::Duration) -> OTelSdkResult {
        Ok(())
    }
}

/// Creates a benchmark for a specific number of attributes
fn benchmark_span_attributes(c: &mut Criterion, num_attributes: usize) {
    let provider = SdkLoggerProvider::builder()
        .with_resource(
            Resource::builder_empty()
                .with_service_name("benchmark")
                .build(),
        )
        .with_log_processor(NoopProcessor)
        .build();

    let ot_layer = tracing_layer::OpenTelemetryTracingBridge::builder(&provider)
        .with_tracing_span_attributes(TracingSpanAttributes::all())
        .build();
    let subscriber = Registry::default().with(ot_layer);

    tracing::subscriber::with_default(subscriber, || {
        c.bench_function(&format!("span_{num_attributes}_attributes"), |b| {
            b.iter(|| match num_attributes {
                4 => {
                    let span = info_span!(
                        "test",
                        attr1 = "value1",
                        attr2 = "value2",
                        attr3 = "value3",
                        attr4 = "value4"
                    );
                    let _enter = span.enter();
                }
                8 => {
                    let span = info_span!(
                        "test",
                        attr1 = "value1",
                        attr2 = "value2",
                        attr3 = "value3",
                        attr4 = "value4",
                        attr5 = "value5",
                        attr6 = "value6",
                        attr7 = "value7",
                        attr8 = "value8"
                    );
                    let _enter = span.enter();
                }
                _ => {
                    // Fall back to 8 attributes for any other number
                    let span = info_span!(
                        "test",
                        attr1 = "value1",
                        attr2 = "value2",
                        attr3 = "value3",
                        attr4 = "value4",
                        attr5 = "value5",
                        attr6 = "value6",
                        attr7 = "value7",
                        attr8 = "value8"
                    );
                    let _enter = span.enter();
                }
            });
        });
    });
}

/// Creates a benchmark for nested spans with a specific depth
/// Each span has 4 attributes
fn benchmark_nested_spans(c: &mut Criterion, depth: usize) {
    let provider = SdkLoggerProvider::builder()
        .with_resource(
            Resource::builder_empty()
                .with_service_name("benchmark")
                .build(),
        )
        .with_log_processor(NoopProcessor)
        .build();

    let ot_layer = tracing_layer::OpenTelemetryTracingBridge::builder(&provider)
        .with_tracing_span_attributes(TracingSpanAttributes::all())
        .build();
    let subscriber = Registry::default().with(ot_layer);

    tracing::subscriber::with_default(subscriber, || {
        c.bench_function(&format!("nested_spans_{depth}_levels"), |b| {
            b.iter(|| match depth {
                1 => {
                    let span1 = info_span!(
                        "level_1",
                        depth = 1,
                        attr1 = "value1",
                        attr2 = "value2",
                        attr3 = "value3",
                        attr4 = "value4"
                    );
                    let _enter1 = span1.enter();
                }
                2 => {
                    let span1 = info_span!(
                        "level_1",
                        depth = 1,
                        attr1 = "value1",
                        attr2 = "value2",
                        attr3 = "value3",
                        attr4 = "value4"
                    );
                    let _enter1 = span1.enter();
                    {
                        let span2 = info_span!(
                            "level_2",
                            depth = 2,
                            attr1 = "value1",
                            attr2 = "value2",
                            attr3 = "value3",
                            attr4 = "value4"
                        );
                        let _enter2 = span2.enter();
                    }
                }
                3 => {
                    let span1 = info_span!(
                        "level_1",
                        depth = 1,
                        attr1 = "value1",
                        attr2 = "value2",
                        attr3 = "value3",
                        attr4 = "value4"
                    );
                    let _enter1 = span1.enter();
                    {
                        let span2 = info_span!(
                            "level_2",
                            depth = 2,
                            attr1 = "value1",
                            attr2 = "value2",
                            attr3 = "value3",
                            attr4 = "value4"
                        );
                        let _enter2 = span2.enter();
                        {
                            let span3 = info_span!(
                                "level_3",
                                depth = 3,
                                attr1 = "value1",
                                attr2 = "value2",
                                attr3 = "value3",
                                attr4 = "value4"
                            );
                            let _enter3 = span3.enter();
                        }
                    }
                }
                _ => {
                    // Fall back to depth 2 for any higher number
                    let span1 = info_span!(
                        "level_1",
                        depth = 1,
                        attr1 = "value1",
                        attr2 = "value2",
                        attr3 = "value3",
                        attr4 = "value4"
                    );
                    let _enter1 = span1.enter();
                    {
                        let span2 = info_span!(
                            "level_2",
                            depth = 2,
                            attr1 = "value1",
                            attr2 = "value2",
                            attr3 = "value3",
                            attr4 = "value4"
                        );
                        let _enter2 = span2.enter();
                    }
                }
            });
        });
    });
}

fn make_provider() -> SdkLoggerProvider {
    SdkLoggerProvider::builder()
        .with_resource(
            Resource::builder_empty()
                .with_service_name("benchmark")
                .build(),
        )
        .with_log_processor(NoopProcessor)
        .build()
}

/// Baseline: emit an error log with 1 attribute, no enclosing span (total = 1 attr).
/// This is the reference point for measuring what span attributes add to logging cost.
/// Kept at 1 attribute so all three log+span benchmarks stay within the SDK's
/// 5-attribute inline-array optimisation threshold.
fn benchmark_log_1_attr_no_span(c: &mut Criterion) {
    let provider = make_provider();
    let ot_layer = tracing_layer::OpenTelemetryTracingBridge::builder(&provider)
        .with_tracing_span_attributes(TracingSpanAttributes::all())
        .build();
    let subscriber = Registry::default().with(ot_layer);

    tracing::subscriber::with_default(subscriber, || {
        c.bench_function("log_1_attr_no_span", |b| {
            b.iter(|| {
                error!(
                    name: "CheckoutFailed",
                    attr1 = "value1",
                    message = "Unable to process checkout."
                );
            });
        });
    });
}

/// Emit an error log with 1 attribute while inside a span that carries 2 attributes
/// (total = 3 attrs on the log record, within the SDK's 5-attr inline threshold).
/// With tracing-span attribute enrichment enabled, the 2 span attributes are copied onto the
/// log record, so the delta vs `log_1_attr_no_span` shows the pure span-attribute cost.
fn benchmark_log_1_attr_in_span_2_attr(c: &mut Criterion) {
    let provider = make_provider();
    let ot_layer = tracing_layer::OpenTelemetryTracingBridge::builder(&provider)
        .with_tracing_span_attributes(TracingSpanAttributes::all())
        .build();
    let subscriber = Registry::default().with(ot_layer);

    tracing::subscriber::with_default(subscriber, || {
        c.bench_function("log_1_attr_in_span_2_attr", |b| {
            b.iter(|| {
                let span = info_span!("checkout", span_attr1 = "svalue1", span_attr2 = "svalue2");
                let _enter = span.enter();
                error!(
                    name: "CheckoutFailed",
                    attr1 = "value1",
                    message = "Unable to process checkout."
                );
            });
        });
    });
}

/// Emit an error log with 1 attribute while inside two nested spans, the outer carrying
/// 2 attributes and the inner carrying 2 attributes (1 log + 2 + 2 = 5 total, hitting
/// the SDK's inline-array limit without exceeding it).
/// Compared to `log_1_attr_in_span_2_attr` this isolates the cost of nesting (traversing
/// two spans) vs the same total number of span attributes on a single span.
fn benchmark_log_1_attr_in_nested_spans_2plus2_attr(c: &mut Criterion) {
    let provider = make_provider();
    let ot_layer = tracing_layer::OpenTelemetryTracingBridge::builder(&provider)
        .with_tracing_span_attributes(TracingSpanAttributes::all())
        .build();
    let subscriber = Registry::default().with(ot_layer);

    tracing::subscriber::with_default(subscriber, || {
        c.bench_function("log_1_attr_in_nested_spans_2plus2_attr", |b| {
            b.iter(|| {
                let outer = info_span!("outer", span_attr1 = "svalue1", span_attr2 = "svalue2");
                let _enter_outer = outer.enter();
                let inner = info_span!("inner", span_attr3 = "svalue3", span_attr4 = "svalue4");
                let _enter_inner = inner.enter();
                error!(
                    name: "CheckoutFailed",
                    attr1 = "value1",
                    message = "Unable to process checkout."
                );
            });
        });
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    // --- Log + span benchmarks ---
    // These show the incremental cost that span attributes add to log emission.
    // Attribute counts are kept within the SDK's 5-attr inline threshold so that
    // heap allocation doesn't skew the comparison between the three cases.
    benchmark_log_1_attr_no_span(c); // baseline: 1 log attr, no span  (total=1)
    benchmark_log_1_attr_in_span_2_attr(c); // 1 log attr + 1 span (2 attrs)   (total=3)
    benchmark_log_1_attr_in_nested_spans_2plus2_attr(c); // 1 log attr + 2 nested spans (2+2 attrs) (total=5)

    // --- Span-only benchmarks (no log emission) ---
    // Kept for reference: cost of creating spans themselves.
    benchmark_span_attributes(c, 4);
    benchmark_span_attributes(c, 8);

    // Benchmark nested spans (1-3 levels deep, each with 4 attributes)
    for i in 1..=3 {
        benchmark_nested_spans(c, i);
    }
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
