/*
    Stress test results:
    OS: Ubuntu 22.04.4 LTS (5.15.153.1-microsoft-standard-WSL2)
    Hardware: Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz, 16vCPUs,
    RAM: 64.0 GB
    ~11.5 M/sec
*/

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
use std::sync::LazyLock;

mod throughput;

static PROVIDER: LazyLock<SdkMeterProvider> = LazyLock::new(|| {
    SdkMeterProvider::builder()
        .with_reader(ManualReader::builder().build())
        .build()
});
static ATTRIBUTE_VALUES: LazyLock<[&str; 10]> = LazyLock::new(|| {
    [
        "value1", "value2", "value3", "value4", "value5", "value6", "value7", "value8", "value9",
        "value10",
    ]
});
static GAUGE: LazyLock<Gauge<u64>> =
    LazyLock::new(|| PROVIDER.meter("test").u64_gauge("test_gauge").build());

thread_local! {
    /// Store random number generator for each thread
    static CURRENT_RNG: RefCell<rngs::SmallRng> = RefCell::new(rngs::SmallRng::from_os_rng());
}

fn main() {
    throughput::test_throughput(test_gauge);
}

fn test_gauge() {
    let len = ATTRIBUTE_VALUES.len();
    let rands = CURRENT_RNG.with(|rng| {
        let mut rng = rng.borrow_mut();
        [
            rng.random_range(0..len),
            rng.random_range(0..len),
            rng.random_range(0..len),
        ]
    });
    let index_first_attribute = rands[0];
    let index_second_attribute = rands[1];
    let index_third_attribute = rands[2];

    // each attribute has 10 possible values, so there are 1000 possible combinations (time-series)
    GAUGE.record(
        1,
        &[
            KeyValue::new("attribute1", ATTRIBUTE_VALUES[index_first_attribute]),
            KeyValue::new("attribute2", ATTRIBUTE_VALUES[index_second_attribute]),
            KeyValue::new("attribute3", ATTRIBUTE_VALUES[index_third_attribute]),
        ],
    );
}
