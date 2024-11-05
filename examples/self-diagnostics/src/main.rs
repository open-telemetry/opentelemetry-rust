use opentelemetry::global::{self, Error as OtelError};
use opentelemetry::KeyValue;
use opentelemetry_appender_tracing::layer;
use opentelemetry_otlp::{LogExporter, MetricExporter, WithExportConfig};
use opentelemetry_sdk::metrics::PeriodicReader;
use tracing_subscriber::prelude::*;

use std::error::Error;

use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use std::sync::mpsc::channel;

fn init_logger_provider() -> opentelemetry_sdk::logs::LoggerProvider {
    let exporter = LogExporter::builder()
        .with_http()
        .with_endpoint("http://localhost:4318/v1/logs")
        .build()
        .unwrap();

    let provider = opentelemetry_sdk::logs::LoggerProvider::builder()
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .build();

    let cloned_provider = provider.clone();

    // Specialized filter to process
    // - ERROR logs from specific targets
    // - ERROR logs generated internally.
    let internal_and_dependency_filter = tracing_subscriber::filter::filter_fn(|metadata| {
        let target = metadata.target();

        // Only allow ERROR logs from specific targets
        (target.starts_with("hyper")
            || target.starts_with("hyper_util")
            || target.starts_with("hyper")
            || target.starts_with("tonic")
            || target.starts_with("tower")
            || target.starts_with("reqwest")
            || target.starts_with("opentelemetry_"))
            && metadata.level() == &tracing::Level::ERROR
    });
    // Configure fmt::Layer to print detailed log information, including structured fields
    let fmt_internal_and_dependency_layer =
        tracing_subscriber::fmt::layer().with_filter(internal_and_dependency_filter.clone());

    // Application filter to exclude specific targets entirely, regardless of level
    let application_filter = tracing_subscriber::filter::filter_fn(|metadata| {
        let target = metadata.target();

        // Exclude logs from specific targets for the application layer
        !(target.starts_with("hyper")
            || target.starts_with("hyper_util")
            || target.starts_with("hyper")
            || target.starts_with("tonic")
            || target.starts_with("tower")
            || target.starts_with("reqwest")
            || target.starts_with("opentelemetry"))
    });

    let application_layer = layer::OpenTelemetryTracingBridge::new(&cloned_provider)
        .with_filter(application_filter.clone());

    tracing_subscriber::registry()
        .with(fmt_internal_and_dependency_layer)
        .with(application_filter)
        .init();
    provider
}

fn init_meter_provider() -> opentelemetry_sdk::metrics::SdkMeterProvider {
    let exporter = MetricExporter::builder()
        .with_http()
        .with_endpoint("http://localhost:4318/v1/metrics")
        .build()
        .unwrap();

    let reader = PeriodicReader::builder(exporter, opentelemetry_sdk::runtime::Tokio)
        .with_interval(std::time::Duration::from_secs(1))
        .build();

    let provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_reader(reader)
        .build();

    let cloned_provider = provider.clone();
    global::set_meter_provider(cloned_provider);
    provider
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let logger_provider = init_logger_provider();

    // Initialize the MeterProvider with the stdout Exporter.
    let meter_provider = init_meter_provider();

    // Create a meter from the above MeterProvider.
    let meter = global::meter("example");
    // Create a Counter Instrument.
    let counter = meter.u64_counter("my_counter").build();

    // Record measurements with unique key-value pairs to exceed the cardinality limit
    // of 2000 and trigger error message
    for i in 0..3000 {
        counter.add(
            10,
            &[KeyValue::new(
                format!("mykey{}", i),
                format!("myvalue{}", i),
            )],
        );
    }

    let (tx, rx) = channel();

    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");

    println!("Press Ctrl-C to continue...");
    rx.recv().expect("Could not receive from channel.");
    println!("Got Ctrl-C, Doing shutdown and existing.");

    // MeterProvider is configured with an OTLP Exporter to export metrics every 1 second,
    // however shutting down the MeterProvider here instantly flushes
    // the metrics, instead of waiting for the 1 sec interval.
    meter_provider.shutdown()?;
    let _ = logger_provider.shutdown();
    Ok(())
}
