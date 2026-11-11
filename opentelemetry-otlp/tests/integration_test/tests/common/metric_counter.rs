use opentelemetry::metrics::{Counter, MeterProvider};
use opentelemetry_sdk::metrics::SdkMeterProvider;

pub fn test_counter(provider: &SdkMeterProvider) -> Counter<u64> {
    provider
        .meter("test_meter")
        .u64_counter("test_counter")
        .build()
}
