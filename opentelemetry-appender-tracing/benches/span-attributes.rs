/*
// Run this benchmark with:
 // cargo bench --bench span-attributes --features experimental_span_attributes
 // The benchmark results:
 // Hardware: Intel(R) Core(TM) i7-7820HQ CPU @ 2.90GHz (4 cores)

| Test                      | Average time | Increment |
|---------------------------|--------------|-----------|
| span_0_attributes         | 146 ns       | -         |
| span_1_attributes         | 296 ns       | +150 ns   |
| span_2_attributes         | 370 ns       | +74 ns    |
| span_3_attributes         | 439 ns       | +69 ns    |
| span_4_attributes         | 587 ns       | +148 ns   |
| span_5_attributes         | 681 ns       | +94 ns    |
| span_6_attributes         | 714 ns       | +33 ns    |
| span_7_attributes         | 890 ns       | +176 ns   |
| span_8_attributes         | 1.18 µs      | +290 ns   |
| span_9_attributes         | 1.24 µs      | +60 ns    |
| span_10_attributes        | 1.34 µs      | +100 ns   |
| span_11_attributes        | 1.39 µs      | +50 ns    |
| span_12_attributes        | 1.48 µs      | +90 ns    |

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

/// Creates a benchmark for a single span with a specific number of attributes
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
            // Create span with the specified number of attributes
            b.iter(|| match num_attributes {
                0 => {
                    let span = info_span!("test");
                    let _enter = span.enter();
                }
                1 => {
                    let span = info_span!("test", attr1 = "value1");
                    let _enter = span.enter();
                }
                2 => {
                    let span = info_span!("test", attr1 = "value1", attr2 = "value2");
                    let _enter = span.enter();
                }
                3 => {
                    let span =
                        info_span!("test", attr1 = "value1", attr2 = "value2", attr3 = "value3");
                    let _enter = span.enter();
                }
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
                5 => {
                    let span = info_span!(
                        "test",
                        attr1 = "value1",
                        attr2 = "value2",
                        attr3 = "value3",
                        attr4 = "value4",
                        attr5 = "value5"
                    );
                    let _enter = span.enter();
                }
                6 => {
                    let span = info_span!(
                        "test",
                        attr1 = "value1",
                        attr2 = "value2",
                        attr3 = "value3",
                        attr4 = "value4",
                        attr5 = "value5",
                        attr6 = "value6"
                    );
                    let _enter = span.enter();
                }
                7 => {
                    let span = info_span!(
                        "test",
                        attr1 = "value1",
                        attr2 = "value2",
                        attr3 = "value3",
                        attr4 = "value4",
                        attr5 = "value5",
                        attr6 = "value6",
                        attr7 = "value7"
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
                9 => {
                    let span = info_span!(
                        "test",
                        attr1 = "value1",
                        attr2 = "value2",
                        attr3 = "value3",
                        attr4 = "value4",
                        attr5 = "value5",
                        attr6 = "value6",
                        attr7 = "value7",
                        attr8 = "value8",
                        attr9 = "value9"
                    );
                    let _enter = span.enter();
                }
                10 => {
                    let span = info_span!(
                        "test",
                        attr1 = "value1",
                        attr2 = "value2",
                        attr3 = "value3",
                        attr4 = "value4",
                        attr5 = "value5",
                        attr6 = "value6",
                        attr7 = "value7",
                        attr8 = "value8",
                        attr9 = "value9",
                        attr10 = "value10"
                    );
                    let _enter = span.enter();
                }
                11 => {
                    let span = info_span!(
                        "test",
                        attr1 = "value1",
                        attr2 = "value2",
                        attr3 = "value3",
                        attr4 = "value4",
                        attr5 = "value5",
                        attr6 = "value6",
                        attr7 = "value7",
                        attr8 = "value8",
                        attr9 = "value9",
                        attr10 = "value10",
                        attr11 = "value11"
                    );
                    let _enter = span.enter();
                }
                12 => {
                    let span = info_span!(
                        "test",
                        attr1 = "value1",
                        attr2 = "value2",
                        attr3 = "value3",
                        attr4 = "value4",
                        attr5 = "value5",
                        attr6 = "value6",
                        attr7 = "value7",
                        attr8 = "value8",
                        attr9 = "value9",
                        attr10 = "value10",
                        attr11 = "value11",
                        attr12 = "value12"
                    );
                    let _enter = span.enter();
                }
                _ => {
                    // Fall back to 8 attributes for any higher number
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

fn criterion_benchmark(c: &mut Criterion) {
    benchmark_span_attributes(c, 4);
    // Run benchmarks for 0 to 12 attributes
    // for i in 0..=12 {
    //     benchmark_span_attributes(c, i);
    // }
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
