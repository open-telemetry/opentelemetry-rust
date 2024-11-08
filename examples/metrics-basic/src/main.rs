use opentelemetry::global;
use opentelemetry::KeyValue;
use opentelemetry_sdk::metrics::data::Temporality;
use opentelemetry_sdk::metrics::PeriodicReaderWithOwnThread;
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::{runtime, Resource};
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;
use std::error::Error;
use std::time::Duration;

fn init_meter_provider() -> opentelemetry_sdk::metrics::SdkMeterProvider {
    let exporter = opentelemetry_stdout::MetricExporterBuilder::default()
        // Build exporter using Delta Temporality (Defaults to Temporality::Cumulative)
        .with_temporality(Temporality::Delta)
        .build();
    let reader = PeriodicReaderWithOwnThread::builder(exporter)
        .with_interval(Duration::from_secs(10))
        .build();
    let provider = SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(Resource::new([KeyValue::new(
            "service.name",
            "metrics-basic-example",
        )]))
        .build();
    global::set_meter_provider(provider.clone());
    provider
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
tracing_subscriber::registry()
    .with(fmt::layer().with_thread_names(true))
    .init();

    // Initialize the MeterProvider with the stdout Exporter.
    let meter_provider = init_meter_provider();

    // Create a meter from the above MeterProvider.
    let meter = global::meter("mylibraryname");

    // Create a ObservableCounter instrument and register a callback that reports the measurement.
    let _observable_counter = meter
        .u64_observable_counter("my_observable_counter")
        .with_description("My observable counter example description")
        .with_unit("myunit")
        .with_callback(|observer| {
            observer.observe(
                100,
                &[
                    KeyValue::new("mykey1", "myvalue1"),
                    KeyValue::new("mykey2", "myvalue2"),
                ],
            )
        })
        .build();

    // Create a Counter Instrument.
    let counter = meter.u64_counter("my_counter").build();

    loop {
        // Record measurements using the Counter instrument.
        counter.add(
            10,
            &[
                KeyValue::new("mykey1", "myvalue1"),
                KeyValue::new("mykey2", "myvalue2"),
            ],
        );
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    // Metrics are exported by default every 30 seconds when using stdout exporter,
    // however shutting down the MeterProvider here instantly flushes
    // the metrics, instead of waiting for the 30 sec interval.
    meter_provider.shutdown()?;
    Ok(())
}
