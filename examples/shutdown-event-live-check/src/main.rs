// Purpose-built binary for validating the otel.sdk.component.shutdown event
// via weaver live-check.
//
// Creates a LoggerProvider with OTLP exporter (pointed at weaver listener),
// then calls shutdown(). The shutdown event flows through the tracing bridge
// → OTel LoggerProvider → OTLP exporter → weaver for validation.

use opentelemetry_otlp::LogExporter;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::Resource;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const SERVICE_NAME: &str = "shutdown-event-live-check";

#[tokio::main]
async fn main() {
    // LoggerProvider with OTLP exporter pointed at weaver (via env var)
    let log_exporter = LogExporter::builder().with_tonic().build().unwrap();
    let logger_provider = SdkLoggerProvider::builder()
        .with_resource(Resource::builder().with_service_name(SERVICE_NAME).build())
        .with_batch_exporter(log_exporter)
        .build();

    // Wire tracing → OTel so the otel_info! shutdown event (which uses
    // tracing internally) flows through this LoggerProvider to weaver.
    let otel_layer =
        opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(&logger_provider);
    // Only capture opentelemetry crate logs (the shutdown event target is
    // the SDK crate name). Filter out h2/tonic/hyper noise.
    let filter = tracing_subscriber::EnvFilter::new("opentelemetry=trace,opentelemetry_sdk=trace");
    tracing_subscriber::registry()
        .with(filter)
        .with(otel_layer)
        .init();

    // Brief pause to let the batch processor's background thread start
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Shutdown: this emits the otel.sdk.component.shutdown event
    let _ = logger_provider.shutdown();

    // Give the OTLP exporter time to flush the shutdown event to weaver
    // (the event is emitted synchronously but the BLP's export is async)
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
}
