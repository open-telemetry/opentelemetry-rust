/*
    Stress test results:
    OS: Ubuntu 22.04.4 LTS (5.15.153.1-microsoft-standard-WSL2)
    Hardware: Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz, 16vCPUs,
    RAM: 64.0 GB
    ~1.9 M/sec
*/

use lazy_static::lazy_static;
use opentelemetry::{
    metrics::{Counter, MeterProvider as _},
    KeyValue,
};
use opentelemetry_sdk::metrics::{ManualReader, SdkMeterProvider};
use rand::{
    rngs::{self},
    Rng, SeedableRng,
};
use std::cell::RefCell;

mod throughput;

lazy_static! {
    static ref PROVIDER: SdkMeterProvider = SdkMeterProvider::builder()
        .with_reader(ManualReader::builder().build())
        .build();
    static ref COUNTER: Counter<u64> = PROVIDER.meter("test").u64_counter("hello").build();
}

thread_local! {
    /// Store random number generator for each thread
    static CURRENT_RNG: RefCell<rngs::SmallRng> = RefCell::new(rngs::SmallRng::from_entropy());
}

fn main() {
    throughput::test_throughput(test_counter);
}

fn test_counter() {
    // The main goal of this test is to ensure that OTel SDK is not growing its
    // memory usage indefinitely even when user code misbehaves by producing
    // unbounded metric points (unique time series).
    // It also checks that SDK's internal logging is also done in a bounded way.
    let rand = CURRENT_RNG.with(|rng| rng.borrow_mut().gen_range(0..100000000));
    COUNTER.add(1, &[KeyValue::new("A", rand)]);
}
