// Purpose-built binary for the sdk-self-observability CI workflow.
//
// Exercises the SDK's self-observability metrics by:
//   1. Creating a MeterProvider whose OTLP exporter points at weaver
//   2. Creating a LoggerProvider with a BatchLogProcessor (no-op exporter)
//   3. Emitting log records (exercises otel.sdk.log.created + otel.sdk.processor.log.processed)
//   4. Calling shutdown (exercises metrics flush + future shutdown event)
//
// Only METRICS go to weaver (for live-check validation). Log records flow
// through the processor pipeline (to generate the self-obs metrics) but
// are NOT exported to weaver — they would fail validation because they
// carry app-specific attributes the semconv registry doesn't declare.

use opentelemetry::global;
use opentelemetry_otlp::MetricExporter;
use opentelemetry_sdk::{
    logs::{InMemoryLogExporter, SdkLoggerProvider},
    metrics::{PeriodicReader, SdkMeterProvider},
    Resource,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const SERVICE_NAME: &str = "self-obs-live-check";

#[tokio::main]
async fn main() {
    // --- MeterProvider: exports metrics to weaver via OTLP gRPC ---
    let metric_exporter = MetricExporter::builder().with_tonic().build().unwrap();
    let reader = PeriodicReader::builder(metric_exporter).build();
    let meter_provider = SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(Resource::builder().with_service_name(SERVICE_NAME).build())
        .build();
    global::set_meter_provider(meter_provider.clone());

    // --- LoggerProvider: logs pass through the BLP (generating self-obs
    // metrics) but export to an in-memory sink, NOT to weaver. ---
    let log_exporter = InMemoryLogExporter::default();
    let logger_provider = SdkLoggerProvider::builder()
        .with_resource(Resource::builder().with_service_name(SERVICE_NAME).build())
        .with_batch_exporter(log_exporter)
        .build();

    // Wire tracing -> OTel logs
    let otel_layer =
        opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(&logger_provider);
    tracing_subscriber::registry()
        .with(otel_layer)
        .with(tracing_subscriber::fmt::layer().with_target(true))
        .init();

    // --- Emit log records to exercise otel.sdk.log.created ---
    for i in 0..10 {
        tracing::info!(iteration = i, "log record {i}");
    }

    // Brief pause to let the BatchLogProcessor's scheduled export fire,
    // exercising otel.sdk.processor.log.processed and the exporter metrics.
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // --- Shutdown: flushes remaining records, exercises processor/exporter ---
    let _ = logger_provider.shutdown();
    let _ = meter_provider.shutdown();
}
