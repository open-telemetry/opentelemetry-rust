//! run with `$ cargo run --example basic

use opentelemetry::KeyValue;
use opentelemetry_appender_tracing::layer;
use opentelemetry_sdk::{logs::LoggerProvider, Resource};
use tracing::{error, Subscriber};
use tracing_subscriber::layer::{Context, Layer};
use tracing_subscriber::prelude::*;
use tracing_subscriber::registry::LookupSpan;

struct SimpleLogLayer;

impl<S> Layer<S> for SimpleLogLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        // Extract the target of the event
        let target = event.metadata().target();

        // Check if the event is for the target "opentelemetry-internal"
        if target == "opentelemetry-internal" {
            // Print the event in a simple format
            println!("OTEL_INTERNAL_LOG - {}: {:?}", target, event);
        }
    }
}

fn main() {
    let exporter = opentelemetry_stdout::LogExporter::default();
    let provider: LoggerProvider = LoggerProvider::builder()
        .with_resource(Resource::new(vec![KeyValue::new(
            "service.name",
            "log-appender-tracing-example",
        )]))
        .with_simple_exporter(exporter)
        .build();
    let simple_log_layer = SimpleLogLayer;
    let otel_layer = layer::OpenTelemetryTracingBridge::new(&provider);
    tracing_subscriber::registry()
        .with(simple_log_layer)
        .with(otel_layer)
        .init();
    error!(name: "my-event-name", target: "my-system", event_id = 20, user_name = "otel", user_email = "otel@opentelemetry.io", message = "This is an example message");
    let _ = provider.shutdown();
}
