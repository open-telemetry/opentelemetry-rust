/*
    Stress test results:
    OS: Ubuntu 22.04.4 LTS (5.15.153.1-microsoft-standard-WSL2)
    Hardware: Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz, 16vCPUs,
    RAM: 64.0 GB
    ~31 M/sec

    Hardware: AMD EPYC 7763 64-Core Processor - 2.44 GHz, 16vCPUs,
    ~44 M /sec
*/

use opentelemetry::InstrumentationScope;
use opentelemetry_appender_tracing::layer;
use opentelemetry_sdk::export::logs::{LogBatch, LogExporter};
use opentelemetry_sdk::logs::{LogProcessor, LogRecord, LogResult, LoggerProvider};
use tracing::error;
use tracing_subscriber::prelude::*;

mod throughput;
use async_trait::async_trait;

#[derive(Debug, Clone)]
struct NoopExporter;

#[async_trait]
impl LogExporter for NoopExporter {
    async fn export(&self, _: LogBatch<'_>) -> LogResult<()> {
        LogResult::Ok(())
    }
}

#[derive(Debug)]
pub struct NoOpLogProcessor {
    exporter: NoopExporter,
}

impl LogProcessor for NoOpLogProcessor {
    fn emit(&self, record: &mut opentelemetry_sdk::logs::LogRecord, scope: &InstrumentationScope) {
        let log_tuple = &[(record as &LogRecord, scope)];
        let _ = self.exporter.export(LogBatch::new(log_tuple));
    }

    fn force_flush(&self) -> opentelemetry_sdk::logs::LogResult<()> {
        Ok(())
    }

    fn shutdown(&self) -> opentelemetry_sdk::logs::LogResult<()> {
        Ok(())
    }
}

fn main() {
    // LoggerProvider with a no-op processor.
    let provider: LoggerProvider = LoggerProvider::builder()
        .with_log_processor(NoOpLogProcessor {
            exporter: NoopExporter {},
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
