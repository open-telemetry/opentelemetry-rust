use lazy_static::lazy_static;
use opentelemetry_api::{metrics::{Counter, MeterProvider as _, Histogram}, KeyValue, Value};
use opentelemetry_sdk::metrics::{ManualReader, MeterProvider};
use rand::Rng;
use std::borrow::Cow;

mod throughput;

lazy_static! {
    static ref PROVIDER: MeterProvider = MeterProvider::builder()
        .with_reader(ManualReader::builder().build())
        .build();
    static ref AttributeValues: [&'static str; 10] = ["value1", "value2", "value3", "value4", "value5", "value6", "value7", "value8", "value9", "value10"];

    static ref COUNTER: Counter<u64> = PROVIDER
        .meter(<&str as Into<Cow<'static, str>>>::into("test"))
        .u64_counter("hello")
        .init();

    static ref HISTOGRAM: Histogram<f64> = PROVIDER
        .meter(<&str as Into<Cow<'static, str>>>::into("test"))
        .f64_histogram("my_histogram")
        .init();
}

fn main() {
    throughput::test_throughput(test_counter);
}

fn test_counter() {
    let mut rng = rand::thread_rng();
    let index_first_attribute = rng.gen_range(0..AttributeValues.len());
    let index_second_attribute = rng.gen_range(0..AttributeValues.len());
    let index_third_attribute = rng.gen_range(0..AttributeValues.len());
    let index_fourth_attribute = rng.gen_range(0..AttributeValues.len());
    let index_fifth_attribute = rng.gen_range(0..AttributeValues.len());
    
    HISTOGRAM.record(1.0, &[
        KeyValue::new("attribute1", AttributeValues[index_first_attribute]),
        KeyValue::new("attribute2", AttributeValues[index_second_attribute]),
        // KeyValue::new("attribute3", AttributeValues[index_third_attribute]),
        // KeyValue::new("attribute4", AttributeValues[index_fourth_attribute]),
        // KeyValue::new("attribute5", AttributeValues[index_fifth_attribute]),
    ]);
}
