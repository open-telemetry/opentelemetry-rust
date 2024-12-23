//! # OpenTelemetry Span Processor Interface
//!
//! Span processor is an interface which allows hooks for span start and end method
//! invocations. The span processors are invoked only when
//! [`is_recording`] is true.
//!
//! Built-in span processors are responsible for batching and conversion of spans to
//! exportable representation and passing batches to exporters.
//!
//! Span processors can be registered directly on SDK [`TracerProvider`] and they are
//! invoked in the same order as they were registered.
//!
//! All `Tracer` instances created by a `TracerProvider` share the same span processors.
//! Changes to this collection reflect in all `Tracer` instances.
//!
//! The following diagram shows `SpanProcessor`'s relationship to other components
//! in the SDK:
//!
//! ```ascii
//!   +-----+--------------+   +-----------------------+   +-------------------+
//!   |     |              |   |                       |   |                   |
//!   |     |              |   | (Batch)SpanProcessor  |   |    SpanExporter   |
//!   |     |              +---> (Simple)SpanProcessor +--->  (OTLPExporter)   |
//!   |     |              |   |                       |   |                   |
//!   | SDK | Tracer.span()|   +-----------------------+   +-------------------+
//!   |     | Span.end()   |
//!   |     |              |
//!   |     |              |
//!   |     |              |
//!   |     |              |
//!   +-----+--------------+
//! ```
//!
//! [`is_recording`]: opentelemetry::trace::Span::is_recording()
//! [`TracerProvider`]: opentelemetry::trace::TracerProvider

use crate::export::trace::{SpanData, SpanExporter};
use crate::resource::Resource;
use crate::trace::Span;
use opentelemetry::otel_error;
use opentelemetry::{otel_debug, otel_warn};
use opentelemetry::{
    trace::{TraceError, TraceResult},
    Context,
};
use std::cmp::min;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::{env, str::FromStr, time::Duration};

use std::sync::atomic::AtomicBool;
use std::thread;
use std::time::Instant;

/// Delay interval between two consecutive exports.
pub(crate) const OTEL_BSP_SCHEDULE_DELAY: &str = "OTEL_BSP_SCHEDULE_DELAY";
/// Default delay interval between two consecutive exports.
pub(crate) const OTEL_BSP_SCHEDULE_DELAY_DEFAULT: u64 = 5_000;
/// Maximum queue size
pub(crate) const OTEL_BSP_MAX_QUEUE_SIZE: &str = "OTEL_BSP_MAX_QUEUE_SIZE";
/// Default maximum queue size
pub(crate) const OTEL_BSP_MAX_QUEUE_SIZE_DEFAULT: usize = 2_048;
/// Maximum batch size, must be less than or equal to OTEL_BSP_MAX_QUEUE_SIZE
pub(crate) const OTEL_BSP_MAX_EXPORT_BATCH_SIZE: &str = "OTEL_BSP_MAX_EXPORT_BATCH_SIZE";
/// Default maximum batch size
pub(crate) const OTEL_BSP_MAX_EXPORT_BATCH_SIZE_DEFAULT: usize = 512;
/// Maximum allowed time to export data.
pub(crate) const OTEL_BSP_EXPORT_TIMEOUT: &str = "OTEL_BSP_EXPORT_TIMEOUT";
/// Default maximum allowed time to export data.
pub(crate) const OTEL_BSP_EXPORT_TIMEOUT_DEFAULT: u64 = 30_000;
/// Environment variable to configure max concurrent exports for batch span
/// processor.
pub(crate) const OTEL_BSP_MAX_CONCURRENT_EXPORTS: &str = "OTEL_BSP_MAX_CONCURRENT_EXPORTS";
/// Default max concurrent exports for BSP
pub(crate) const OTEL_BSP_MAX_CONCURRENT_EXPORTS_DEFAULT: usize = 1;

/// `SpanProcessor` is an interface which allows hooks for span start and end
/// method invocations. The span processors are invoked only when is_recording
/// is true.
pub trait SpanProcessor: Send + Sync + std::fmt::Debug {
    /// `on_start` is called when a `Span` is started.  This method is called
    /// synchronously on the thread that started the span, therefore it should
    /// not block or throw exceptions.
    fn on_start(&self, span: &mut Span, cx: &Context);
    /// `on_end` is called after a `Span` is ended (i.e., the end timestamp is
    /// already set). This method is called synchronously within the `Span::end`
    /// API, therefore it should not block or throw an exception.
    fn on_end(&self, span: SpanData);
    /// Force the spans lying in the cache to be exported.
    fn force_flush(&self) -> TraceResult<()>;
    /// Shuts down the processor. Called when SDK is shut down. This is an
    /// opportunity for processors to do any cleanup required.
    ///
    /// Implementation should make sure shutdown can be called multiple times.
    fn shutdown(&self) -> TraceResult<()>;
    /// Set the resource for the span processor.
    fn set_resource(&mut self, _resource: &Resource) {}
}

/// A [SpanProcessor] that passes finished spans to the configured
/// `SpanExporter`, as soon as they are finished, without any batching. This is
/// typically useful for debugging and testing. For scenarios requiring higher
/// performance/throughput, consider using [BatchSpanProcessor].
#[derive(Debug)]
pub struct SimpleSpanProcessor {
    exporter: Mutex<Box<dyn SpanExporter>>,
}

impl SimpleSpanProcessor {
    /// Create a new [SimpleSpanProcessor] using the provided exporter.
    pub fn new(exporter: Box<dyn SpanExporter>) -> Self {
        Self {
            exporter: Mutex::new(exporter),
        }
    }
}

impl SpanProcessor for SimpleSpanProcessor {
    fn on_start(&self, _span: &mut Span, _cx: &Context) {
        // Ignored
    }

    fn on_end(&self, span: SpanData) {
        if !span.span_context.is_sampled() {
            return;
        }

        let result = self
            .exporter
            .lock()
            .map_err(|_| TraceError::Other("SimpleSpanProcessor mutex poison".into()))
            .and_then(|mut exporter| futures_executor::block_on(exporter.export(vec![span])));

        if let Err(err) = result {
            // TODO: check error type, and log `error` only if the error is user-actiobable, else log `debug`
            otel_debug!(
                name: "SimpleProcessor.OnEnd.Error",
                reason = format!("{:?}", err)
            );
        }
    }

    fn force_flush(&self) -> TraceResult<()> {
        // Nothing to flush for simple span processor.
        Ok(())
    }

    fn shutdown(&self) -> TraceResult<()> {
        if let Ok(mut exporter) = self.exporter.lock() {
            exporter.shutdown();
            Ok(())
        } else {
            Err(TraceError::Other(
                "SimpleSpanProcessor mutex poison at shutdown".into(),
            ))
        }
    }

    fn set_resource(&mut self, resource: &Resource) {
        if let Ok(mut exporter) = self.exporter.lock() {
            exporter.set_resource(resource);
        }
    }
}

/// The `BatchSpanProcessor` collects finished spans in a buffer and exports them
/// in batches to the configured `SpanExporter`. This processor is ideal for
/// high-throughput environments, as it minimizes the overhead of exporting spans
/// individually. It uses a **dedicated background thread** to manage and export spans
/// asynchronously, ensuring that the application's main execution flow is not blocked.
///
/// /// # Example
///
/// This example demonstrates how to configure and use the `BatchSpanProcessor`
/// with a custom configuration. Note that a dedicated thread is used internally
/// to manage the export process.
///
/// ```rust
/// use opentelemetry::global;
/// use opentelemetry_sdk::{
///     trace::{BatchSpanProcessor, BatchConfigBuilder, TracerProvider},
///     runtime,
///     testing::trace::NoopSpanExporter,
/// };
/// use opentelemetry::trace::Tracer as _;
/// use opentelemetry::trace::Span;
/// use std::time::Duration;
///
/// fn main() {
///     // Step 1: Create an exporter (e.g., a No-Op Exporter for demonstration).
///     let exporter = NoopSpanExporter::new();
///
///     // Step 2: Configure the BatchSpanProcessor.
///     let batch_processor = BatchSpanProcessor::builder(exporter)
///         .with_batch_config(
///             BatchConfigBuilder::default()
///                 .with_max_queue_size(1024) // Buffer up to 1024 spans.
///                 .with_max_export_batch_size(256) // Export in batches of up to 256 spans.
///                 .with_scheduled_delay(Duration::from_secs(5)) // Export every 5 seconds.
///                 .with_max_export_timeout(Duration::from_secs(10)) // Timeout after 10 seconds.
///                 .build(),
///         )
///         .build();
///
///     // Step 3: Set up a TracerProvider with the configured processor.
///     let provider = TracerProvider::builder()
///         .with_span_processor(batch_processor)
///         .build();
///     global::set_tracer_provider(provider.clone());
///
///     // Step 4: Create spans and record operations.
///     let tracer = global::tracer("example-tracer");
///     let mut span = tracer.start("example-span");
///     span.end(); // Mark the span as completed.
///
///     // Step 5: Ensure all spans are flushed before exiting.
///     provider.shutdown();
/// }
/// ```
use futures_executor::block_on;
use std::sync::mpsc::sync_channel;
use std::sync::mpsc::RecvTimeoutError;
use std::sync::mpsc::SyncSender;

/// Messages exchanged between the main thread and the background thread.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
enum BatchMessage {
    ExportSpan(SpanData),
    ForceFlush(SyncSender<TraceResult<()>>),
    Shutdown(SyncSender<TraceResult<()>>),
    SetResource(Arc<Resource>),
}

/// A batch span processor with a dedicated background thread.
#[derive(Debug)]
pub struct BatchSpanProcessor {
    message_sender: SyncSender<BatchMessage>,
    handle: Mutex<Option<thread::JoinHandle<()>>>,
    forceflush_timeout: Duration,
    shutdown_timeout: Duration,
    is_shutdown: AtomicBool,
    dropped_span_count: Arc<AtomicUsize>,
}

impl BatchSpanProcessor {
    /// Creates a new instance of `BatchSpanProcessor`.
    pub fn new<E>(
        mut exporter: E,
        config: BatchConfig,
        //max_queue_size: usize,
        //scheduled_delay: Duration,
        //shutdown_timeout: Duration,
    ) -> Self
    where
        E: SpanExporter + Send + 'static,
    {
        let (message_sender, message_receiver) = sync_channel(config.max_queue_size);

        let handle = thread::Builder::new()
            .name("BatchSpanProcessorThread".to_string())
            .spawn(move || {
                let mut spans = Vec::with_capacity(config.max_export_batch_size);
                let mut last_export_time = Instant::now();

                loop {
                    let remaining_time_option = config
                        .scheduled_delay
                        .checked_sub(last_export_time.elapsed());
                    let remaining_time = match remaining_time_option {
                        Some(remaining_time) => remaining_time,
                        None => config.scheduled_delay,
                    };
                    match message_receiver.recv_timeout(remaining_time) {
                        Ok(message) => match message {
                            BatchMessage::ExportSpan(span) => {
                                spans.push(span);
                                if spans.len() >= config.max_queue_size
                                    || last_export_time.elapsed() >= config.scheduled_delay
                                {
                                    if let Err(err) = block_on(exporter.export(spans.split_off(0)))
                                    {
                                        otel_error!(
                                            name: "BatchSpanProcessor.ExportError",
                                            error = format!("{}", err)
                                        );
                                    }
                                    last_export_time = Instant::now();
                                }
                            }
                            BatchMessage::ForceFlush(sender) => {
                                let result = block_on(exporter.export(spans.split_off(0)));
                                let _ = sender.send(result);
                            }
                            BatchMessage::Shutdown(sender) => {
                                let result = block_on(exporter.export(spans.split_off(0)));
                                let _ = sender.send(result);
                                break;
                            }
                            BatchMessage::SetResource(resource) => {
                                exporter.set_resource(&resource);
                            }
                        },
                        Err(RecvTimeoutError::Timeout) => {
                            if last_export_time.elapsed() >= config.scheduled_delay {
                                if let Err(err) = block_on(exporter.export(spans.split_off(0))) {
                                    otel_error!(
                                        name: "BatchSpanProcessor.ExportError",
                                        error = format!("{}", err)
                                    );
                                }
                                last_export_time = Instant::now();
                            }
                        }
                        Err(RecvTimeoutError::Disconnected) => {
                            otel_error!(
                                name: "BatchSpanProcessor.InternalError.ChannelDisconnected",
                                message = "Channel disconnected, shutting down processor thread."
                            );
                            break;
                        }
                    }
                }
            })
            .expect("Failed to spawn thread"); //TODO: Handle thread spawn failure

        Self {
            message_sender,
            handle: Mutex::new(Some(handle)),
            forceflush_timeout: Duration::from_secs(5), // TODO: make this configurable
            shutdown_timeout: Duration::from_secs(5),   // TODO: make this configurable
            is_shutdown: AtomicBool::new(false),
            dropped_span_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// builder
    pub fn builder<E>(exporter: E) -> BatchSpanProcessorBuilder<E>
    where
        E: SpanExporter + Send + 'static,
    {
        BatchSpanProcessorBuilder {
            exporter,
            config: BatchConfig::default(),
        }
    }
}

impl SpanProcessor for BatchSpanProcessor {
    /// Handles span start.
    fn on_start(&self, _span: &mut Span, _cx: &Context) {
        // Ignored
    }

    /// Handles span end.
    fn on_end(&self, span: SpanData) {
        if self.is_shutdown.load(Ordering::Relaxed) {
            // this is a warning, as the user is trying to emit after the processor has been shutdown
            otel_warn!(
                name: "BatchSpanProcessor.Emit.ProcessorShutdown",
            );
            return;
        }
        let result = self.message_sender.try_send(BatchMessage::ExportSpan(span));

        // TODO - Implement throttling to prevent error flooding when the queue is full or closed.
        if result.is_err() {
            // Increment dropped span count. The first time we have to drop a span,
            // emit a warning.
            if self.dropped_span_count.fetch_add(1, Ordering::Relaxed) == 0 {
                otel_warn!(name: "BatchSpanProcessorDedicatedThread.SpanDroppingStarted",
                    message = "BatchSpanProcessorDedicatedThread dropped a Span due to queue full/internal errors. No further span will be emitted for further drops until Shutdown. During Shutdown time, a log will be emitted with exact count of total logs dropped.");
            }
        }
    }

    /// Flushes all pending spans.
    fn force_flush(&self) -> TraceResult<()> {
        if self.is_shutdown.load(Ordering::Relaxed) {
            return Err(TraceError::Other("Processor already shutdown".into()));
        }
        let (sender, receiver) = sync_channel(1);
        self.message_sender
            .try_send(BatchMessage::ForceFlush(sender))
            .map_err(|_| TraceError::Other("Failed to send ForceFlush message".into()))?;

        receiver
            .recv_timeout(self.forceflush_timeout)
            .map_err(|_| TraceError::ExportTimedOut(self.forceflush_timeout))?
    }

    /// Shuts down the processor.
    fn shutdown(&self) -> TraceResult<()> {
        let dropped_spans = self.dropped_span_count.load(Ordering::Relaxed);
        if dropped_spans > 0 {
            otel_warn!(
                name: "BatchSpanProcessor.LogsDropped",
                dropped_span_count = dropped_spans,
                message = "Spans were dropped due to a queue being full or other error. The count represents the total count of spans dropped in the lifetime of this BatchSpanProcessor. Consider increasing the queue size and/or decrease delay between intervals."
            );
        }
        if self.is_shutdown.swap(true, Ordering::Relaxed) {
            return Err(TraceError::Other("Processor already shutdown".into()));
        }
        let (sender, receiver) = sync_channel(1);
        self.message_sender
            .try_send(BatchMessage::Shutdown(sender))
            .map_err(|_| TraceError::Other("Failed to send Shutdown message".into()))?;

        let result = receiver
            .recv_timeout(self.shutdown_timeout)
            .map_err(|_| TraceError::ExportTimedOut(self.shutdown_timeout))?;
        if let Some(handle) = self.handle.lock().unwrap().take() {
            if let Err(err) = handle.join() {
                return Err(TraceError::Other(format!(
                    "Background thread failed to join during shutdown. This may indicate a panic or unexpected termination: {:?}",
                    err
                ).into()));
            }
        }
        result
    }

    /// Set the resource for the processor.
    fn set_resource(&mut self, resource: &Resource) {
        let resource = Arc::new(resource.clone());
        let _ = self
            .message_sender
            .try_send(BatchMessage::SetResource(resource));
    }
}

/// Builder for `BatchSpanProcessorDedicatedThread`.
#[derive(Debug, Default)]
pub struct BatchSpanProcessorBuilder<E>
where
    E: SpanExporter + Send + 'static,
{
    exporter: E,
    config: BatchConfig,
}

impl<E> BatchSpanProcessorBuilder<E>
where
    E: SpanExporter + Send + 'static,
{
    /// Set the BatchConfig for [BatchSpanProcessorBuilder]
    pub fn with_batch_config(self, config: BatchConfig) -> Self {
        BatchSpanProcessorBuilder { config, ..self }
    }

    /// Build a new instance of `BatchSpanProcessor`.
    pub fn build(self) -> BatchSpanProcessor {
        BatchSpanProcessor::new(self.exporter, self.config)
    }
}

/// Batch span processor configuration.
/// Use [`BatchConfigBuilder`] to configure your own instance of [`BatchConfig`].
#[derive(Debug)]
pub struct BatchConfig {
    /// The maximum queue size to buffer spans for delayed processing. If the
    /// queue gets full it drops the spans. The default value of is 2048.
    pub(crate) max_queue_size: usize,

    /// The delay interval in milliseconds between two consecutive processing
    /// of batches. The default value is 5 seconds.
    pub(crate) scheduled_delay: Duration,

    #[allow(dead_code)]
    /// The maximum number of spans to process in a single batch. If there are
    /// more than one batch worth of spans then it processes multiple batches
    /// of spans one batch after the other without any delay. The default value
    /// is 512.
    pub(crate) max_export_batch_size: usize,

    #[allow(dead_code)]
    /// The maximum duration to export a batch of data.
    pub(crate) max_export_timeout: Duration,

    #[allow(dead_code)]
    /// Maximum number of concurrent exports
    ///
    /// Limits the number of spawned tasks for exports and thus memory consumed
    /// by an exporter. A value of 1 will cause exports to be performed
    /// synchronously on the BatchSpanProcessor task.
    pub(crate) max_concurrent_exports: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        BatchConfigBuilder::default().build()
    }
}

/// A builder for creating [`BatchConfig`] instances.
#[derive(Debug)]
pub struct BatchConfigBuilder {
    max_queue_size: usize,
    scheduled_delay: Duration,
    max_export_batch_size: usize,
    max_export_timeout: Duration,
    max_concurrent_exports: usize,
}

impl Default for BatchConfigBuilder {
    /// Create a new [`BatchConfigBuilder`] initialized with default batch config values as per the specs.
    /// The values are overriden by environment variables if set.
    /// The supported environment variables are:
    /// * `OTEL_BSP_MAX_QUEUE_SIZE`
    /// * `OTEL_BSP_SCHEDULE_DELAY`
    /// * `OTEL_BSP_MAX_EXPORT_BATCH_SIZE`
    /// * `OTEL_BSP_EXPORT_TIMEOUT`
    /// * `OTEL_BSP_MAX_CONCURRENT_EXPORTS`
    fn default() -> Self {
        BatchConfigBuilder {
            max_queue_size: OTEL_BSP_MAX_QUEUE_SIZE_DEFAULT,
            scheduled_delay: Duration::from_millis(OTEL_BSP_SCHEDULE_DELAY_DEFAULT),
            max_export_batch_size: OTEL_BSP_MAX_EXPORT_BATCH_SIZE_DEFAULT,
            max_export_timeout: Duration::from_millis(OTEL_BSP_EXPORT_TIMEOUT_DEFAULT),
            max_concurrent_exports: OTEL_BSP_MAX_CONCURRENT_EXPORTS_DEFAULT,
        }
        .init_from_env_vars()
    }
}

impl BatchConfigBuilder {
    /// Set max_queue_size for [`BatchConfigBuilder`].
    /// It's the maximum queue size to buffer spans for delayed processing.
    /// If the queue gets full it will drops the spans.
    /// The default value of is 2048.
    pub fn with_max_queue_size(mut self, max_queue_size: usize) -> Self {
        self.max_queue_size = max_queue_size;
        self
    }

    /// Set max_export_batch_size for [`BatchConfigBuilder`].
    /// It's the maximum number of spans to process in a single batch. If there are
    /// more than one batch worth of spans then it processes multiple batches
    /// of spans one batch after the other without any delay. The default value
    /// is 512.
    pub fn with_max_export_batch_size(mut self, max_export_batch_size: usize) -> Self {
        self.max_export_batch_size = max_export_batch_size;
        self
    }

    /// Set max_concurrent_exports for [`BatchConfigBuilder`].
    /// It's the maximum number of concurrent exports.
    /// Limits the number of spawned tasks for exports and thus memory consumed by an exporter.
    /// The default value is 1.
    /// IF the max_concurrent_exports value is default value, it will cause exports to be performed
    /// synchronously on the BatchSpanProcessor task.
    pub fn with_max_concurrent_exports(mut self, max_concurrent_exports: usize) -> Self {
        self.max_concurrent_exports = max_concurrent_exports;
        self
    }

    /// Set scheduled_delay_duration for [`BatchConfigBuilder`].
    /// It's the delay interval in milliseconds between two consecutive processing of batches.
    /// The default value is 5000 milliseconds.
    pub fn with_scheduled_delay(mut self, scheduled_delay: Duration) -> Self {
        self.scheduled_delay = scheduled_delay;
        self
    }

    /// Set max_export_timeout for [`BatchConfigBuilder`].
    /// It's the maximum duration to export a batch of data.
    /// The The default value is 30000 milliseconds.
    pub fn with_max_export_timeout(mut self, max_export_timeout: Duration) -> Self {
        self.max_export_timeout = max_export_timeout;
        self
    }

    /// Builds a `BatchConfig` enforcing the following invariants:
    /// * `max_export_batch_size` must be less than or equal to `max_queue_size`.
    pub fn build(self) -> BatchConfig {
        // max export batch size must be less or equal to max queue size.
        // we set max export batch size to max queue size if it's larger than max queue size.
        let max_export_batch_size = min(self.max_export_batch_size, self.max_queue_size);

        BatchConfig {
            max_queue_size: self.max_queue_size,
            scheduled_delay: self.scheduled_delay,
            max_export_timeout: self.max_export_timeout,
            max_concurrent_exports: self.max_concurrent_exports,
            max_export_batch_size,
        }
    }

    fn init_from_env_vars(mut self) -> Self {
        if let Some(max_concurrent_exports) = env::var(OTEL_BSP_MAX_CONCURRENT_EXPORTS)
            .ok()
            .and_then(|max_concurrent_exports| usize::from_str(&max_concurrent_exports).ok())
        {
            self.max_concurrent_exports = max_concurrent_exports;
        }

        if let Some(max_queue_size) = env::var(OTEL_BSP_MAX_QUEUE_SIZE)
            .ok()
            .and_then(|queue_size| usize::from_str(&queue_size).ok())
        {
            self.max_queue_size = max_queue_size;
        }

        if let Some(scheduled_delay) = env::var(OTEL_BSP_SCHEDULE_DELAY)
            .ok()
            .and_then(|delay| u64::from_str(&delay).ok())
        {
            self.scheduled_delay = Duration::from_millis(scheduled_delay);
        }

        if let Some(max_export_batch_size) = env::var(OTEL_BSP_MAX_EXPORT_BATCH_SIZE)
            .ok()
            .and_then(|batch_size| usize::from_str(&batch_size).ok())
        {
            self.max_export_batch_size = max_export_batch_size;
        }

        // max export batch size must be less or equal to max queue size.
        // we set max export batch size to max queue size if it's larger than max queue size.
        if self.max_export_batch_size > self.max_queue_size {
            self.max_export_batch_size = self.max_queue_size;
        }

        if let Some(max_export_timeout) = env::var(OTEL_BSP_EXPORT_TIMEOUT)
            .ok()
            .and_then(|timeout| u64::from_str(&timeout).ok())
        {
            self.max_export_timeout = Duration::from_millis(max_export_timeout);
        }

        self
    }
}

#[cfg(all(test, feature = "testing", feature = "trace"))]
mod tests {
    // cargo test trace::span_processor::tests:: --features=testing
    use super::{
        BatchSpanProcessor, SimpleSpanProcessor, SpanProcessor, OTEL_BSP_EXPORT_TIMEOUT,
        OTEL_BSP_MAX_EXPORT_BATCH_SIZE, OTEL_BSP_MAX_QUEUE_SIZE, OTEL_BSP_MAX_QUEUE_SIZE_DEFAULT,
        OTEL_BSP_SCHEDULE_DELAY, OTEL_BSP_SCHEDULE_DELAY_DEFAULT,
    };
    use crate::export::trace::{ExportResult, SpanData, SpanExporter};
    use crate::testing::trace::{new_test_export_span_data, InMemorySpanExporterBuilder};
    use crate::trace::span_processor::{
        OTEL_BSP_EXPORT_TIMEOUT_DEFAULT, OTEL_BSP_MAX_CONCURRENT_EXPORTS,
        OTEL_BSP_MAX_CONCURRENT_EXPORTS_DEFAULT, OTEL_BSP_MAX_EXPORT_BATCH_SIZE_DEFAULT,
    };
    use crate::trace::{BatchConfig, BatchConfigBuilder, SpanEvents, SpanLinks};
    use opentelemetry::trace::{SpanContext, SpanId, SpanKind, Status};
    use std::fmt::Debug;
    use std::time::Duration;

    #[test]
    fn simple_span_processor_on_end_calls_export() {
        let exporter = InMemorySpanExporterBuilder::new().build();
        let processor = SimpleSpanProcessor::new(Box::new(exporter.clone()));
        let span_data = new_test_export_span_data();
        processor.on_end(span_data.clone());
        assert_eq!(exporter.get_finished_spans().unwrap()[0], span_data);
        let _result = processor.shutdown();
    }

    #[test]
    fn simple_span_processor_on_end_skips_export_if_not_sampled() {
        let exporter = InMemorySpanExporterBuilder::new().build();
        let processor = SimpleSpanProcessor::new(Box::new(exporter.clone()));
        let unsampled = SpanData {
            span_context: SpanContext::empty_context(),
            parent_span_id: SpanId::INVALID,
            span_kind: SpanKind::Internal,
            name: "opentelemetry".into(),
            start_time: opentelemetry::time::now(),
            end_time: opentelemetry::time::now(),
            attributes: Vec::new(),
            dropped_attributes_count: 0,
            events: SpanEvents::default(),
            links: SpanLinks::default(),
            status: Status::Unset,
            instrumentation_scope: Default::default(),
        };
        processor.on_end(unsampled);
        assert!(exporter.get_finished_spans().unwrap().is_empty());
    }

    #[test]
    fn simple_span_processor_shutdown_calls_shutdown() {
        let exporter = InMemorySpanExporterBuilder::new().build();
        let processor = SimpleSpanProcessor::new(Box::new(exporter.clone()));
        let span_data = new_test_export_span_data();
        processor.on_end(span_data.clone());
        assert!(!exporter.get_finished_spans().unwrap().is_empty());
        let _result = processor.shutdown();
        // Assume shutdown is called by ensuring spans are empty in the exporter
        assert!(exporter.get_finished_spans().unwrap().is_empty());
    }

    #[test]
    fn test_default_const_values() {
        assert_eq!(OTEL_BSP_MAX_QUEUE_SIZE, "OTEL_BSP_MAX_QUEUE_SIZE");
        assert_eq!(OTEL_BSP_MAX_QUEUE_SIZE_DEFAULT, 2048);
        assert_eq!(OTEL_BSP_SCHEDULE_DELAY, "OTEL_BSP_SCHEDULE_DELAY");
        assert_eq!(OTEL_BSP_SCHEDULE_DELAY_DEFAULT, 5000);
        assert_eq!(
            OTEL_BSP_MAX_EXPORT_BATCH_SIZE,
            "OTEL_BSP_MAX_EXPORT_BATCH_SIZE"
        );
        assert_eq!(OTEL_BSP_MAX_EXPORT_BATCH_SIZE_DEFAULT, 512);
        assert_eq!(OTEL_BSP_EXPORT_TIMEOUT, "OTEL_BSP_EXPORT_TIMEOUT");
        assert_eq!(OTEL_BSP_EXPORT_TIMEOUT_DEFAULT, 30000);
    }

    #[test]
    fn test_default_batch_config_adheres_to_specification() {
        let env_vars = vec![
            OTEL_BSP_SCHEDULE_DELAY,
            OTEL_BSP_EXPORT_TIMEOUT,
            OTEL_BSP_MAX_QUEUE_SIZE,
            OTEL_BSP_MAX_EXPORT_BATCH_SIZE,
            OTEL_BSP_MAX_CONCURRENT_EXPORTS,
        ];

        let config = temp_env::with_vars_unset(env_vars, BatchConfig::default);

        assert_eq!(
            config.max_concurrent_exports,
            OTEL_BSP_MAX_CONCURRENT_EXPORTS_DEFAULT
        );
        assert_eq!(
            config.scheduled_delay,
            Duration::from_millis(OTEL_BSP_SCHEDULE_DELAY_DEFAULT)
        );
        assert_eq!(
            config.max_export_timeout,
            Duration::from_millis(OTEL_BSP_EXPORT_TIMEOUT_DEFAULT)
        );
        assert_eq!(config.max_queue_size, OTEL_BSP_MAX_QUEUE_SIZE_DEFAULT);
        assert_eq!(
            config.max_export_batch_size,
            OTEL_BSP_MAX_EXPORT_BATCH_SIZE_DEFAULT
        );
    }

    #[test]
    fn test_batch_config_configurable_by_env_vars() {
        let env_vars = vec![
            (OTEL_BSP_SCHEDULE_DELAY, Some("2000")),
            (OTEL_BSP_EXPORT_TIMEOUT, Some("60000")),
            (OTEL_BSP_MAX_QUEUE_SIZE, Some("4096")),
            (OTEL_BSP_MAX_EXPORT_BATCH_SIZE, Some("1024")),
        ];

        let config = temp_env::with_vars(env_vars, BatchConfig::default);

        assert_eq!(config.scheduled_delay, Duration::from_millis(2000));
        assert_eq!(config.max_export_timeout, Duration::from_millis(60000));
        assert_eq!(config.max_queue_size, 4096);
        assert_eq!(config.max_export_batch_size, 1024);
    }

    #[test]
    fn test_batch_config_max_export_batch_size_validation() {
        let env_vars = vec![
            (OTEL_BSP_MAX_QUEUE_SIZE, Some("256")),
            (OTEL_BSP_MAX_EXPORT_BATCH_SIZE, Some("1024")),
        ];

        let config = temp_env::with_vars(env_vars, BatchConfig::default);

        assert_eq!(config.max_queue_size, 256);
        assert_eq!(config.max_export_batch_size, 256);
        assert_eq!(
            config.scheduled_delay,
            Duration::from_millis(OTEL_BSP_SCHEDULE_DELAY_DEFAULT)
        );
        assert_eq!(
            config.max_export_timeout,
            Duration::from_millis(OTEL_BSP_EXPORT_TIMEOUT_DEFAULT)
        );
    }

    #[test]
    fn test_batch_config_with_fields() {
        let batch = BatchConfigBuilder::default()
            .with_max_export_batch_size(10)
            .with_scheduled_delay(Duration::from_millis(10))
            .with_max_export_timeout(Duration::from_millis(10))
            .with_max_concurrent_exports(10)
            .with_max_queue_size(10)
            .build();
        assert_eq!(batch.max_export_batch_size, 10);
        assert_eq!(batch.scheduled_delay, Duration::from_millis(10));
        assert_eq!(batch.max_export_timeout, Duration::from_millis(10));
        assert_eq!(batch.max_concurrent_exports, 10);
        assert_eq!(batch.max_queue_size, 10);
    }

    // Helper function to create a default test span
    fn create_test_span(name: &str) -> SpanData {
        SpanData {
            span_context: SpanContext::empty_context(),
            parent_span_id: SpanId::INVALID,
            span_kind: SpanKind::Internal,
            name: name.to_string().into(),
            start_time: opentelemetry::time::now(),
            end_time: opentelemetry::time::now(),
            attributes: Vec::new(),
            dropped_attributes_count: 0,
            events: SpanEvents::default(),
            links: SpanLinks::default(),
            status: Status::Unset,
            instrumentation_scope: Default::default(),
        }
    }

    use crate::Resource;
    use futures_util::future::BoxFuture;
    use futures_util::FutureExt;
    use opentelemetry::{Key, KeyValue, Value};
    use std::sync::{atomic::Ordering, Arc, Mutex};

    // Mock exporter to test functionality
    #[derive(Debug)]
    struct MockSpanExporter {
        exported_spans: Arc<Mutex<Vec<SpanData>>>,
        exported_resource: Arc<Mutex<Option<Resource>>>,
    }

    impl MockSpanExporter {
        fn new() -> Self {
            Self {
                exported_spans: Arc::new(Mutex::new(Vec::new())),
                exported_resource: Arc::new(Mutex::new(None)),
            }
        }
    }

    impl SpanExporter for MockSpanExporter {
        fn export(&mut self, batch: Vec<SpanData>) -> BoxFuture<'static, ExportResult> {
            let exported_spans = self.exported_spans.clone();
            async move {
                exported_spans.lock().unwrap().extend(batch);
                Ok(())
            }
            .boxed()
        }

        fn shutdown(&mut self) {}
        fn set_resource(&mut self, resource: &Resource) {
            let mut exported_resource = self.exported_resource.lock().unwrap();
            *exported_resource = Some(resource.clone());
        }
    }

    #[test]
    fn batchspanprocessor_handles_on_end() {
        let exporter = MockSpanExporter::new();
        let exporter_shared = exporter.exported_spans.clone();
        let config = BatchConfigBuilder::default()
            .with_max_queue_size(10)
            .with_max_export_batch_size(10)
            .with_scheduled_delay(Duration::from_secs(5))
            .with_max_export_timeout(Duration::from_secs(2))
            .build();
        let processor = BatchSpanProcessor::new(exporter, config);

        let test_span = create_test_span("test_span");
        processor.on_end(test_span.clone());

        // Wait for flush interval to ensure the span is processed
        std::thread::sleep(Duration::from_secs(6));

        let exported_spans = exporter_shared.lock().unwrap();
        assert_eq!(exported_spans.len(), 1);
        assert_eq!(exported_spans[0].name, "test_span");
    }

    #[test]
    fn batchspanprocessor_force_flush() {
        let exporter = MockSpanExporter::new();
        let exporter_shared = exporter.exported_spans.clone(); // Shared access to verify exported spans
        let config = BatchConfigBuilder::default()
            .with_max_queue_size(10)
            .with_max_export_batch_size(10)
            .with_scheduled_delay(Duration::from_secs(5))
            .with_max_export_timeout(Duration::from_secs(2))
            .build();
        let processor = BatchSpanProcessor::new(exporter, config);

        // Create a test span and send it to the processor
        let test_span = create_test_span("force_flush_span");
        processor.on_end(test_span.clone());

        // Call force_flush to immediately export the spans
        let flush_result = processor.force_flush();
        assert!(flush_result.is_ok(), "Force flush failed unexpectedly");

        // Verify the exported spans in the mock exporter
        let exported_spans = exporter_shared.lock().unwrap();
        assert_eq!(
            exported_spans.len(),
            1,
            "Unexpected number of exported spans"
        );
        assert_eq!(exported_spans[0].name, "force_flush_span");
    }

    #[test]
    fn batchspanprocessor_shutdown() {
        let exporter = MockSpanExporter::new();
        let exporter_shared = exporter.exported_spans.clone(); // Shared access to verify exported spans
        let config = BatchConfigBuilder::default()
            .with_max_queue_size(10)
            .with_max_export_batch_size(10)
            .with_scheduled_delay(Duration::from_secs(5))
            .with_max_export_timeout(Duration::from_secs(2))
            .build();
        let processor = BatchSpanProcessor::new(exporter, config);

        // Create a test span and send it to the processor
        let test_span = create_test_span("shutdown_span");
        processor.on_end(test_span.clone());

        // Call shutdown to flush and export all pending spans
        let shutdown_result = processor.shutdown();
        assert!(shutdown_result.is_ok(), "Shutdown failed unexpectedly");

        // Verify the exported spans in the mock exporter
        let exported_spans = exporter_shared.lock().unwrap();
        assert_eq!(
            exported_spans.len(),
            1,
            "Unexpected number of exported spans"
        );
        assert_eq!(exported_spans[0].name, "shutdown_span");

        // Ensure further calls to shutdown are idempotent
        let second_shutdown_result = processor.shutdown();
        assert!(
            second_shutdown_result.is_err(),
            "Shutdown should fail when called a second time"
        );
    }

    #[test]
    fn batchspanprocessor_handles_dropped_spans() {
        let exporter = MockSpanExporter::new();
        let exporter_shared = exporter.exported_spans.clone(); // Shared access to verify exported spans
        let config = BatchConfigBuilder::default()
            .with_max_queue_size(2) // Small queue size to test span dropping
            .with_scheduled_delay(Duration::from_secs(5))
            .with_max_export_timeout(Duration::from_secs(2))
            .build();
        let processor = BatchSpanProcessor::new(exporter, config);

        // Create test spans and send them to the processor
        let span1 = create_test_span("span1");
        let span2 = create_test_span("span2");
        let span3 = create_test_span("span3"); // This span should be dropped

        processor.on_end(span1.clone());
        processor.on_end(span2.clone());
        processor.on_end(span3.clone()); // This span exceeds the queue size

        // Wait for the scheduled delay to expire
        std::thread::sleep(Duration::from_secs(3));

        let exported_spans = exporter_shared.lock().unwrap();

        // Verify that only the first two spans are exported
        assert_eq!(
            exported_spans.len(),
            2,
            "Unexpected number of exported spans"
        );
        assert!(exported_spans.iter().any(|s| s.name == "span1"));
        assert!(exported_spans.iter().any(|s| s.name == "span2"));

        // Ensure the third span is dropped
        assert!(
            !exported_spans.iter().any(|s| s.name == "span3"),
            "Span3 should have been dropped"
        );

        // Verify dropped spans count (if accessible in your implementation)
        let dropped_count = processor.dropped_span_count.load(Ordering::Relaxed);
        assert_eq!(dropped_count, 1, "Unexpected number of dropped spans");
    }

    #[test]
    fn validate_span_attributes_exported_correctly() {
        let exporter = MockSpanExporter::new();
        let exporter_shared = exporter.exported_spans.clone();
        let config = BatchConfigBuilder::default().build();
        let processor = BatchSpanProcessor::new(exporter, config);

        // Create a span with attributes
        let mut span_data = create_test_span("attribute_validation");
        span_data.attributes = vec![
            KeyValue::new("key1", "value1"),
            KeyValue::new("key2", "value2"),
        ];
        processor.on_end(span_data.clone());

        // Force flush to export the span
        let _ = processor.force_flush();

        // Validate the exported attributes
        let exported_spans = exporter_shared.lock().unwrap();
        assert_eq!(exported_spans.len(), 1);
        let exported_span = &exported_spans[0];
        assert!(exported_span
            .attributes
            .contains(&KeyValue::new("key1", "value1")));
        assert!(exported_span
            .attributes
            .contains(&KeyValue::new("key2", "value2")));
    }

    #[test]
    fn batchspanprocessor_sets_and_exports_with_resource() {
        let exporter = MockSpanExporter::new();
        let exporter_shared = exporter.exported_spans.clone();
        let resource_shared = exporter.exported_resource.clone();
        let config = BatchConfigBuilder::default().build();
        let mut processor = BatchSpanProcessor::new(exporter, config);

        // Set a resource for the processor
        let resource = Resource::new(vec![KeyValue::new("service.name", "test_service")]);
        processor.set_resource(&resource);

        // Create a span and send it to the processor
        let test_span = create_test_span("resource_test");
        processor.on_end(test_span.clone());

        // Force flush to ensure the span is exported
        let _ = processor.force_flush();

        // Validate spans are exported
        let exported_spans = exporter_shared.lock().unwrap();
        assert_eq!(exported_spans.len(), 1);

        // Validate the resource is correctly set in the exporter
        let exported_resource = resource_shared.lock().unwrap();
        assert!(exported_resource.is_some());
        assert_eq!(
            exported_resource
                .as_ref()
                .unwrap()
                .get(Key::new("service.name")),
            Some(Value::from("test_service"))
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_batch_processor_current_thread_runtime() {
        let exporter = MockSpanExporter::new();
        let exporter_shared = exporter.exported_spans.clone();

        let config = BatchConfigBuilder::default()
            .with_max_queue_size(5)
            .with_max_export_batch_size(3)
            .with_scheduled_delay(Duration::from_millis(50))
            .build();

        let processor = BatchSpanProcessor::new(exporter, config);

        for _ in 0..4 {
            let span = new_test_export_span_data();
            processor.on_end(span);
        }

        tokio::time::sleep(Duration::from_millis(200)).await;

        let exported_spans = exporter_shared.lock().unwrap();
        assert_eq!(exported_spans.len(), 4);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_batch_processor_multi_thread_count_1_runtime() {
        let exporter = MockSpanExporter::new();
        let exporter_shared = exporter.exported_spans.clone();

        let config = BatchConfigBuilder::default()
            .with_max_queue_size(5)
            .with_max_export_batch_size(3)
            .with_scheduled_delay(Duration::from_millis(50))
            .build();

        let processor = BatchSpanProcessor::new(exporter, config);

        for _ in 0..4 {
            let span = new_test_export_span_data();
            processor.on_end(span);
        }

        tokio::time::sleep(Duration::from_millis(200)).await;

        let exported_spans = exporter_shared.lock().unwrap();
        assert_eq!(exported_spans.len(), 4);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_batch_processor_multi_thread() {
        let exporter = MockSpanExporter::new();
        let exporter_shared = exporter.exported_spans.clone();

        let config = BatchConfigBuilder::default()
            .with_max_queue_size(20)
            .with_max_export_batch_size(5)
            .with_scheduled_delay(Duration::from_millis(50))
            .build();

        // Create the processor with the thread-safe exporter
        let processor = Arc::new(BatchSpanProcessor::new(exporter, config));

        let mut handles = vec![];
        for _ in 0..10 {
            let processor_clone = Arc::clone(&processor);
            let handle = tokio::spawn(async move {
                let span = new_test_export_span_data();
                processor_clone.on_end(span);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        // Allow time for batching and export
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Verify exported spans
        let exported_spans = exporter_shared.lock().unwrap();
        assert_eq!(exported_spans.len(), 10);
    }
}
