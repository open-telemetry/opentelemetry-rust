use criterion::{criterion_group, criterion_main, Criterion};
use opentelemetry::attributes::AttributeSet;
use opentelemetry::{
    metrics::{Counter, MeterProvider as _},
    KeyValue,
};
use opentelemetry_sdk::metrics::{ManualReader, SdkMeterProvider};
use rand::{rngs::SmallRng, Rng, SeedableRng};

// Run this benchmark with:
// cargo bench --bench metric_counter --features=metrics
fn create_counter() -> Counter<u64> {
    let meter_provider: SdkMeterProvider = SdkMeterProvider::builder()
        .with_reader(ManualReader::builder().build())
        .build();
    let meter = meter_provider.meter("benchmarks");
    let counter = meter.u64_counter("counter_bench").init();
    counter
}

fn criterion_benchmark(c: &mut Criterion) {
    counter_add(c);
}

fn counter_add(c: &mut Criterion) {
    let attribute_values = [
        "value1", "value2", "value3", "value4", "value5", "value6", "value7", "value8", "value9",
        "value10",
    ];

    let counter = create_counter();
    c.bench_function("Counter_Add_Sorted", |b| {
        b.iter(|| {
            let mut rng = SmallRng::from_entropy();
            // 4*4*10*10 = 1600 time series.
            let index_first_attribute = rng.gen_range(0..4);
            let index_second_attribute = rng.gen_range(0..4);
            let index_third_attribute = rng.gen_range(0..10);
            let index_forth_attribute = rng.gen_range(0..10);
            counter.add(
                1,
                [
                    KeyValue::new("attribute1", attribute_values[index_first_attribute]),
                    KeyValue::new("attribute2", attribute_values[index_second_attribute]),
                    KeyValue::new("attribute3", attribute_values[index_third_attribute]),
                    KeyValue::new("attribute4", attribute_values[index_forth_attribute]),
                ],
            );
        });
    });

    c.bench_function("Counter_Add_Unsorted", |b| {
        b.iter(|| {
            let mut rng = SmallRng::from_entropy();
            // 4*4*10*10 = 1600 time series.
            let index_first_attribute = rng.gen_range(0..4);
            let index_second_attribute = rng.gen_range(0..4);
            let index_third_attribute = rng.gen_range(0..10);
            let index_forth_attribute = rng.gen_range(0..10);
            counter.add(
                1,
                [
                    KeyValue::new("attribute2", attribute_values[index_second_attribute]),
                    KeyValue::new("attribute3", attribute_values[index_third_attribute]),
                    KeyValue::new("attribute1", attribute_values[index_first_attribute]),
                    KeyValue::new("attribute4", attribute_values[index_forth_attribute]),
                ],
            );
        });
    });

    c.bench_function("Counter_Add_Cached_Attributes", |b| {
        let attributes = AttributeSet::from([
            KeyValue::new("attribute2", attribute_values[0]),
            KeyValue::new("attribute3", attribute_values[1]),
            KeyValue::new("attribute1", attribute_values[2]),
            KeyValue::new("attribute4", attribute_values[3]),
        ]);

        b.iter(|| {
            counter.add(1, attributes.clone());
        });
    });
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
