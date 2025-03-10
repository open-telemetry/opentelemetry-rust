/*
    Stress test results:
    OS: Ubuntu 22.04.4 LTS (5.15.153.1-microsoft-standard-WSL2)
    Hardware: Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz, 16vCPUs,
    RAM: 64.0 GB
    ~31 M/sec

    Hardware: AMD EPYC 7763 64-Core Processor - 2.44 GHz, 16vCPUs,
    ~44 M /sec

    Hardware: Apple M4 Pro
    Total Number of Cores:	14 (10 performance and 4 efficiency)
    ~50 M/sec
    ~1.1 B/sec (when disabled)

    With existing SimpleLogProcessor:
     3 M/sec (when enabled)  (.with_log_processor(SimpleLogProcessor::new(NoopExporter::new(true))))
    26 M/sec (when disabled) (.with_log_processor(SimpleLogProcessor::new(NoopExporter::new(false)))
*/
use opentelemetry_appender_tracing::layer;
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::logs::concurrent_log_processor::ConcurrentExportProcessor;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::logs::{LogBatch, LogExporter};

use opentelemetry_sdk::Resource;
use tracing::error;
use tracing_subscriber::prelude::*;

mod throughput;

#[derive(Debug)]
struct NoopExporter {
    enabled: bool,
}
impl NoopExporter {
    fn new(enabled: bool) -> Self {
        Self { enabled }
    }
}
impl LogExporter for NoopExporter {
    async fn export(&self, _: LogBatch<'_>) -> OTelSdkResult {
        Ok(())
    }

    fn shutdown(&self) -> OTelSdkResult {
        Ok(())
    }

    fn event_enabled(
        &self,
        _level: opentelemetry::logs::Severity,
        _target: &str,
        _name: Option<&str>,
    ) -> bool {
        self.enabled
    }

    fn set_resource(&mut self, _: &Resource) {}
}

fn main() {
    // change this to false to test the throughput when enabled is false.
    let enabled = true;

    // LoggerProvider with a no-op processor.
    let provider: SdkLoggerProvider = SdkLoggerProvider::builder()
        .with_log_processor(ConcurrentExportProcessor::new(NoopExporter::new(enabled)))
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
