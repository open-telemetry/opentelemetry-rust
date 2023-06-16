//! run with `$ cargo run --example basic --all-features

use opentelemetry_api::{
    metrics::{MeterProvider as _, Unit},
    KeyValue,
};
use opentelemetry_sdk::metrics::exporter::PushMetricsExporter;
use opentelemetry_sdk::{
    metrics::{MeterProvider, PeriodicReader},
    runtime, Resource,
};
use opentelemetry_user_events_metrics::MetricsExporter;

use std::thread;
use std::time::Duration;

fn init_metrics(exporter: MetricsExporter) -> MeterProvider {
    let reader = PeriodicReader::builder(exporter, runtime::Tokio).build();
    MeterProvider::builder()
        .with_resource(Resource::new(vec![KeyValue::new(
            "service.name",
            "metric-demo",
        )]))
        .with_reader(reader)
        .build()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let exporter = opentelemetry_user_events_metrics::MetricsExporter::new();
    let meter_provider = init_metrics(exporter);

    let meter = meter_provider.versioned_meter(
        "user-event-test",
        Some("test-version"),
        Some("test_url"),
        Some(vec![KeyValue::new("key", "value")]),
    );
    let c = meter
        .f64_counter("counter_test")
        .with_description("test_decription")
        .with_unit(Unit::new("test_unit"))
        .init();

    c.add(
        1.0,
        [
            KeyValue::new("mykey1", "myvalue1"),
            KeyValue::new("mykey4", "myvalue4"),
        ]
        .as_ref(),
    );

    meter_provider.shutdown()?;
    // Call this explictly since provider shutdown is not propagated to exporter for now
    // Waiting on: https://github.com/open-telemetry/opentelemetry-rust/issues/1118
    #[allow(unused_must_use)]
    exporter.shutdown();

    thread::sleep(Duration::from_secs(5));

    Ok(())
}
