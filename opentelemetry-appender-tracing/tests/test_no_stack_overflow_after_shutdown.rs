use opentelemetry_appender_tracing::layer;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[test]
fn test_logging_after_shutdown_does_not_cause_telemetry_induced_telemetry() {
    //! Reproduces [#3161](https://github.com/open-telemetry/opentelemetry-rust/issues/3161)
    let exporter = opentelemetry_stdout::LogExporter::default();
    let provider: SdkLoggerProvider = SdkLoggerProvider::builder()
        .with_batch_exporter(exporter)
        .build();

    let otel_layer = layer::OpenTelemetryTracingBridge::new(&provider);

    tracing_subscriber::registry().with(otel_layer).init();

    provider.shutdown().unwrap();

    // If logging causes telemetry-induced-telemetry after shutting down the provider, then a stack
    // overflow may occur.
    info!("Don't crash")
}
