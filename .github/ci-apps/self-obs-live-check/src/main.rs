// Purpose-built binary for the sdk-self-observability CI workflow.
//
// Exercises the otel.sdk.processor.log.processed metric by:
//   1. Creating a MeterProvider whose OTLP/gRPC exporter points at weaver
//   2. Creating a LoggerProvider with a BatchLogProcessor (in-memory exporter)
//   3. Emitting log records to trigger the processor metric
//   4. Shutting down to flush the metric export
//
// Only METRICS go to weaver (for live-check validation). Log records use
// an in-memory exporter — they are NOT sent to weaver.

use opentelemetry::global;
use opentelemetry::logs::{LogRecord, Logger, LoggerProvider};
use opentelemetry_otlp::MetricExporter;
use opentelemetry_sdk::{
    logs::{InMemoryLogExporter, SdkLoggerProvider},
    metrics::{PeriodicReader, SdkMeterProvider},
    Resource,
};

const SERVICE_NAME: &str = "self-obs-live-check";

#[tokio::main]
async fn main() {
    // MeterProvider: exports metrics to weaver via OTLP/gRPC
    let metric_exporter = MetricExporter::builder().with_tonic().build().unwrap();
    let reader = PeriodicReader::builder(metric_exporter).build();
    let meter_provider = SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(Resource::builder().with_service_name(SERVICE_NAME).build())
        .build();
    global::set_meter_provider(meter_provider.clone());

    // LoggerProvider: BatchLogProcessor generates otel.sdk.processor.log.processed
    // metrics; logs go to an in-memory sink (not weaver).
    let log_exporter = InMemoryLogExporter::default();
    let logger_provider = SdkLoggerProvider::builder()
        .with_resource(Resource::builder().with_service_name(SERVICE_NAME).build())
        .with_batch_exporter(log_exporter)
        .build();

    // Emit log records to exercise otel.sdk.processor.log.processed
    let logger = logger_provider.logger("self-obs-live-check");
    for _ in 0..10 {
        let mut record = logger.create_log_record();
        record.set_body("test".into());
        logger.emit(record);
    }

    // Wait for the periodic metric export to fire
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // Shutdown flushes remaining metrics to weaver
    let _ = logger_provider.shutdown();
    let _ = meter_provider.shutdown();
}
