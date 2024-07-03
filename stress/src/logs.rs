/*
    Stress test results:
    OS: Ubuntu 22.04.3 LTS (5.15.146.1-microsoft-standard-WSL2)
    Hardware: AMD EPYC 7763 64-Core Processor - 2.44 GHz, 16vCPUs,
    RAM: 64.0 GB
    69 M/sec
*/

use opentelemetry_appender_tracing::layer;
use opentelemetry_sdk::logs::{LogProcessor, LoggerProvider};
use tracing::error;
use tracing_subscriber::prelude::*;

mod throughput;

#[derive(Debug)]
pub struct NoOpLogProcessor;

impl LogProcessor for NoOpLogProcessor {
    fn emit(&self, _data: &mut opentelemetry_sdk::export::logs::LogData) {}

    fn force_flush(&self) -> opentelemetry::logs::LogResult<()> {
        Ok(())
    }

    fn shutdown(&self) -> opentelemetry::logs::LogResult<()> {
        Ok(())
    }

    fn event_enabled(
        &self,
        _level: opentelemetry::logs::Severity,
        _target: &str,
        _name: &str,
    ) -> bool {
        true
    }
}

fn main() {
    // LoggerProvider with a no-op processor.
    let provider: LoggerProvider = LoggerProvider::builder()
        .with_log_processor(NoOpLogProcessor {})
        .build();

    // Use the OpenTelemetryTracingBridge to test the throughput of the appender-tracing.
    let layer = layer::OpenTelemetryTracingBridge::new(&provider);
    tracing_subscriber::registry().with(layer).init();
    throughput::test_throughput(test_log);
}

fn test_log() {
    error!(target: "my-system", event_id = 20, event_name = "my-event_name", user_name = "otel", user_email = "otel@opentelemetry.io");
}
