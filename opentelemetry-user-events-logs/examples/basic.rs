//! run with `$ cargo run --example basic --all-features

use opentelemetry_appender_tracing::layer;
use opentelemetry_sdk::logs::LoggerProvider;
use opentelemetry_user_events_logs::{ExporterConfig, ReentrantLogProcessor};
use std::collections::HashMap;
use tracing::error;
use tracing_subscriber::prelude::*;

fn init_logger() -> LoggerProvider {
    let exporter_config = ExporterConfig {
        default_keyword: 1,
        keywords_map: HashMap::new(),
    };
    let reenterant_processor = ReentrantLogProcessor::new("test", None, exporter_config);
    LoggerProvider::builder()
        .with_log_processor(reenterant_processor)
        .build()
}

fn main() {
    // Example with tracing appender.
    let logger_provider = init_logger();
    let layer = layer::OpenTelemetryTracingBridge::new(&logger_provider);
    tracing_subscriber::registry().with(layer).init();

    // event_name is now passed as an attribute, but once https://github.com/tokio-rs/tracing/issues/1426
    // is done, it can be passed with name:"my-event-name", so it'll be available as metadata for
    // fast filtering.
    // event_id is also passed as an attribute now, there is nothing in metadata where a
    // numeric id can be stored.
    error!(
        name: "my-event-name",
        event_id = 20,
        user_name = "otel user",
        user_email = "otel@opentelemetry.io"
    );
}
