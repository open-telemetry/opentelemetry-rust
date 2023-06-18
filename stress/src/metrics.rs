use lazy_static::lazy_static;
use opentelemetry_api::{metrics::{Counter, MeterProvider as _}, KeyValue};
use opentelemetry_sdk::metrics::{ManualReader, MeterProvider};
use rand::Rng;
use std::borrow::Cow;

mod throughput;

lazy_static! {
    static ref PROVIDER: MeterProvider = MeterProvider::builder()
        .with_reader(ManualReader::builder().build())
        .build();
    static ref ATTRIBUTE_VALUES: [&'static str; 10] = ["value1", "value2", "value3", "value4", "value5", "value6", "value7", "value8", "value9", "value10"];

    static ref COUNTER: Counter<u64> = PROVIDER
        .meter(<&str as Into<Cow<'static, str>>>::into("test"))
        .u64_counter("hello")
        .init();
}

fn main() {
    throughput::test_throughput(test_counter);
}

fn test_counter() {
    let mut rng = rand::thread_rng();
    let len = ATTRIBUTE_VALUES.len();
    let index_first_attribute = rng.gen_range(0..len);
    let index_second_attribute = rng.gen_range(0..len);
    let index_third_attribute = rng.gen_range(0..len);
    
    // each attribute has 10 possible values, so there are 1000 possible combinations (time-series)
    COUNTER.add(1, &[
        KeyValue::new("attribute1", ATTRIBUTE_VALUES[index_first_attribute]),
        KeyValue::new("attribute2", ATTRIBUTE_VALUES[index_second_attribute]),
        KeyValue::new("attribute3", ATTRIBUTE_VALUES[index_third_attribute]),
    ]);
}
