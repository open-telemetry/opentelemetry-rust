/*
    Stress test for bound Histogram instrument.

    Run with:
        cargo run --release --bin metrics_histogram_bound \
            --features experimental_metrics_bound_instruments

    Stress test results:
    OS: macOS 26.4.1
    Hardware: Apple M4 Pro, 14 cores (10 performance + 4 efficiency)
    RAM: 24.0 GB
    ~11 B /sec
*/

use lazy_static::lazy_static;
use opentelemetry::{
    metrics::{BoundHistogram, Histogram, MeterProvider as _},
    KeyValue,
};
use opentelemetry_sdk::metrics::{ManualReader, SdkMeterProvider};

mod throughput;

lazy_static! {
    static ref PROVIDER: SdkMeterProvider = SdkMeterProvider::builder()
        .with_reader(ManualReader::builder().build())
        .build();
    static ref HISTOGRAM: Histogram<u64> = PROVIDER.meter("test").u64_histogram("hello").build();
    // A single bound histogram created once at startup. The hot path simply
    // calls record() on it, exercising the bound-instrument fast path with no
    // per-call attribute resolution.
    static ref BOUND_HISTOGRAM: BoundHistogram<u64> = HISTOGRAM.bind(&[
        KeyValue::new("attribute1", "value1"),
        KeyValue::new("attribute2", "value2"),
        KeyValue::new("attribute3", "value3"),
    ]);
}

fn main() {
    throughput::test_throughput(test_histogram_bound);
}

fn test_histogram_bound() {
    BOUND_HISTOGRAM.record(1);
}
