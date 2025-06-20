use opentelemetry::global;
use opentelemetry::KeyValue;
use opentelemetry_sdk::metrics::{Instrument, SdkMeterProvider, Stream, Temporality};
use opentelemetry_sdk::Resource;
use std::error::Error;

fn init_meter_provider() -> opentelemetry_sdk::metrics::SdkMeterProvider {
    // for example 1
    let my_view_rename_and_unit = |i: &Instrument| {
        if i.name() == "my_histogram" {
            Some(
                Stream::builder()
                    .with_name("my_histogram_renamed")
                    .with_unit("milliseconds")
                    .build()
                    .unwrap(),
            )
        } else {
            None
        }
    };

    // for example 2
    let my_view_change_cardinality = |i: &Instrument| {
        if i.name() == "my_second_histogram" {
            // Note: If Stream is invalid, build() will return an error. By
            // calling `.ok()`, any such error is ignored and treated as if the
            // view does not match the instrument. If this is not the desired
            // behavior, consider handling the error explicitly.
            Stream::builder().with_cardinality_limit(2).build().ok()
        } else {
            None
        }
    };

    // Build exporter using Delta Temporality.
    let exporter = opentelemetry_stdout::MetricExporterBuilder::default()
        .with_temporality(Temporality::Delta)
        .build();

    let resource = Resource::builder()
        .with_service_name("metrics-advanced-example")
        .build();

    let provider = SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .with_resource(resource)
        .with_view(my_view_rename_and_unit)
        .with_view(my_view_change_cardinality)
        .build();
    global::set_meter_provider(provider.clone());
    provider
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let meter_provider = init_meter_provider();
    let meter = global::meter("mylibraryname");

    // Example 1 - Rename metric using View.
    // This instrument will be renamed to "my_histogram_renamed",
    // and its unit changed to "milliseconds"
    // using view.
    let histogram = meter
        .f64_histogram("my_histogram")
        .with_unit("ms")
        .with_description("My histogram example description")
        .build();

    // Record measurements using the histogram instrument.
    histogram.record(
        10.5,
        &[
            KeyValue::new("mykey1", "myvalue1"),
            KeyValue::new("mykey2", "myvalue2"),
            KeyValue::new("mykey3", "myvalue3"),
            KeyValue::new("mykey4", "myvalue4"),
        ],
    );

    // Example 2 - Change cardinality using View.
    let histogram2 = meter
        .f64_histogram("my_second_histogram")
        .with_unit("ms")
        .with_description("My histogram example description")
        .build();

    // Record measurements using the histogram instrument. This metric will have
    // a cardinality limit of 2, as set in the view. Because of this, only the
    // first two distinct attribute combinations will be recorded, and the rest
    // will be folded into the overflow attribute. Any number of measurements
    // can be recorded as long as they use the same or already-seen attribute
    // combinations.
    histogram2.record(1.5, &[KeyValue::new("mykey1", "v1")]);
    histogram2.record(1.2, &[KeyValue::new("mykey1", "v2")]);

    // Repeatedly emitting measurements for "v1" and "v2" will not
    // trigger overflow, as they are already seen attribute combinations.
    histogram2.record(1.7, &[KeyValue::new("mykey1", "v1")]);
    histogram2.record(1.8, &[KeyValue::new("mykey1", "v2")]);

    // Emitting measurements for new attribute combinations will trigger
    // overflow, as the cardinality limit of 2 has been reached.
    // All the below measurements will be folded into the overflow attribute.
    histogram2.record(1.23, &[KeyValue::new("mykey1", "v3")]);

    histogram2.record(1.4, &[KeyValue::new("mykey1", "v4")]);

    histogram2.record(1.6, &[KeyValue::new("mykey1", "v5")]);

    histogram2.record(1.7, &[KeyValue::new("mykey1", "v6")]);

    histogram2.record(1.8, &[KeyValue::new("mykey1", "v7")]);

    // Metrics are exported by default every 60 seconds when using stdout exporter,
    // however shutting down the MeterProvider here instantly flushes
    // the metrics, instead of waiting for the 60 sec interval.
    meter_provider.shutdown()?;
    Ok(())
}
