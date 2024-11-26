use opentelemetry::global;
use opentelemetry::KeyValue;
use opentelemetry_sdk::metrics::PeriodicReader;
use opentelemetry_sdk::Resource;
use std::error::Error;
use tracing::info;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

fn init_meter_provider() -> opentelemetry_sdk::metrics::SdkMeterProvider {
    let exporter = opentelemetry_stdout::MetricExporterBuilder::default().build();

    let reader = PeriodicReader::builder(exporter, opentelemetry_sdk::runtime::Tokio).build();

    let provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_resource(Resource::new([KeyValue::new(
            "service.name",
            "self-diagnostics-example",
        )]))
        .with_reader(reader)
        .build();

    let cloned_provider = provider.clone();
    global::set_meter_provider(cloned_provider);
    provider
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    // OpenTelemetry uses `tracing` crate for its internal logging. Unless a
    // tracing subscriber is set, the logs will be discarded. In this example,
    // we configure a `tracing` subscriber to:
    // 1. Print logs of level DEBUG or higher to stdout using tracing's fmt layer.
    // 2. Filter logs from OpenTelemetry's dependencies (like tonic, hyper,
    // reqwest etc. which are commonly used by the OTLP exporter) to only print
    // ERROR-level logs. This filtering helps reduce repetitive log messages
    // that could otherwise create an infinite loop of log output. This is a
    // workaround until
    // https://github.com/open-telemetry/opentelemetry-rust/issues/761 is
    // resolved.

    // Target names used by all OpenTelemetry official crates always start with "opentelemetry".
    // Hence, one may use "add_directive("opentelemetry=off".parse().unwrap())"
    // to turn off all logs from OpenTelemetry.

    let filter = EnvFilter::new("debug")
        .add_directive("hyper=error".parse().unwrap())
        .add_directive("tonic=error".parse().unwrap())
        .add_directive("h2=error".parse().unwrap())
        .add_directive("tower=error".parse().unwrap())
        .add_directive("reqwest=error".parse().unwrap());
    tracing_subscriber::registry()
        .with(fmt::layer().with_thread_names(true).with_filter(filter))
        .init();

    // Initialize the MeterProvider with the stdout Exporter.
    let meter_provider = init_meter_provider();
    info!("Starting self-diagnostics example");

    let meter = global::meter("example");
    let counter = meter.u64_counter("my_counter").build();
    counter.add(10, &[KeyValue::new("key", "value")]);

    let _observable_counter = meter
        .u64_observable_counter("my_observable_counter")
        .with_callback(|observer| observer.observe(10, &[KeyValue::new("key", "value")]))
        .build();

    meter_provider.shutdown()?;
    info!("Shutdown complete. Bye!");
    Ok(())
}
