use opentelemetry::InstrumentationScope;

use crate::error::OTelSdkResult;

use super::{LogBatch, LogExporter, LogProcessor, SdkLogRecord};

/// A concurrent log processor calls exporter's export method on each emit. This
/// processor does not buffer logs. Note: This invokes exporter's export method
/// on the current thread without synchronization. i.e multiple export() calls
/// can happen simultaneously from different threads. This is not a problem if
/// the exporter is designed to handle that. As of now, exporters in the
/// opentelemetry-rust project (stdout/otlp) are not thread-safe.
/// This is intended to be used when exporting to operating system
/// tracing facilities like Windows ETW, Linux TracePoints etc.
#[derive(Debug)]
pub struct SimpleConcurrentProcessor<T: LogExporter> {
    exporter: T,
}

impl<T: LogExporter> SimpleConcurrentProcessor<T> {
    /// Creates a new `ConcurrentExportProcessor` with the given exporter.
    pub fn new(exporter: T) -> Self {
        Self { exporter }
    }
}

impl<T: LogExporter> LogProcessor for SimpleConcurrentProcessor<T> {
    fn emit(&self, record: &mut SdkLogRecord, instrumentation: &InstrumentationScope) {
        let log_tuple = &[(record as &SdkLogRecord, instrumentation)];
        let _ = futures_executor::block_on(self.exporter.export(LogBatch::new(log_tuple)));
    }

    fn force_flush(&self) -> OTelSdkResult {
        // TODO: invoke flush on exporter
        // once https://github.com/open-telemetry/opentelemetry-rust/issues/2261
        // is resolved
        Ok(())
    }

    fn shutdown(&self) -> OTelSdkResult {
        self.exporter.shutdown()
    }

    #[cfg(feature = "spec_unstable_logs_enabled")]
    fn event_enabled(
        &self,
        level: opentelemetry::logs::Severity,
        target: &str,
        name: Option<&str>,
    ) -> bool {
        self.exporter.event_enabled(level, target, name)
    }
}
