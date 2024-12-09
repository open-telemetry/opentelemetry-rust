use opentelemetry::KeyValue;
use opentelemetry_appender_tracing::layer;
use opentelemetry_sdk::logs::LoggerProvider;
use opentelemetry_sdk::Resource;
use tracing::error;
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
    let layer = layer::OpenTelemetryTracingBridge::new(&provider);
    tracing_subscriber::registry().with(layer).init();

    error!(name: "my-event-name", target: "my-system", event_id = 20, user_name = "otel", user_email = "otel@opentelemetry.io", message = "This is an example message");
    let _ = provider.shutdown();
}
