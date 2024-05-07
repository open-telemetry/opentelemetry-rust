use opentelemetry::global;
use opentelemetry::metrics::Unit;
use opentelemetry::KeyValue;
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::{runtime, Resource};
use std::error::Error;

fn init_meter_provider() {
    let exporter = opentelemetry_stdout::MetricsExporterBuilder::default()
        // uncomment the below lines to pretty print output.
        //  .with_encoder(|writer, data|
        //    Ok(serde_json::to_writer_pretty(writer, &data).unwrap()))
        .build();
    let reader = PeriodicReader::builder(exporter, runtime::Tokio).build();
    let provider = SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(Resource::new(vec![KeyValue::new(
            "service.name",
            "metrics-basic-example",
        )]))
        .build();
    global::set_meter_provider(provider);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    // Initialize the MeterProvider with the stdout Exporter.
    init_meter_provider();

    // Create a meter from the above MeterProvider.
    let meter = global::meter("mylibraryname");

    // Create a Counter Instrument.
    let counter = meter.u64_counter("my_counter").init();

    // Record measurements using the Counter instrument.
    counter.add(
        10,
        &[
            KeyValue::new("mykey1", "myvalue1"),
            KeyValue::new("mykey2", "myvalue2"),
        ],
    );

    // Create a ObservableCounter instrument and register a callback that reports the measurement.
    let _observable_counter = meter
        .u64_observable_counter("my_observable_counter")
        .with_description("My observable counter example description")
        .with_unit(Unit::new("myunit"))
        .with_callback(|observer| {
            observer.observe(
                100,
                &[
                    KeyValue::new("mykey1", "myvalue1"),
                    KeyValue::new("mykey2", "myvalue2"),
                ],
            )
        })
        .init();

    // Create a UpCounter Instrument.
    let updown_counter = meter.i64_up_down_counter("my_updown_counter").init();

    // Record measurements using the UpCounter instrument.
    updown_counter.add(
        -10,
        &[
            KeyValue::new("mykey1", "myvalue1"),
            KeyValue::new("mykey2", "myvalue2"),
        ],
    );

    // Create a Observable UpDownCounter instrument and register a callback that reports the measurement.
    let _observable_up_down_counter = meter
        .i64_observable_up_down_counter("my_observable_updown_counter")
        .with_description("My observable updown counter example description")
        .with_unit(Unit::new("myunit"))
        .with_callback(|observer| {
            observer.observe(
                100,
                &[
                    KeyValue::new("mykey1", "myvalue1"),
                    KeyValue::new("mykey2", "myvalue2"),
                ],
            )
        })
        .init();

    // Create a Histogram Instrument.
    let histogram = meter
        .f64_histogram("my_histogram")
        .with_description("My histogram example description")
        .init();

    // Record measurements using the histogram instrument.
    histogram.record(
        10.5,
        &[
            KeyValue::new("mykey1", "myvalue1"),
            KeyValue::new("mykey2", "myvalue2"),
        ],
    );

    // Note that there is no ObservableHistogram instrument.

    // Create a Gauge Instrument.
    let gauge = meter
        .f64_gauge("my_gauge")
        .with_description("A gauge set to 1.0")
        .with_unit(Unit::new("myunit"))
        .init();

    gauge.record(
        1.0,
        &[
            KeyValue::new("mykey1", "myvalue1"),
            KeyValue::new("mykey2", "myvalue2"),
        ],
    );

    // Create a ObservableGauge instrument and register a callback that reports the measurement.
    let _observable_gauge = meter
        .f64_observable_gauge("my_observable_gauge")
        .with_description("An observable gauge set to 1.0")
        .with_unit(Unit::new("myunit"))
        .with_callback(|observer| {
            observer.observe(
                1.0,
                &[
                    KeyValue::new("mykey1", "myvalue1"),
                    KeyValue::new("mykey2", "myvalue2"),
                ],
            )
        })
        .init();

    // Metrics are exported by default every 30 seconds when using stdout exporter,
    // however shutting down the MeterProvider here instantly flushes
    // the metrics, instead of waiting for the 30 sec interval.
    global::shutdown_meter_provider();
    Ok(())
}
