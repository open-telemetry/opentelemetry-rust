use lazy_static::lazy_static;
use opentelemetry_api::{
    metrics::{Counter, MeterProvider as _,},
    Context,
};
use opentelemetry_sdk::{
    metrics::{
        ManualReader, MeterProvider
    },
};

mod throughput;

lazy_static! {
    static ref CONTEXT: Context = Context::new();
    static ref PROVIDER: MeterProvider = MeterProvider::builder().with_reader(ManualReader::builder().build()).build();
    static ref COUNTER: Counter<u64> = PROVIDER.meter("test".into()).u64_counter("hello").init();
}

fn main() {
    throughput::test_throughput(test_counter);
}

fn test_counter() {
    COUNTER.add(&CONTEXT, 1, &[]);
}
