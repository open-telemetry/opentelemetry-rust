//! run with `$ cargo run --example basic

use opentelemetry::KeyValue;
use opentelemetry_appender_tracing::layer;
use opentelemetry_sdk::{
    logs::{Config, LoggerProvider},
    Resource,
};
use tracing::{error, info, info_span};
use tracing_subscriber::prelude::*;

fn main() {
    let exporter = opentelemetry_stdout::LogExporter::default();
    let provider: LoggerProvider = LoggerProvider::builder()
        .with_config(
            Config::default().with_resource(Resource::new(vec![KeyValue::new(
                "service.name",
                "log-appender-tracing-example",
            )])),
        )
        .with_simple_exporter(exporter)
        .build();
    let layer = layer::OpenTelemetryTracingBridge::new(&provider);
    tracing_subscriber::registry().with(layer).init();

    info_span!("my-span").in_scope(|| {
        info!(target: "my-system", "an info log");
    });

    error!(name: "my-event-name", target: "my-system", event_id = 20, user_name = "otel", user_email = "otel@opentelemetry.io");
    drop(provider);
}
