use lazy_static::lazy_static;
use opentelemetry::attributes::AttributeSet;
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
    static ref ATTRIBUTE_SETS: Vec<AttributeSet> = {
        let mut vec = Vec::new();
        for i0 in 0..ATTRIBUTE_VALUES.len() {
            for i1 in 0..ATTRIBUTE_VALUES.len() {
                for i2 in 0..ATTRIBUTE_VALUES.len() {
                    vec.push(AttributeSet::from(&[
                        KeyValue::new("attribute1", ATTRIBUTE_VALUES[i0]),
                        KeyValue::new("attribute2", ATTRIBUTE_VALUES[i1]),
                        KeyValue::new("attribute3", ATTRIBUTE_VALUES[i2]),
                    ]))
                }
            }
        }

        vec
    };
}

fn main() {
    throughput::test_throughput(test_counter);
}

fn test_counter() {
    let mut rng = SmallRng::from_entropy();
    let len = ATTRIBUTE_SETS.len();
    let index = rng.gen_range(0..len);

    COUNTER.add(1, ATTRIBUTE_SETS[index].clone());
}
