//! run with `$ cargo run --example basic

use opentelemetry::KeyValue;
use opentelemetry_appender_tracing::layer;
use opentelemetry_sdk::{logs::LoggerProvider, Resource};
use tracing::error;
use tracing_subscriber::filter::filter_fn;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::Layer;
use tracing_subscriber::prelude::*;

fn main() {
    let exporter = opentelemetry_stdout::LogExporter::default();
    let provider: LoggerProvider = LoggerProvider::builder()
        .with_resource(Resource::new(vec![KeyValue::new(
            "service.name",
            "log-appender-tracing-example",
        )]))
        .with_simple_exporter(exporter)
        .build();
    let otel_layer = layer::OpenTelemetryTracingBridge::new(&provider);
    let internal_log_layer = fmt::Layer::default()
        .with_writer(std::io::stdout) // Writes to stdout
        .pretty()
        .with_filter(filter_fn(|meta| meta.target().starts_with("opentelemetry"))); // Custom filter function
    tracing_subscriber::registry()
        .with(internal_log_layer)
        .with(otel_layer)
        .init();
    error!(name: "my-event-name", target: "my-system", event_id = 20, user_name = "otel", user_email = "otel@opentelemetry.io", message = "This is an example message");
    let _ = provider.shutdown();
}
