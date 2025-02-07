/*
    Stress test results:
    OS: Ubuntu 22.04.4 LTS (5.15.153.1-microsoft-standard-WSL2)
    Hardware: Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz, 16vCPUs,
    RAM: 64.0 GB
    ~31 M/sec

    Hardware: AMD EPYC 7763 64-Core Processor - 2.44 GHz, 16vCPUs,
    ~40 M /sec
*/

use opentelemetry::InstrumentationScope;
use opentelemetry_appender_tracing::layer;
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::logs::{LogBatch, LogExporter};
use opentelemetry_sdk::logs::{LogProcessor, SdkLogRecord, SdkLoggerProvider};

use tracing::error;
use tracing_subscriber::prelude::*;

mod throughput;

#[derive(Debug, Clone)]
struct MockLogExporter;

impl LogExporter for MockLogExporter {
    fn export(
        &self,
        _batch: LogBatch<'_>,
    ) -> impl std::future::Future<Output = OTelSdkResult> + Send {
        async { Ok(()) }
    }
}

#[derive(Debug)]
pub struct MockLogProcessor {
    exporter: MockLogExporter,
}

impl LogProcessor for MockLogProcessor {
    fn emit(
        &self,
        record: &mut opentelemetry_sdk::logs::SdkLogRecord,
        scope: &InstrumentationScope,
    ) {
        let log_tuple = &[(record as &SdkLogRecord, scope)];
        let _ = futures_executor::block_on(self.exporter.export(LogBatch::new(log_tuple)));
    }

    fn force_flush(&self) -> OTelSdkResult {
        Ok(())
    }

    fn shutdown(&self) -> OTelSdkResult {
        Ok(())
    }
}

fn main() {
    // LoggerProvider with a no-op processor.
    let provider: SdkLoggerProvider = SdkLoggerProvider::builder()
        .with_log_processor(MockLogProcessor {
            exporter: MockLogExporter {},
        })
        .build();

    // Use the OpenTelemetryTracingBridge to test the throughput of the appender-tracing.
    let layer = layer::OpenTelemetryTracingBridge::new(&provider);
    tracing_subscriber::registry().with(layer).init();
    throughput::test_throughput(test_log);
}

fn test_log() {
    error!(
        name = "CheckoutFailed",
        book_id = "12345",
        book_title = "Rust Programming Adventures",
        message = "Unable to process checkout."
    );
}
