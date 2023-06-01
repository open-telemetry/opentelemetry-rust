//! run with `$ cargo run --example basic --all-features

use opentelemetry_appender_tracing::{layer};
use tracing::{info, error};
use opentelemetry_sdk::{logs::{LoggerProvider, Config}, Resource};
use opentelemetry_api::KeyValue;
use tracing_subscriber::prelude::*;

fn main() {
    let exporter  = opentelemetry_stdout::LogExporter::default();
    let provider: LoggerProvider = LoggerProvider::builder()
    .with_config(
        Config::default().with_resource(Resource::new(vec![KeyValue::new(
            "service.name",
            "log-appender-tracing-example",
        )])),
    )
    .with_simple_exporter(exporter)
    .build();
    let layer  = layer::OpenTelemetryTracingBridge::new(&provider);
    tracing_subscriber::registry().with(layer).init();

    error!(fruit = "apple", price = 2.99, message = "hello fruit with price");
    drop(provider);
}