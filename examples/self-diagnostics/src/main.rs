use opentelemetry::global::{self, set_error_handler, Error as OtelError};
use opentelemetry::KeyValue;
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::{runtime, Resource};

use std::error::Error;
use tracing::error;

fn custom_error_handler(err: OtelError) {
    match err {
        #[cfg(feature = "metrics")]
        Error::Metric(err) => error!("OpenTelemetry metrics error occurred: {}", err),
        #[cfg(feature = "trace")]
        Error::Trace(err) => error!("OpenTelemetry trace error occurred: {}", err),
        #[cfg(feature = "logs")]
        Error::Log(err) => error!("OpenTelemetry log error occurred: {}", err),
        OtelError::Propagation(err) => error!("OpenTelemetry propagation error occurred: {}", err),
        OtelError::Other(err_msg) => error!("OpenTelemetry error occurred: {}", err_msg),
        _ => error!("An unknown OpenTelemetry error occurred."),
    }
}

fn init_self_diagnostics() {
    // Set the custom error handler
    if let Err(err) = set_error_handler(custom_error_handler) {
        eprintln!("Failed to set custom error handler: {}", err);
    }
    // TODO - use otel-tracing-subscriber or provide option to use either.
    tracing_subscriber::fmt::init();
}

fn init_meter_provider() -> opentelemetry_sdk::metrics::SdkMeterProvider {
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
    global::set_meter_provider(provider.clone());
    provider
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    init_self_diagnostics();

    // Initialize the MeterProvider with the stdout Exporter.
    let meter_provider = init_meter_provider();

    // Create a meter from the above MeterProvider.
    let meter = global::meter("mylibraryname");

    // Create a Counter Instrument.
    let counter = meter.u64_counter("my_counter").init();

    // Record measurements with unique key-value pairs, and enforce the error message
    // generation by exceeding the cardinality limit
    for i in 0..2100 {
        counter.add(
            10,
            &[KeyValue::new(
                format!("mykey{}", i),
                format!("myvalue{}", i),
            )],
        );
    }

    // Metrics are exported by default every 30 seconds when using stdout exporter,
    // however shutting down the MeterProvider here instantly flushes
    // the metrics, instead of waiting for the 30 sec interval.
    meter_provider.shutdown()?;
    Ok(())
}
