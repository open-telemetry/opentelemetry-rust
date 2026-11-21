/*
    Stress test for bound Counter instrument.

    Run with:
        cargo run --release --bin metrics_counter_bound \
            --features experimental_metrics_bound_instruments

    Stress test results:
    OS: macOS 26.4.1
    Hardware: Apple M4 Pro, 14 cores (10 performance + 4 efficiency)
    RAM: 24.0 GB
    ~13 B /sec
*/

use lazy_static::lazy_static;
use opentelemetry::{
    metrics::{BoundCounter, Counter, MeterProvider as _},
    KeyValue,
};
use opentelemetry_sdk::metrics::{ManualReader, SdkMeterProvider};

mod throughput;

lazy_static! {
    static ref PROVIDER: SdkMeterProvider = SdkMeterProvider::builder()
        .with_reader(ManualReader::builder().build())
        .build();
    static ref COUNTER: Counter<u64> = PROVIDER.meter("test").u64_counter("hello").build();
    // A single bound counter created once at startup. The hot path simply
    // calls add(1) on it, exercising the bound-instrument fast path with no
    // per-call attribute resolution.
    static ref BOUND_COUNTER: BoundCounter<u64> = COUNTER.bind(&[
        KeyValue::new("attribute1", "value1"),
        KeyValue::new("attribute2", "value2"),
        KeyValue::new("attribute3", "value3"),
    ]);
}

fn main() {
    throughput::test_throughput(test_counter_bound);
}

fn test_counter_bound() {
    BOUND_COUNTER.add(1);
}
