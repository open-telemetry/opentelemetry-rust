use opentelemetry::global;
use opentelemetry::KeyValue;
use opentelemetry_appender_tracing::layer;
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

use std::error::Error;

use std::sync::mpsc::channel;

fn init_logger_provider() -> opentelemetry_sdk::logs::LoggerProvider {
    let provider = opentelemetry_otlp::new_pipeline()
        .logging()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint("http://localhost:4318/v1/logs"),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .unwrap();
    let cloned_provider = provider.clone();

    // Add a tracing filter to filter events from crates used by opentelemetry-otlp.
    // The filter levels are set as follows:
    // - Allow `info` level and above by default.
    // - Restrict `hyper`, `tonic`, and `reqwest` to `error` level logs only.
    // This ensures events generated from these crates within the OTLP Exporter are not looped back,
    // thus preventing infinite event generation.
    // Note: This will also drop events from these crates used outside the OTLP Exporter.
    // For more details, see: https://github.com/open-telemetry/opentelemetry-rust/issues/761
    let filter = EnvFilter::new("info")
        .add_directive("hyper=error".parse().unwrap())
        .add_directive("tonic=error".parse().unwrap())
        .add_directive("reqwest=error".parse().unwrap());

    // Configuring the formatting layer specifically for OpenTelemetry internal logs.
    // These logs starts with "opentelemetry" prefix in target. This allows specific logs
    // from the OpenTelemetry-related components to be filtered and handled separately
    // from the application logs

    let opentelemetry_filter = tracing_subscriber::filter::filter_fn(|metadata| {
        metadata.target().starts_with("opentelemetry")
    });

    let fmt_opentelemetry_layer = fmt::layer()
        .with_filter(LevelFilter::DEBUG)
        .with_filter(opentelemetry_filter);

    // Configures the appender tracing layer, filtering out OpenTelemetry internal logs
    // to prevent infinite logging loops.

    let non_opentelemetry_filter = tracing_subscriber::filter::filter_fn(|metadata| {
        !metadata.target().starts_with("opentelemetry")
    });

    let otel_layer = layer::OpenTelemetryTracingBridge::new(&cloned_provider)
        .with_filter(non_opentelemetry_filter.clone());

    tracing_subscriber::registry()
        .with(fmt_opentelemetry_layer)
        .with(fmt::layer().with_filter(filter))
        .with(otel_layer)
        .init();
    provider
}

fn init_meter_provider() -> opentelemetry_sdk::metrics::SdkMeterProvider {
    let provider = opentelemetry_otlp::new_pipeline()
        .metrics(opentelemetry_sdk::runtime::Tokio)
        .with_period(std::time::Duration::from_secs(1))
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint("http://localhost:4318/v1/metrics"),
        )
        .build()
        .unwrap();
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
    let counter = meter.u64_counter("my_counter").init();

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
