use opentelemetry_appender_tracing::layer;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[test]
fn test_no_stack_overflow_when_event_is_emitted_after_shutdown() {
    let exporter = opentelemetry_stdout::LogExporter::default();
    let provider: SdkLoggerProvider = SdkLoggerProvider::builder()
        .with_batch_exporter(exporter)
        .build();

    let otel_layer = layer::OpenTelemetryTracingBridge::new(&provider);

    tracing_subscriber::registry().with(otel_layer).init();

    provider.shutdown().unwrap();

    info!("Don't crash")
}
