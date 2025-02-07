use opentelemetry::{global, KeyValue};
use opentelemetry_sdk::error::OTelSdkError;
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::Resource;
use std::error::Error;
use std::vec;

fn init_meter_provider() -> opentelemetry_sdk::metrics::SdkMeterProvider {
    let exporter = opentelemetry_stdout::MetricExporterBuilder::default()
        // Build exporter using Delta Temporality (Defaults to Temporality::Cumulative)
        // .with_temporality(opentelemetry_sdk::metrics::Temporality::Delta)
        .build();
    let provider = SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .with_resource(
            Resource::builder()
                .with_service_name("metrics-basic-example")
                .build(),
        )
        .build();
    global::set_meter_provider(provider.clone());
    provider
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize the MeterProvider with the stdout Exporter.
    let meter_provider = init_meter_provider();

    // Create a meter from the above MeterProvider.
    let meter = global::meter("mylibraryname");

    // Create a Counter Instrument.
    let counter = meter.u64_counter("my_counter").build();

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

    // Create a UpCounter Instrument.
    let updown_counter = meter.i64_up_down_counter("my_updown_counter").build();

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

    // Create a Histogram Instrument.
    let histogram = meter
        .f64_histogram("my_histogram")
        .with_description("My histogram example description")
        // Setting boundaries is optional. By default, the boundaries are set to
        // [0.0, 5.0, 10.0, 25.0, 50.0, 75.0, 100.0, 250.0, 500.0, 750.0, 1000.0, 2500.0, 5000.0, 7500.0, 10000.0]
        .with_boundaries(vec![0.0, 5.0, 10.0, 15.0, 20.0, 25.0])
        .build();

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
        .with_unit("myunit")
        .build();

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
        .with_unit("myunit")
        .with_callback(|observer| {
            observer.observe(
                1.0,
                &[
                    KeyValue::new("mykey1", "myvalue1"),
                    KeyValue::new("mykey2", "myvalue2"),
                ],
            )
        })
        .build();

    // Metrics are exported by default every 30 seconds when using stdout
    // exporter, however shutting down the MeterProvider here instantly flushes
    // the metrics, instead of waiting for the 30 sec interval. Shutdown returns
    // a result, which is bubbled up to the caller The commented code below
    // demonstrates handling the shutdown result, instead of bubbling up the
    // error.
    meter_provider.shutdown()?;

    // let shutdown_result = meter_provider.shutdown();

    // Handle the shutdown result.
    // match shutdown_result {
    //     Ok(_) => println!("MeterProvider shutdown successfully"),
    //     Err(e) => {
    //         match e {
    //             opentelemetry_sdk::error::ShutdownError::InternalFailure(message) => {
    //                 // This indicates some internal failure during shutdown. The
    //                 // error message is intended for logging purposes only and
    //                 // should not be used to make programmatic decisions.
    //                 println!("MeterProvider shutdown failed: {}", message)
    //             }
    //             opentelemetry_sdk::error::ShutdownError::AlreadyShutdown => {
    //                 // This indicates some user code tried to shutdown
    //                 // elsewhere. user need to review their code to ensure
    //                 // shutdown is called only once.
    //                 println!("MeterProvider already shutdown")
    //             }
    //             opentelemetry_sdk::error::ShutdownError::Timeout(e) => {
    //                 // This indicates the shutdown timed out, and a good hint to
    //                 // user to increase the timeout. (Shutdown method does not
    //                 // allow custom timeout today, but that is temporary)
    //                 println!("MeterProvider shutdown timed out after {:?}", e)
    //             }
    //         }
    //     }
    // }
    Ok(())
}
