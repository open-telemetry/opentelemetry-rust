use opentelemetry::global::{self, set_error_handler, Error as OtelError};
use opentelemetry::KeyValue;
use opentelemetry_appender_tracing::layer;
use opentelemetry_sdk::logs::LoggerProvider;
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::{runtime, Resource};
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

use std::error::Error;
use tracing::error;

fn custom_error_handler(err: OtelError) {
    match err {
        OtelError::Metric(err) => error!("OpenTelemetry metrics error occurred: {}", err),
        OtelError::Trace(err) => error!("OpenTelemetry trace error occurred: {}", err),
        OtelError::Log(err) => error!("OpenTelemetry log error occurred: {}", err),
        OtelError::Propagation(err) => error!("OpenTelemetry propagation error occurred: {}", err),
        OtelError::Other(err_msg) => error!("OpenTelemetry error occurred: {}", err_msg),
        _ => error!("An unknown OpenTelemetry error occurred"), //won't reach here
    }
}

fn init_self_diagnostics() {
    // Set the custom error handler
    if let Err(err) = set_error_handler(custom_error_handler) {
        eprintln!("Failed to set custom error handler: {}", err);
    }
    //init tracing subscriber
    let exporter = opentelemetry_stdout::LogExporter::default();
    let provider: LoggerProvider = LoggerProvider::builder()
        .with_simple_exporter(exporter)
        .build();
    //  create filter to only allow error messages
    let filter = EnvFilter::new("error");
    let layer = layer::OpenTelemetryTracingBridge::new(&provider);
    tracing_subscriber::registry()
        .with(filter)
        .with(layer)
        .init();
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

    // Record measurements with unique key-value pairs to exceed the cardinality limit
    // and trigger error message
    for i in 0..2001 {
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
    // shutdown again to trigger error message.
    meter_provider.shutdown()?;
    Ok(())
}
