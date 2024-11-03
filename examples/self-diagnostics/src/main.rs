use opentelemetry::global::{self, Error as OtelError};
use opentelemetry::KeyValue;
use opentelemetry_appender_tracing::layer;
use opentelemetry_otlp::{LogExporter, MetricExporter, WithExportConfig};
use opentelemetry_sdk::metrics::PeriodicReader;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

use std::error::Error;

use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

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
        let error_message = err.to_string();
        !seen_errors.insert(error_message) // Returns false if already present
    }
}

static GLOBAL_ERROR_STATE: Lazy<Arc<ErrorState>> = Lazy::new(|| Arc::new(ErrorState::new()));


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

    // Filter for logs with "opentelemetry_" target prefix to use eprintln,
    // only if they haven't been logged before.
    let opentelemetry_filter = tracing_subscriber::filter::filter_fn(|metadata| {
        if metadata.target().starts_with("opentelemetry_") {
            // Ignore debug level logs
            if metadata.level() == &tracing::Level::DEBUG {
                return false;
            }
            let err = OtelError::Other(metadata.target().to_string()); // Convert target to an OtelError variant
            if !GLOBAL_ERROR_STATE.mark_as_seen(&err) {
                eprintln!(
                    "[{}] - {}: {}",
                    metadata.level(),
                    metadata.target(),
                    metadata.fields()
                );
            }
            false // Prevent these logs from propagating further
        } else {
            true // Allow other logs to continue
        }
    });

    // Layer for OpenTelemetry internal logs
    let fmt_opentelemetry_layer = fmt::layer().with_filter(opentelemetry_filter);

    // Layer for application logs, excluding OpenTelemetry internal logs
    let fmt_application_layer = fmt::layer().with_filter(filter);

    // Configures the appender tracing layer, filtering out OpenTelemetry internal logs
    // to prevent infinite logging loops.

    let non_opentelemetry_filter = tracing_subscriber::filter::filter_fn(|metadata| {
        !metadata.target().starts_with("opentelemetry")
    });

    let otel_layer = layer::OpenTelemetryTracingBridge::new(&cloned_provider)
        .with_filter(non_opentelemetry_filter.clone());

    tracing_subscriber::registry()
        .with(fmt_opentelemetry_layer)
        .with(fmt_application_layer)
        .with(otel_layer)
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
