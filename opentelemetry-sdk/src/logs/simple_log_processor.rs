//! # OpenTelemetry Simple Log Processor
//! The `SimpleLogProcessor` is one implementation of the `LogProcessor` interface.
//!
//! It forwards log records to the exporter immediately after they are emitted
//! (or one exporter after another if applicable). This processor is
//! **synchronous** and is designed for debugging or testing purposes. It is
//! **not suitable for production** environments due to its lack of batching,
//! performance optimizations, or support for high-throughput scenarios.
//!
//! ## Diagram
//!
//! ```ascii
//!   +-----+---------------+   +-----------------------+   +-------------------+
//!   |     |               |   |                       |   |                   |
//!   | SDK | Logger.emit() +---> (Simple)LogProcessor  +--->  LogExporter      |
//!   +-----+---------------+   +-----------------------+   +-------------------+
//! ```

use crate::error::{OTelSdkError, OTelSdkResult};
use crate::logs::log_processor::LogProcessor;
use crate::{
    logs::{LogBatch, LogExporter, SdkLogRecord},
    Resource,
};

#[cfg(feature = "experimental_metrics_bound_instruments")]
use opentelemetry::KeyValue;
use opentelemetry::{otel_debug, otel_error, otel_warn, Context, InstrumentationScope};

use std::fmt::Debug;
use std::sync::atomic::AtomicBool;
#[cfg(feature = "experimental_metrics_bound_instruments")]
use std::sync::atomic::AtomicUsize;
use std::sync::Mutex;
use std::time::Duration;

/// A [`LogProcessor`] designed for testing and debugging purpose, that immediately
/// exports log records as they are emitted. Log records are exported synchronously
/// in the same thread that emits the log record.
/// When using this processor with the OTLP Exporter, the following exporter
/// features are supported:
/// - `grpc-tonic`: This requires LoggerProvider to be created within a tokio
///   runtime. Logs can be emitted from any thread, including tokio runtime
///   threads.
/// - `reqwest-blocking-client`: LoggerProvider may be created anywhere, but
///   logs must be emitted from a non-tokio runtime thread.
/// - `reqwest-client`: LoggerProvider may be created anywhere, but logs must be
///   emitted from a tokio runtime thread.
///
/// ## Example
///
/// ### Using a SimpleLogProcessor
///
/// ```rust
/// # #[cfg(feature = "testing")]
/// # {
/// use opentelemetry_sdk::logs::{SimpleLogProcessor, SdkLoggerProvider, LogExporter};
/// use opentelemetry::global;
/// use opentelemetry_sdk::logs::InMemoryLogExporter;
///
/// let exporter = InMemoryLogExporter::default(); // Replace with an actual exporter
/// let provider = SdkLoggerProvider::builder()
///     .with_simple_exporter(exporter)
///     .build();
/// # }
/// ```
///
#[derive(Debug)]
pub struct SimpleLogProcessor<T: LogExporter> {
    exporter: Mutex<T>,
    is_shutdown: AtomicBool,

    // Self-diagnostics: otel.sdk.processor.log.processed counter, gated behind
    // experimental_metrics_bound_instruments so the hot-path `add` is a single
    // atomic increment with no per-call attribute resolution.
    //
    // The SimpleLogProcessor submits each record to the exporter synchronously
    // and has no queue, so the only processor-side drop is `already_shutdown`.
    #[cfg(feature = "experimental_metrics_bound_instruments")]
    processed_success: opentelemetry::metrics::BoundCounter<u64>,
    #[cfg(feature = "experimental_metrics_bound_instruments")]
    processed_after_shutdown: opentelemetry::metrics::BoundCounter<u64>,
}

impl<T: LogExporter> SimpleLogProcessor<T> {
    /// Creates a new instance of `SimpleLogProcessor`.
    pub fn new(exporter: T) -> Self {
        #[cfg(feature = "experimental_metrics_bound_instruments")]
        let (processed_success, processed_after_shutdown) = {
            static INSTANCE_COUNTER: AtomicUsize = AtomicUsize::new(0);
            let instance_id = INSTANCE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let component_name = format!("simple_log_processor/{instance_id}");

            let meter = opentelemetry::global::meter("otel.sdk");
            let counter = meter
                .u64_counter("otel.sdk.processor.log.processed")
                .with_description(
                    "The number of log records for which the processing has finished, \
                     either successful or failed.",
                )
                .with_unit("{log_record}")
                .build();

            // Attribute values follow the OTel semantic conventions for SDK metrics:
            // https://github.com/open-telemetry/semantic-conventions/blob/main/docs/otel/sdk-metrics.md#metric-otelsdkprocessorlogprocessed
            // https://github.com/open-telemetry/semantic-conventions/blob/main/docs/registry/attributes/otel.md#otel-component-attributes
            let success_attrs = [
                KeyValue::new("otel.component.type", "simple_log_processor"),
                KeyValue::new("otel.component.name", component_name.clone()),
            ];
            let after_shutdown_attrs = [
                KeyValue::new("error.type", "already_shutdown"),
                KeyValue::new("otel.component.type", "simple_log_processor"),
                KeyValue::new("otel.component.name", component_name),
            ];

            (
                counter.bind(&success_attrs),
                counter.bind(&after_shutdown_attrs),
            )
        };

        SimpleLogProcessor {
            exporter: Mutex::new(exporter),
            is_shutdown: AtomicBool::new(false),
            #[cfg(feature = "experimental_metrics_bound_instruments")]
            processed_success,
            #[cfg(feature = "experimental_metrics_bound_instruments")]
            processed_after_shutdown,
        }
    }
}

impl<T: LogExporter> LogProcessor for SimpleLogProcessor<T> {
    fn emit(&self, record: &mut SdkLogRecord, instrumentation: &InstrumentationScope) {
        let _suppress_guard = Context::enter_telemetry_suppressed_scope();
        // noop after shutdown
        if self.is_shutdown.load(std::sync::atomic::Ordering::Relaxed) {
            // Record the post-shutdown drop in self-diagnostics before returning.
            #[cfg(feature = "experimental_metrics_bound_instruments")]
            self.processed_after_shutdown.add(1);

            // this is a warning, as the user is trying to log after the processor has been shutdown
            otel_warn!(
                name: "SimpleLogProcessor.Emit.ProcessorShutdown",
            );
            return;
        }

        let result = match self.exporter.lock() {
            Ok(exporter) => {
                let log_tuple = &[(record as &SdkLogRecord, instrumentation)];
                // Count the record as processed right before submitting it to
                // the exporter, independent of the export outcome, per semconv.
                // Matches BatchLogProcessor, which records success before export.
                #[cfg(feature = "experimental_metrics_bound_instruments")]
                self.processed_success.add(1);
                futures_executor::block_on(exporter.export(LogBatch::new(log_tuple)))
            }
            Err(_) => Err(OTelSdkError::InternalFailure(
                "SimpleLogProcessor mutex poison".into(),
            )),
        };
        // Handle errors with specific static names
        match result {
            Err(OTelSdkError::InternalFailure(_)) => {
                // logging as debug as this is not a user error
                otel_debug!(
                    name: "SimpleLogProcessor.Emit.MutexPoisoning",
                );
            }
            Err(err) => {
                otel_error!(
                    name: "SimpleLogProcessor.Emit.ExportError",
                    error = format!("{}",err)
                );
            }
            _ => {}
        }
    }

    fn force_flush(&self) -> OTelSdkResult {
        Ok(())
    }

    fn shutdown_with_timeout(&self, timeout: Duration) -> OTelSdkResult {
        self.is_shutdown
            .store(true, std::sync::atomic::Ordering::Relaxed);
        if let Ok(exporter) = self.exporter.lock() {
            exporter.shutdown_with_timeout(timeout)
        } else {
            Err(OTelSdkError::InternalFailure(
                "SimpleLogProcessor mutex poison at shutdown".into(),
            ))
        }
    }

    fn set_resource(&mut self, resource: &Resource) {
        if let Ok(mut exporter) = self.exporter.lock() {
            exporter.set_resource(resource);
        }
    }

    #[inline]
    fn event_enabled(
        &self,
        level: opentelemetry::logs::Severity,
        target: &str,
        name: Option<&str>,
    ) -> bool {
        if let Ok(exporter) = self.exporter.lock() {
            exporter.event_enabled(level, target, name)
        } else {
            true
        }
    }
}

#[cfg(all(test, feature = "testing", feature = "logs"))]
mod tests {
    use crate::logs::log_processor::tests::MockLogExporter;
    use crate::logs::{LogBatch, LogExporter, SdkLogRecord, SdkLogger};
    use crate::{
        error::OTelSdkResult,
        logs::{InMemoryLogExporterBuilder, LogProcessor, SdkLoggerProvider, SimpleLogProcessor},
        Resource,
    };
    use opentelemetry::logs::{LogRecord, Logger, LoggerProvider};
    use opentelemetry::InstrumentationScope;
    use opentelemetry::KeyValue;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time;
    use std::time::Duration;

    #[derive(Debug, Clone)]
    struct LogExporterThatRequiresTokio {
        export_count: Arc<AtomicUsize>,
    }

    impl LogExporterThatRequiresTokio {
        /// Creates a new instance of `LogExporterThatRequiresTokio`.
        fn new() -> Self {
            LogExporterThatRequiresTokio {
                export_count: Arc::new(AtomicUsize::new(0)),
            }
        }

        /// Returns the number of logs stored in the exporter.
        fn len(&self) -> usize {
            self.export_count.load(Ordering::Acquire)
        }
    }

    impl LogExporter for LogExporterThatRequiresTokio {
        async fn export(&self, batch: LogBatch<'_>) -> OTelSdkResult {
            // Simulate minimal dependency on tokio by sleeping asynchronously for a short duration
            tokio::time::sleep(Duration::from_millis(50)).await;

            for _ in batch.iter() {
                self.export_count.fetch_add(1, Ordering::Acquire);
            }
            Ok(())
        }
        fn shutdown_with_timeout(&self, _timeout: time::Duration) -> OTelSdkResult {
            Ok(())
        }
    }

    #[test]
    fn test_set_resource_simple_processor() {
        let exporter = MockLogExporter {
            resource: Arc::new(Mutex::new(None)),
        };
        let processor = SimpleLogProcessor::new(exporter.clone());
        let _ = SdkLoggerProvider::builder()
            .with_log_processor(processor)
            .with_resource(
                Resource::builder_empty()
                    .with_attributes([
                        KeyValue::new("k1", "v1"),
                        KeyValue::new("k2", "v3"),
                        KeyValue::new("k3", "v3"),
                        KeyValue::new("k4", "v4"),
                        KeyValue::new("k5", "v5"),
                    ])
                    .build(),
            )
            .build();
        assert_eq!(exporter.get_resource().unwrap().into_iter().count(), 5);
    }

    #[test]
    fn test_simple_shutdown() {
        let exporter = InMemoryLogExporterBuilder::default()
            .keep_records_on_shutdown()
            .build();
        let processor = SimpleLogProcessor::new(exporter.clone());

        let mut record: SdkLogRecord = SdkLogRecord::new();
        let instrumentation: InstrumentationScope = Default::default();

        processor.emit(&mut record, &instrumentation);

        processor.shutdown().unwrap();

        let is_shutdown = processor
            .is_shutdown
            .load(std::sync::atomic::Ordering::Relaxed);
        assert!(is_shutdown);

        processor.emit(&mut record, &instrumentation);

        assert_eq!(1, exporter.get_emitted_logs().unwrap().len());
        assert!(exporter.is_shutdown_called());
    }

    #[test]
    fn test_simple_processor_sync_exporter_without_runtime() {
        let exporter = InMemoryLogExporterBuilder::default().build();
        let processor = SimpleLogProcessor::new(exporter.clone());

        let mut record: SdkLogRecord = SdkLogRecord::new();
        let instrumentation: InstrumentationScope = Default::default();

        processor.emit(&mut record, &instrumentation);

        assert_eq!(exporter.get_emitted_logs().unwrap().len(), 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_simple_processor_sync_exporter_with_runtime() {
        let exporter = InMemoryLogExporterBuilder::default().build();
        let processor = SimpleLogProcessor::new(exporter.clone());

        let mut record: SdkLogRecord = SdkLogRecord::new();
        let instrumentation: InstrumentationScope = Default::default();

        processor.emit(&mut record, &instrumentation);

        assert_eq!(exporter.get_emitted_logs().unwrap().len(), 1);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_simple_processor_sync_exporter_with_multi_thread_runtime() {
        let exporter = InMemoryLogExporterBuilder::default().build();
        let processor = Arc::new(SimpleLogProcessor::new(exporter.clone()));

        let mut handles = vec![];
        for _ in 0..10 {
            let processor_clone = Arc::clone(&processor);
            let handle = tokio::spawn(async move {
                let mut record: SdkLogRecord = SdkLogRecord::new();
                let instrumentation: InstrumentationScope = Default::default();
                processor_clone.emit(&mut record, &instrumentation);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        assert_eq!(exporter.get_emitted_logs().unwrap().len(), 10);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_simple_processor_sync_exporter_with_current_thread_runtime() {
        let exporter = InMemoryLogExporterBuilder::default().build();
        let processor = SimpleLogProcessor::new(exporter.clone());

        let mut record: SdkLogRecord = SdkLogRecord::new();
        let instrumentation: InstrumentationScope = Default::default();

        processor.emit(&mut record, &instrumentation);

        assert_eq!(exporter.get_emitted_logs().unwrap().len(), 1);
    }

    #[test]
    fn test_simple_processor_async_exporter_without_runtime() {
        // Use `catch_unwind` to catch the panic caused by missing Tokio runtime
        let result = std::panic::catch_unwind(|| {
            let exporter = LogExporterThatRequiresTokio::new();
            let processor = SimpleLogProcessor::new(exporter.clone());

            let mut record: SdkLogRecord = SdkLogRecord::new();
            let instrumentation: InstrumentationScope = Default::default();

            // This will panic because an tokio async operation within exporter without a runtime.
            processor.emit(&mut record, &instrumentation);
        });

        // Verify that the panic occurred and check the panic message for the absence of a Tokio runtime
        assert!(
            result.is_err(),
            "The test should fail due to missing Tokio runtime, but it did not."
        );
        let panic_payload = result.unwrap_err();
        let panic_message = panic_payload
            .downcast_ref::<String>()
            .map(|s| s.as_str())
            .or_else(|| panic_payload.downcast_ref::<&str>().copied())
            .unwrap_or("No panic message");

        assert!(
            panic_message.contains("no reactor running")
                || panic_message.contains("must be called from the context of a Tokio 1.x runtime"),
            "Expected panic message about missing Tokio runtime, but got: {panic_message}"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[ignore]
    // This test demonstrates a potential deadlock scenario in a multi-threaded Tokio runtime.
    // It spawns Tokio tasks equal to the number of runtime worker threads (4) to emit log events.
    // Each task attempts to acquire a mutex on the exporter in `SimpleLogProcessor::emit`.
    // Only one task obtains the lock, while the others are blocked, waiting for its release.
    //
    // The task holding the lock invokes the LogExporterThatRequiresTokio, which performs an
    // asynchronous operation (e.g., network I/O simulated by `tokio::sleep`). This operation
    // requires yielding control back to the Tokio runtime to make progress.
    //
    // However, all worker threads are occupied:
    // - One thread is executing the async exporter operation
    // - Three threads are blocked waiting for the mutex
    //
    // This leads to a deadlock as there are no available threads to drive the async operation
    // to completion, preventing the mutex from being released. Consequently, neither the blocked
    // tasks nor the exporter can proceed.
    async fn test_simple_processor_async_exporter_with_all_runtime_worker_threads_blocked() {
        let exporter = LogExporterThatRequiresTokio::new();
        let processor = Arc::new(SimpleLogProcessor::new(exporter.clone()));

        let concurrent_emit = 4; // number of worker threads

        let mut handles = vec![];
        // try send `concurrent_emit` events concurrently
        for _ in 0..concurrent_emit {
            let processor_clone = Arc::clone(&processor);
            let handle = tokio::spawn(async move {
                let mut record: SdkLogRecord = SdkLogRecord::new();
                let instrumentation: InstrumentationScope = Default::default();
                processor_clone.emit(&mut record, &instrumentation);
            });
            handles.push(handle);
        }

        // below code won't get executed
        for handle in handles {
            handle.await.unwrap();
        }
        assert_eq!(exporter.len(), concurrent_emit);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    // This test uses a multi-threaded runtime setup with a single worker thread. Note that even
    // though only one worker thread is created, it is distinct from the main thread. The processor
    // emits a log event, and the exporter performs an async operation that requires the runtime.
    // The single worker thread handles this operation without deadlocking, as long as no other
    // tasks occupy the runtime.
    async fn test_simple_processor_async_exporter_with_runtime() {
        let exporter = LogExporterThatRequiresTokio::new();
        let processor = SimpleLogProcessor::new(exporter.clone());

        let mut record: SdkLogRecord = SdkLogRecord::new();
        let instrumentation: InstrumentationScope = Default::default();

        processor.emit(&mut record, &instrumentation);

        assert_eq!(exporter.len(), 1);
    }

    #[tokio::test(flavor = "multi_thread")]
    // This test uses a multi-threaded runtime setup with the default number of worker threads.
    // The processor emits a log event, and the exporter, which requires the runtime for its async
    // operations, can access one of the available worker threads to complete its task. As there
    // are multiple threads, the exporter can proceed without blocking other tasks, ensuring the
    // test completes successfully.
    async fn test_simple_processor_async_exporter_with_multi_thread_runtime() {
        let exporter = LogExporterThatRequiresTokio::new();

        let processor = SimpleLogProcessor::new(exporter.clone());

        let mut record: SdkLogRecord = SdkLogRecord::new();
        let instrumentation: InstrumentationScope = Default::default();

        processor.emit(&mut record, &instrumentation);

        assert_eq!(exporter.len(), 1);
    }

    #[tokio::test(flavor = "current_thread")]
    #[ignore]
    // This test uses a current-thread runtime, where all operations run on the main thread.
    // The processor emits a log event while the runtime is blocked using `futures::block_on`
    // to complete the export operation. The exporter, which performs an async operation and
    // requires the runtime, cannot progress because the main thread is already blocked.
    // This results in a deadlock, as the runtime cannot move forward.
    async fn test_simple_processor_async_exporter_with_current_thread_runtime() {
        let exporter = LogExporterThatRequiresTokio::new();

        let processor = SimpleLogProcessor::new(exporter.clone());

        let mut record: SdkLogRecord = SdkLogRecord::new();
        let instrumentation: InstrumentationScope = Default::default();

        processor.emit(&mut record, &instrumentation);

        assert_eq!(exporter.len(), 1);
    }

    #[derive(Debug, Clone)]
    struct ReentrantLogExporter {
        logger: Arc<Mutex<Option<SdkLogger>>>,
    }

    impl ReentrantLogExporter {
        fn new() -> Self {
            Self {
                logger: Arc::new(Mutex::new(None)),
            }
        }

        fn set_logger(&self, logger: SdkLogger) {
            let mut guard = self.logger.lock().unwrap();
            *guard = Some(logger);
        }
    }

    impl LogExporter for ReentrantLogExporter {
        async fn export(&self, _batch: LogBatch<'_>) -> OTelSdkResult {
            let logger = self.logger.lock().unwrap();
            if let Some(logger) = logger.as_ref() {
                let mut log_record = logger.create_log_record();
                log_record.set_severity_number(opentelemetry::logs::Severity::Error);
                logger.emit(log_record);
            }

            Ok(())
        }
    }

    #[test]
    fn exporter_internal_log_does_not_deadlock_with_simple_processor() {
        // This tests that even when exporter produces logs while
        // exporting, it does not deadlock, as SimpleLogProcessor
        // activates SuppressGuard before calling the exporter.
        let exporter: ReentrantLogExporter = ReentrantLogExporter::new();
        let logger_provider = SdkLoggerProvider::builder()
            .with_simple_exporter(exporter.clone())
            .build();
        exporter.set_logger(logger_provider.logger("processor-logger"));

        let logger = logger_provider.logger("test-logger");
        let mut log_record = logger.create_log_record();
        log_record.set_severity_number(opentelemetry::logs::Severity::Error);
        logger.emit(log_record);
    }

    #[cfg(feature = "experimental_metrics_bound_instruments")]
    mod self_obs {
        use super::*;

        /// Sums the values of `otel.sdk.processor.log.processed` data points whose
        /// `error.type` attribute equals `error_type` (or that have no `error.type`
        /// attribute when `error_type` is `None`).
        #[cfg(feature = "experimental_metrics_bound_instruments")]
        fn sum_processed_log_records(
            metric_exporter: &crate::metrics::InMemoryMetricExporter,
            error_type: Option<&str>,
        ) -> u64 {
            use crate::metrics::data::{AggregatedMetrics, MetricData};

            let metrics = metric_exporter.get_finished_metrics().unwrap();
            let mut total: u64 = 0;
            for rm in &metrics {
                for sm in &rm.scope_metrics {
                    for metric in &sm.metrics {
                        if metric.name == "otel.sdk.processor.log.processed" {
                            if let AggregatedMetrics::U64(MetricData::Sum(sum)) = &metric.data {
                                for dp in sum.data_points() {
                                    let dp_error_type = dp
                                        .attributes()
                                        .find(|kv| kv.key.as_str() == "error.type")
                                        .map(|kv| kv.value.as_str().to_string());
                                    let matches = match error_type {
                                        Some(expected) => {
                                            dp_error_type.as_deref() == Some(expected)
                                        }
                                        None => dp_error_type.is_none(),
                                    };
                                    if matches {
                                        total += dp.value();
                                    }
                                }
                            }
                        }
                    }
                }
            }
            total
        }

        /// Verifies that `otel.sdk.processor.log.processed` counts each record the
        /// SimpleLogProcessor submits to the exporter (with no `error.type`),
        /// independent of the export outcome.
        ///
        /// `#[ignore]`d because it mutates process-wide state via
        /// `global::set_meter_provider()`. CI runs it in isolation via `test.sh`.
        #[cfg(feature = "experimental_metrics_bound_instruments")]
        #[test]
        #[ignore]
        fn self_diagnostics_counter_records_success() {
            use crate::logs::InMemoryLogExporterBuilder;
            use crate::metrics::{InMemoryMetricExporter, SdkMeterProvider};

            let metric_exporter = InMemoryMetricExporter::default();
            let meter_provider = SdkMeterProvider::builder()
                .with_periodic_exporter(metric_exporter.clone())
                .build();
            opentelemetry::global::set_meter_provider(meter_provider.clone());

            let log_exporter = InMemoryLogExporterBuilder::default().build();
            let processor = SimpleLogProcessor::new(log_exporter);

            let instrumentation = InstrumentationScope::default();
            for _ in 0..10 {
                let mut record = SdkLogRecord::new();
                processor.emit(&mut record, &instrumentation);
            }

            meter_provider.force_flush().unwrap();

            let processed = sum_processed_log_records(&metric_exporter, None);
            assert_eq!(processed, 10, "expected 10 processed logs, got {processed}");

            meter_provider.shutdown().unwrap();
        }

        /// Verifies that `otel.sdk.processor.log.processed` records post-shutdown
        /// emits with `error.type = already_shutdown`.
        ///
        /// `#[ignore]`d because it mutates process-wide state via
        /// `global::set_meter_provider()`. CI runs it in isolation via `test.sh`.
        #[cfg(feature = "experimental_metrics_bound_instruments")]
        #[test]
        #[ignore]
        fn self_diagnostics_counter_records_already_shutdown_drops() {
            use crate::logs::InMemoryLogExporterBuilder;
            use crate::metrics::{InMemoryMetricExporter, SdkMeterProvider};

            let metric_exporter = InMemoryMetricExporter::default();
            let meter_provider = SdkMeterProvider::builder()
                .with_periodic_exporter(metric_exporter.clone())
                .build();
            opentelemetry::global::set_meter_provider(meter_provider.clone());

            let log_exporter = InMemoryLogExporterBuilder::default().build();
            let processor = SimpleLogProcessor::new(log_exporter);

            // Shut the processor down; subsequent emits hit the already_shutdown branch.
            processor.shutdown().unwrap();

            let instrumentation = InstrumentationScope::default();
            for _ in 0..7 {
                let mut record = SdkLogRecord::new();
                processor.emit(&mut record, &instrumentation);
            }

            meter_provider.force_flush().unwrap();

            let already_shutdown =
                sum_processed_log_records(&metric_exporter, Some("already_shutdown"));
            assert_eq!(
                already_shutdown, 7,
                "expected 7 already_shutdown drops, got {already_shutdown}"
            );
            let success = sum_processed_log_records(&metric_exporter, None);
            assert_eq!(
                success, 0,
                "post-shutdown emits must not be counted as success, got {success}"
            );

            meter_provider.shutdown().unwrap();
        }
    }
}
