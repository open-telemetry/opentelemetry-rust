use criterion::{criterion_group, criterion_main, Criterion};
use opentelemetry::{metrics::MeterProvider as _, KeyValue};
use opentelemetry_sdk::metrics::{ManualReader, SdkMeterProvider, Temporality};

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
