use opentelemetry::metrics::Unit;
use opentelemetry::Key;
use opentelemetry::{metrics::MeterProvider as _, KeyValue};
use opentelemetry_sdk::metrics::{Instrument, MeterProvider, PeriodicReader, Stream};
use opentelemetry_sdk::{runtime, Resource};
use std::error::Error;

fn init_meter_provider() -> MeterProvider {
    // for example 1
    let my_view_rename_and_unit = |i: &Instrument| {
        if i.name == "my_histogram" {
            Some(
                Stream::new()
                    .name("my_histogram_renamed")
                    .unit(Unit::new("milliseconds")),
            )
        } else {
            None
        }
    };

    // for example 2
    let my_view_drop_attributes = |i: &Instrument| {
        if i.name == "my_counter" {
            Some(Stream::new().allowed_attribute_keys(vec![Key::from("mykey1")]))
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
            "metrics-advanced-example",
        )]))
        .with_view(my_view_rename_and_unit)
        .with_view(my_view_drop_attributes)
        .build()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let meter_provider = init_meter_provider();
    let meter = meter_provider.meter("mylibraryname");

    // Example 1 - Rename metric using View.
    // This instrument will be renamed to "my_histogram_renamed",
    // and its unit changed to "milliseconds"
    // using view.
    let histogram = meter
        .f64_histogram("my_histogram")
        .with_unit(Unit::new("ms"))
        .with_description("My histogram example description")
        .init();

    // Record measurements using the histogram instrument.
    histogram.record(
        10.5,
        [
            KeyValue::new("mykey1", "myvalue1"),
            KeyValue::new("mykey2", "myvalue2"),
            KeyValue::new("mykey3", "myvalue3"),
            KeyValue::new("mykey4", "myvalue4"),
        ]
        .as_ref(),
    );

    // Example 2 - Drop unwanted attributes using view.
    let counter = meter.u64_counter("my_counter").init();

    // Record measurements using the Counter instrument.
    // Though we are passing 4 attributes here, only 1 will be used
    // for aggregation as view is configured to use only "mykey1"
    // attribute.
    counter.add(
        10,
        [
            KeyValue::new("mykey1", "myvalue1"),
            KeyValue::new("mykey2", "myvalue2"),
            KeyValue::new("mykey3", "myvalue3"),
            KeyValue::new("mykey4", "myvalue4"),
        ]
        .as_ref(),
    );

    // Metrics are exported by default every 30 seconds when using stdout exporter,
    // however shutting down the MeterProvider here instantly flushes
    // the metrics, instead of waiting for the 30 sec interval.
    meter_provider.shutdown()?;
    Ok(())
}
