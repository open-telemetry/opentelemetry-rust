use opentelemetry_api::KeyValue;
use opentelemetry_appender_tracing::layer;
use opentelemetry_sdk::{
    logs::{Config, LoggerProvider},
    Resource,
};
use opentelemetry_user_events_logs::{ExporterConfig, ReentrantLogProcessor};
use tracing::error;
use tracing_subscriber::{prelude::*, Layer};

mod throughput;

struct NoopEventVisitor;

impl tracing::field::Visit for NoopEventVisitor {
    fn record_debug(&mut self, _field: &tracing::field::Field, _value: &dyn std::fmt::Debug) {}
}

struct NoOpLogLayer;
impl<S> Layer<S> for NoOpLogLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = NoopEventVisitor;
        event.record(&mut visitor);
    }

    fn event_enabled(
        &self,
        _event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) -> bool {
        true
    }
}

fn init_logger() -> LoggerProvider {
    let exporter_config = ExporterConfig { keyword: 1 };
    let reenterant_processor = ReentrantLogProcessor::new("test", None, exporter_config);
    LoggerProvider::builder()
        .with_log_processor(reenterant_processor)
        .build()
}

fn main() {
    // LoggerProvider with user_events exporter.
    let provider: LoggerProvider = init_logger();

    // Use the OpenTelemetryTracingBridge to test the throughput of the appender-tracing.
    let layer = layer::OpenTelemetryTracingBridge::new(&provider);
    tracing_subscriber::registry().with(layer).init();
    throughput::test_throughput(test_log);
}

fn test_log() {
    error!(target: "my-system", event_id = 20, event_name = "my-event_name", user_name = "otel", user_email = "otel@opentelemetry.io");
}
