#[cfg(feature = "metrics")]
#[test]
fn default_meter_provider_builds() {
    let provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder().build();
    drop(provider);
}
