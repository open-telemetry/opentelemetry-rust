/*
// Run this benchmark with:
 // cargo bench --bench span-attributes --features experimental_span_attributes
 // The benchmark results:
 // Hardware: Intel(R) Core(TM) i7-7820HQ CPU @ 2.90GHz (4 cores)

| Test                      | Average time | Increment |
|---------------------------|--------------|-----------|
| span_4_attributes         | 538 ns       | -         |
| span_8_attributes         | 1.03 µs      | +492 ns   |
| nested_spans_1_levels     | 591 ns       | -         |
| nested_spans_2_levels     | 1.42 µs      | +829 ns   |
| nested_spans_3_levels     | 2.24 µs      | +820 ns   |

// Hardware: Mac M4 Pro (8 cores)
// Total Number of Cores:14 (10 performance and 4 efficiency)

| Test                      | Average time | Increment |
|---------------------------|--------------|-----------|
| span_4_attributes         | 293 ns       | -         |
| span_8_attributes         | 552 ns     | +277 ns   |
| nested_spans_1_levels     | 340 ns       | -         |
| nested_spans_2_levels     | 710 ns      | +1.08 µs   |
| nested_spans_3_levels     | 1000 ns      | +820 ns   |

*/

use criterion::{criterion_group, criterion_main, Criterion};
use opentelemetry::InstrumentationScope;
use opentelemetry_appender_tracing::layer as tracing_layer;
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::logs::{LogProcessor, SdkLogRecord, SdkLoggerProvider};
use opentelemetry_sdk::Resource;
#[cfg(all(not(target_os = "windows"), feature = "bench_profiling"))]
use pprof::criterion::{Output, PProfProfiler};
use tracing::info_span;
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

    let ot_layer = tracing_layer::OpenTelemetryTracingBridge::new(&provider);
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

    let ot_layer = tracing_layer::OpenTelemetryTracingBridge::new(&provider);
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

fn criterion_benchmark(c: &mut Criterion) {
    // Benchmark single spans with 4 and 8 attributes
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
