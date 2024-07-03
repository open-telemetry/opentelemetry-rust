use opentelemetry::global::{self, set_error_handler, Error as OtelError};
use opentelemetry::KeyValue;
use opentelemetry_appender_tracing::layer;
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

use std::error::Error;
use tracing::error;

use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use ctrlc;
use std::sync::mpsc::channel;

struct ErrorState {
    seen_errors: Mutex<HashSet<String>>,
}

impl ErrorState {
    fn new() -> Self {
        ErrorState {
            seen_errors: Mutex::new(HashSet::new()),
        }
    }

    fn mark_as_seen(&self, err: &OtelError) -> bool {
        let mut seen_errors = self.seen_errors.lock().unwrap();
        seen_errors.insert(err.to_string())
    }
}

static GLOBAL_ERROR_STATE: Lazy<Arc<ErrorState>> = Lazy::new(|| Arc::new(ErrorState::new()));

fn custom_error_handler(err: OtelError) {
    if GLOBAL_ERROR_STATE.mark_as_seen(&err) {
        // log error not already seen
        match err {
            OtelError::Metric(err) => error!("OpenTelemetry metrics error occurred: {}", err),
            OtelError::Trace(err) => error!("OpenTelemetry trace error occurred: {}", err),
            OtelError::Log(err) => error!("OpenTelemetry log error occurred: {}", err),
            OtelError::Propagation(err) => {
                error!("OpenTelemetry propagation error occurred: {}", err)
            }
            OtelError::Other(err_msg) => error!("OpenTelemetry error occurred: {}", err_msg),
            _ => error!("OpenTelemetry error occurred: {:?}", err),
        }
    }
}

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
    let cloned_provider = provider.clone();
    let layer = layer::OpenTelemetryTracingBridge::new(&cloned_provider);
    tracing_subscriber::registry()
        .with(filter)
        .with(layer)
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
    // Set the custom error handler
    if let Err(err) = set_error_handler(custom_error_handler) {
        eprintln!("Failed to set custom error handler: {}", err);
    }

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
