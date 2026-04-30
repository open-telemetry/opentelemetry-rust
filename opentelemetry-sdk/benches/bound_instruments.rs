use criterion::{criterion_group, criterion_main, Criterion};
use opentelemetry::{metrics::MeterProvider as _, Key, KeyValue};
use opentelemetry_sdk::metrics::{Instrument, ManualReader, SdkMeterProvider, Stream, Temporality};

// Run this benchmark with:
// cargo bench --bench bound_instruments --features metrics,experimental_metrics_custom_reader,experimental_metrics_bound_instruments,spec_unstable_metrics_views
//
// Apple M4 Max, 16 cores (12 performance + 4 efficiency), macOS 15.4
//
// Results (3 attributes: method, status, path):
// Counter_Unbound_Delta              time:   [50.20 ns]
// Counter_Bound_Delta                time:   [ 1.80 ns]  ~28x faster
// Counter_Bound_With_View_Delta      time:   [ 1.82 ns]  view filter applied at bind, not on hot path
// Counter_Bound_AtOverflow_Delta     time:   [ 1.82 ns]  bind() at cardinality limit binds directly to the overflow
//                                                        tracker — perf parity with a normal bind, no per-call resolution
// Histogram_Unbound_Delta            time:   [58.64 ns]
// Histogram_Bound_Delta              time:   [ 6.50 ns]  ~9.0x faster
// Histogram_Bound_AtOverflow_Delta   time:   [ 6.58 ns]  perf parity with a normal bind
// Counter_Bound_Multithread/2        time:   [21.59 µs]  (100 adds/thread)
// Counter_Bound_Multithread/4        time:   [37.21 µs]  (100 adds/thread)
// Counter_Bound_Multithread/8        time:   [71.70 µs]  (100 adds/thread)
//
// Note: criterion does not fail CI on regression by itself. These numbers are
// reference values for human review; use `cargo criterion --baseline` locally
// if you need automated comparison against a saved baseline.

fn create_provider(temporality: Temporality) -> SdkMeterProvider {
    let reader = ManualReader::builder()
        .with_temporality(temporality)
        .build();
    SdkMeterProvider::builder().with_reader(reader).build()
}

fn bench_bound_instruments(c: &mut Criterion) {
    let mut group = c.benchmark_group("BoundInstruments");
    group.sample_size(100);

    let attrs = [
        KeyValue::new("method", "GET"),
        KeyValue::new("status", "200"),
        KeyValue::new("path", "/api/v1/users"),
    ];

    // Counter: Unbound vs Bound (Delta)
    {
        let provider = create_provider(Temporality::Delta);
        let meter = provider.meter("bench");
        let counter = meter.u64_counter("unbound").build();
        group.bench_function("Counter_Unbound_Delta", |b| {
            b.iter(|| counter.add(1, &attrs));
        });
    }

    {
        let provider = create_provider(Temporality::Delta);
        let meter = provider.meter("bench");
        let counter = meter.u64_counter("bound").build();
        let bound = counter.bind(&attrs);
        group.bench_function("Counter_Bound_Delta", |b| {
            b.iter(|| bound.add(1));
        });
    }

    // Counter: Bound with a View filter — confirms the filter is applied at
    // bind() time and the hot path stays free of attribute processing.
    {
        let view = |i: &opentelemetry_sdk::metrics::Instrument| {
            if i.name() == "bound_with_view" {
                Stream::builder()
                    .with_allowed_attribute_keys(vec![
                        Key::new("method"),
                        Key::new("status"),
                        Key::new("path"),
                    ])
                    .build()
                    .ok()
            } else {
                None
            }
        };
        let reader = ManualReader::builder()
            .with_temporality(Temporality::Delta)
            .build();
        let provider = SdkMeterProvider::builder()
            .with_reader(reader)
            .with_view(view)
            .build();
        let meter = provider.meter("bench");
        let counter = meter.u64_counter("bound_with_view").build();
        let bound = counter.bind(&attrs);
        group.bench_function("Counter_Bound_With_View_Delta", |b| {
            b.iter(|| bound.add(1));
        });
    }

    // Counter: Bound at overflow — confirms that binding when the cardinality
    // limit is exhausted yields the same hot-path performance as a normal bind
    // (writes go directly to the overflow tracker, no per-call resolution).
    {
        let cardinality_limit = 4;
        let view = move |i: &Instrument| {
            if i.name() == "bound_at_overflow" {
                Stream::builder()
                    .with_cardinality_limit(cardinality_limit)
                    .build()
                    .ok()
            } else {
                None
            }
        };
        let reader = ManualReader::builder()
            .with_temporality(Temporality::Delta)
            .build();
        let provider = SdkMeterProvider::builder()
            .with_reader(reader)
            .with_view(view)
            .build();
        let meter = provider.meter("bench");
        let counter = meter.u64_counter("bound_at_overflow").build();
        // Saturate cardinality with unbound calls so bind() lands in overflow.
        for i in 0..cardinality_limit {
            counter.add(1, &[KeyValue::new("filler", i as i64)]);
        }
        let bound = counter.bind(&attrs);
        group.bench_function("Counter_Bound_AtOverflow_Delta", |b| {
            b.iter(|| bound.add(1));
        });
    }

    // Histogram: Unbound vs Bound (Delta)
    {
        let provider = create_provider(Temporality::Delta);
        let meter = provider.meter("bench");
        let histogram = meter.f64_histogram("unbound_hist").build();
        group.bench_function("Histogram_Unbound_Delta", |b| {
            b.iter(|| histogram.record(1.5, &attrs));
        });
    }

    {
        let provider = create_provider(Temporality::Delta);
        let meter = provider.meter("bench");
        let histogram = meter.f64_histogram("bound_hist").build();
        let bound = histogram.bind(&attrs);
        group.bench_function("Histogram_Bound_Delta", |b| {
            b.iter(|| bound.record(1.5));
        });
    }

    // Histogram: Bound at overflow — same property as the counter version.
    {
        let cardinality_limit = 4;
        let view = move |i: &Instrument| {
            if i.name() == "bound_hist_at_overflow" {
                Stream::builder()
                    .with_cardinality_limit(cardinality_limit)
                    .build()
                    .ok()
            } else {
                None
            }
        };
        let reader = ManualReader::builder()
            .with_temporality(Temporality::Delta)
            .build();
        let provider = SdkMeterProvider::builder()
            .with_reader(reader)
            .with_view(view)
            .build();
        let meter = provider.meter("bench");
        let histogram = meter.f64_histogram("bound_hist_at_overflow").build();
        for i in 0..cardinality_limit {
            histogram.record(1.5, &[KeyValue::new("filler", i as i64)]);
        }
        let bound = histogram.bind(&attrs);
        group.bench_function("Histogram_Bound_AtOverflow_Delta", |b| {
            b.iter(|| bound.record(1.5));
        });
    }

    // Multi-threaded bound counter
    for num_threads in [2, 4, 8] {
        let provider = create_provider(Temporality::Delta);
        let meter = provider.meter("bench");
        let counter = meter.u64_counter("mt_bound").build();
        let bound = counter.bind(&attrs);

        group.bench_function(format!("Counter_Bound_Multithread/{num_threads}"), |b| {
            b.iter(|| {
                std::thread::scope(|s| {
                    for _ in 0..num_threads {
                        s.spawn(|| {
                            for _ in 0..100 {
                                bound.add(1);
                            }
                        });
                    }
                });
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_bound_instruments);
criterion_main!(benches);
