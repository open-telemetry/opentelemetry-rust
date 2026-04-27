/*
    Stress test results:
    Hardware: Apple M4 Pro
    Total Number of Cores:	14 (10 performance and 4 efficiency)
    ~27 M/sec
    ~1.4 B/sec (when disabled)
*/
use opentelemetry::logs::Severity;
use opentelemetry::InstrumentationScope;
use opentelemetry_appender_tracing::layer;
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::logs::{LogProcessor, SdkLogRecord, SdkLoggerProvider};
use opentelemetry_sdk::Resource;
use tracing::error;
use tracing_subscriber::prelude::*;

mod throughput;

#[derive(Debug)]
struct NoopProcessor {
    enabled: bool,
}

impl LogProcessor for NoopProcessor {
    fn emit(&self, _data: &mut SdkLogRecord, _instrumentation: &InstrumentationScope) {}

    fn force_flush(&self) -> OTelSdkResult {
        Ok(())
    }

    fn event_enabled(&self, _level: Severity, _target: &str, _name: Option<&str>) -> bool {
        self.enabled
    }

    fn set_resource(&mut self, _resource: &Resource) {}
}

fn main() {
    // change this to false to test the throughput when disabled.
    let enabled = false;

    // LoggerProvider with a no-op processor.
    let provider: SdkLoggerProvider = SdkLoggerProvider::builder()
        .with_log_processor(NoopProcessor { enabled })
        .build();

    // Use the OpenTelemetryTracingBridge to test the throughput of the appender-tracing.
    let layer = layer::OpenTelemetryTracingBridge::new(&provider);
    tracing_subscriber::registry().with(layer).init();
    throughput::test_throughput(test_log);
}

fn test_log() {
    error!(
        name : "CheckoutFailed",
        book_id = "12345",
        book_title = "Rust Programming Adventures",
        message = "Unable to process checkout."
    );
}
