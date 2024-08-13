/*
    The benchmark results:
    criterion = "0.5.1"
    OS: Ubuntu 22.04.4 LTS (5.15.153.1-microsoft-standard-WSL2)
    Hardware: Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz, 16vCPUs,
    RAM: 64.0 GB
    | Test                           | Average time|
    |--------------------------------|-------------|
    | Gauge_Add                      | 178.37 ns   |
*/

use criterion::{criterion_group, criterion_main, Criterion};
use opentelemetry::{
    metrics::{Gauge, MeterProvider as _},
    KeyValue,
};
use opentelemetry_sdk::metrics::{ManualReader, SdkMeterProvider};
use rand::{
    rngs::{self},
    Rng, SeedableRng,
};
use std::cell::RefCell;

thread_local! {
    /// Store random number generator for each thread
    static CURRENT_RNG: RefCell<rngs::SmallRng> = RefCell::new(rngs::SmallRng::from_entropy());
}

static ATTRIBUTE_VALUES: [&str; 10] = [
    "value1", "value2", "value3", "value4", "value5", "value6", "value7", "value8", "value9",
    "value10",
];

// Run this benchmark with:
// cargo bench --bench metric_gauge
fn create_gauge() -> Gauge<u64> {
    let meter_provider: SdkMeterProvider = SdkMeterProvider::builder()
        .with_reader(ManualReader::builder().build())
        .build();
    let meter = meter_provider.meter("benchmarks");

    meter.u64_gauge("gauge_bench").init()
}

fn criterion_benchmark(c: &mut Criterion) {
    gauge_record(c);
}

fn gauge_record(c: &mut Criterion) {
    let gauge = create_gauge();
    c.bench_function("Gauge_Add", |b| {
        b.iter(|| {
            // 4*4*10*10 = 1600 time series.
            let rands = CURRENT_RNG.with(|rng| {
                let mut rng = rng.borrow_mut();
                [
                    rng.gen_range(0..4),
                    rng.gen_range(0..4),
                    rng.gen_range(0..10),
                    rng.gen_range(0..10),
                ]
            });
            let index_first_attribute = rands[0];
            let index_second_attribute = rands[1];
            let index_third_attribute = rands[2];
            let index_fourth_attribute = rands[3];
            gauge.record(
                1,
                &[
                    KeyValue::new("attribute1", ATTRIBUTE_VALUES[index_first_attribute]),
                    KeyValue::new("attribute2", ATTRIBUTE_VALUES[index_second_attribute]),
                    KeyValue::new("attribute3", ATTRIBUTE_VALUES[index_third_attribute]),
                    KeyValue::new("attribute4", ATTRIBUTE_VALUES[index_fourth_attribute]),
                ],
            );
        });
    });
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
