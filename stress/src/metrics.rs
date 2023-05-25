use lazy_static::lazy_static;
use opentelemetry_api::{
    metrics::{Counter, MeterProvider as _},
    Context,
};
use opentelemetry_sdk::metrics::{ManualReader, MeterProvider};
use std::borrow::Cow;

mod throughput;

lazy_static! {
    static ref PROVIDER: MeterProvider = MeterProvider::builder()
        .with_reader(ManualReader::builder().build())
        .build();
    static ref COUNTER: Counter<u64> = PROVIDER
        .meter(<&str as Into<Cow<'static, str>>>::into("test"))
        .u64_counter("hello")
        .init();
}

fn main() {
    throughput::test_throughput(test_counter);
}

fn test_counter() {
    COUNTER.add(1, &[]);
}
