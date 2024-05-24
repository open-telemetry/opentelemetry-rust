/*
    Stress test results:
    OS: Ubuntu 22.04.3 LTS (5.15.146.1-microsoft-standard-WSL2)
    Hardware: AMD EPYC 7763 64-Core Processor - 2.44 GHz, 16vCPUs,
    RAM: 64.0 GB
    4.5M /sec
*/

use lazy_static::lazy_static;
use opentelemetry::{
    metrics::{Counter, MeterProvider as _},
    KeyValue,
};
use opentelemetry_sdk::metrics::{ManualReader, SdkMeterProvider};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use std::borrow::Cow;

mod throughput;

lazy_static! {
    static ref PROVIDER: SdkMeterProvider = SdkMeterProvider::builder()
        .with_reader(ManualReader::builder().build())
        .build();
    static ref ATTRIBUTE_VALUES: [&'static str; 10] = [
        "value1", "value2", "value3", "value4", "value5", "value6", "value7", "value8", "value9",
        "value10"
    ];
    static ref COUNTER: Counter<u64> = PROVIDER
        .meter(<&str as Into<Cow<'static, str>>>::into("test"))
        .u64_counter("hello")
        .init();
}

fn main() {
    for v in 0..2001 {
        COUNTER.add(100, &[KeyValue::new("A", v.to_string())]);
    }
    throughput::test_throughput(test_counter);
}

fn test_counter() {
    COUNTER.add(1, &[KeyValue::new("A", "2001")]);
}
