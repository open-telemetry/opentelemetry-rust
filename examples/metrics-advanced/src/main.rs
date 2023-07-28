use opentelemetry_api::metrics::Unit;
use opentelemetry_api::{metrics::MeterProvider as _, KeyValue};
use opentelemetry_sdk::metrics::{MeterProvider, PeriodicReader, new_view, Instrument, Stream};
use opentelemetry_sdk::{runtime, Resource};
use std::error::Error;

fn init_meter_provider() -> MeterProvider {
    let my_view = |i: &Instrument| {
        if i.name == "my_histogram" {
            Some(
                Stream::new().name("my_histogram_renamed")
            )
        } else {
            None
        }        
    };

    let exporter = opentelemetry_stdout::MetricsExporter::default();
    let reader = PeriodicReader::builder(exporter, runtime::Tokio).build();
    MeterProvider::builder()
        .with_reader(reader)
        .with_resource(Resource::new(vec![KeyValue::new(
            "service.name",
            "metrics-basic-example",
        )]))
        .with_view(my_view)
        .build()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    // Initialize the MeterProvider with the stdout Exporter.
    let meter_provider = init_meter_provider();

    // Create a meter from the above MeterProvider.
    let meter = meter_provider.meter("mylibraryname");

    // Create a Histogram Instrument.
    let histogram = meter
        .f64_histogram("my_histogram")
        .with_description("My histogram example description")
        .init();

    // Record measurements using the histogram instrument.
    histogram.record(
        10.5,
        [
            KeyValue::new("mykey1", "myvalue1"),
            KeyValue::new("mykey2", "myvalue2"),
        ]
        .as_ref(),
    );

    

    // Metrics are exported by default every 30 seconds when using stdout exporter,
    // however shutting down the MeterProvider here instantly flushes
    // the metrics, instead of waiting for the 30 sec interval.
    meter_provider.shutdown()?;
    Ok(())
}
