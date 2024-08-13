use opentelemetry::global;
use opentelemetry::KeyValue;
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::{runtime, Resource};
use std::error::Error;

fn init_meter_provider() -> opentelemetry_sdk::metrics::SdkMeterProvider {
    let exporter = opentelemetry_stdout::MetricsExporterBuilder::default()
        // uncomment the below lines to pretty print output.
        //  .with_encoder(|writer, data|
        //    Ok(serde_json::to_writer_pretty(writer, &data).unwrap()))
        .build();
    let reader = PeriodicReader::builder(exporter, runtime::Tokio).build();
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
            );

            observer.observe(
                200,
                &[
                    KeyValue::new("mykey3", "myvalue1"),
                    KeyValue::new("mykey4", "myvalue2"),
                ],
            );
        })
        .init();

    // Metrics are exported by default every 30 seconds when using stdout exporter,
    // however shutting down the MeterProvider here instantly flushes
    // the metrics, instead of waiting for the 30 sec interval.
    meter_provider.shutdown()?;
    Ok(())
}
