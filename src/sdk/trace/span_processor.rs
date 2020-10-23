use crate::{
    api,
    exporter::trace::{SpanData, SpanExporter},
};
use futures::{channel::mpsc, executor, future::BoxFuture, Future, FutureExt, Stream, StreamExt};
use std::fmt;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Mutex;
use std::time;

/// Delay interval between two consecutive exports, default to be 5000.
const OTEL_BSP_SCHEDULE_DELAY_MILLIS: &str = "OTEL_BSP_SCHEDULE_DELAY_MILLIS";
/// Default delay interval between two consecutive exports.
const OTEL_BSP_SCHEDULE_DELAY_MILLIS_DEFAULT: u64 = 5000;
/// Maximum queue size, default to be 2048
const OTEL_BSP_MAX_QUEUE_SIZE: &str = "OTEL_BSP_MAX_QUEUE_SIZE";
/// Default maximum queue size
const OTEL_BSP_MAX_QUEUE_SIZE_DEFAULT: usize = 2048;
/// Maximum batch size, must be less than or equal to OTEL_BSP_MAX_QUEUE_SIZE, default to be 512
const OTEL_BSP_MAX_EXPORT_BATCH_SIZE: &str = "OTEL_BSP_MAX_EXPORT_BATCH_SIZE";
/// Default maximum batch size
const OTEL_BSP_MAX_EXPORT_BATCH_SIZE_DEFAULT: usize = 512;

/// A [`SpanProcessor`] that exports synchronously when spans are finished.
///
/// # Examples
///
/// Note that the simple processor exports synchronously every time a span is
/// ended. If you find this limiting, consider the batch processor instead.
///
/// ```
/// use opentelemetry::{api::trace as apitrace, sdk::trace as sdktrace, global};
///
/// // Configure your preferred exporter
/// let exporter = apitrace::NoopSpanExporter::new();
///
/// // Then use the `with_simple_exporter` method to have the provider export when spans finish.
/// let provider = sdktrace::TracerProvider::builder()
///     .with_simple_exporter(exporter)
///     .build();
///
/// let guard = global::set_tracer_provider(provider);
/// # drop(guard)
/// ```
///
/// [`SpanProcessor`]: ../../api/trace/span_processor/trait.SpanProcessor.html
#[derive(Debug)]
pub struct SimpleSpanProcessor {
    exporter: Box<dyn SpanExporter>,
}

impl SimpleSpanProcessor {
    pub(crate) fn new(exporter: Box<dyn SpanExporter>) -> Self {
        SimpleSpanProcessor { exporter }
    }
}

impl api::trace::SpanProcessor for SimpleSpanProcessor {
    fn on_start(&self, _span: &SpanData) {
        // Ignored
    }

    fn on_end(&self, span: SpanData) {
        executor::block_on(self.exporter.export(vec![span]));
    }

    fn shutdown(&mut self) {
        self.exporter.shutdown();
    }
}

/// A [`SpanProcessor`] that asynchronously buffers finished spans and reports
/// them at a preconfigured interval.
///
/// # Examples
///
/// This processor can be configured with an [`executor`] of your choice to
/// batch and upload spans asynchronously when they end. If you have added a
/// library like [`tokio`] or [`async-std`], you can pass in their respective
/// `spawn` and `interval` functions to have batching performed in those
/// contexts.
///
/// ```
/// use futures::{stream};
/// use opentelemetry::{api::trace as apitrace, sdk::trace as sdktrace, global};
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() {
///     // Configure your preferred exporter
///     let exporter = apitrace::NoopSpanExporter::new();
///
///     // Then build a batch processor. You can use whichever executor you have available, for
///     // example if you are using `async-std` instead of `tokio` you can replace the spawn and
///     // interval functions with `async_std::task::spawn` and `async_std::stream::interval`.
///     let batch = sdktrace::BatchSpanProcessor::builder(exporter, tokio::spawn, tokio::time::interval)
///         .with_max_queue_size(4096)
///         .build();
///
///     // Then use the `with_batch_exporter` method to have the provider export spans in batches.
///     let provider = sdktrace::TracerProvider::builder()
///         .with_batch_exporter(batch)
///         .build();
///
///     let guard = global::set_tracer_provider(provider);
///     # drop(guard)
/// }
/// ```
///
/// [`SpanProcessor`]: ../../api/trace/span_processor/trait.SpanProcessor.html
/// [`executor`]: https://docs.rs/futures/0.3/futures/executor/index.html
/// [`tokio`]: https://tokio.rs
/// [`async-std`]: https://async.rs
pub struct BatchSpanProcessor {
    message_sender: Mutex<mpsc::Sender<BatchMessage>>,
    worker_handle: Option<Pin<Box<dyn Future<Output = ()> + Send + Sync>>>,
}

impl fmt::Debug for BatchSpanProcessor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BatchSpanProcessor")
            .field("message_sender", &self.message_sender)
            .finish()
    }
}

impl api::trace::SpanProcessor for BatchSpanProcessor {
    fn on_start(&self, _span: &SpanData) {
        // Ignored
    }

    fn on_end(&self, span: SpanData) {
        if let Ok(mut sender) = self.message_sender.lock() {
            let _ = sender.try_send(BatchMessage::ExportSpan(span));
        }
    }

    fn shutdown(&mut self) {
        if let Ok(mut sender) = self.message_sender.lock() {
            // Send shutdown message to worker future
            if sender.try_send(BatchMessage::Shutdown).is_ok() {
                if let Some(worker_handle) = self.worker_handle.take() {
                    // Block waiting for worker to shut down if sending was successful
                    futures::executor::block_on(worker_handle)
                }
            }
        }
    }
}

#[derive(Debug)]
enum BatchMessage {
    ExportSpan(SpanData),
    Tick,
    Shutdown,
}

impl BatchSpanProcessor {
    pub(crate) fn new<S, SH, SO, I, IS, ISI>(
        mut exporter: Box<dyn SpanExporter>,
        spawn: S,
        interval: I,
        config: BatchConfig,
    ) -> Self
    where
        S: Fn(BoxFuture<'static, ()>) -> SH,
        SH: Future<Output = SO> + Send + Sync + 'static,
        I: Fn(time::Duration) -> IS,
        IS: Stream<Item = ISI> + Send + 'static,
    {
        let (message_sender, message_receiver) = mpsc::channel(config.max_queue_size);
        let ticker = interval(config.scheduled_delay).map(|_| BatchMessage::Tick);

        // Spawn worker process via user-defined spawn function.
        let worker_handle = spawn(Box::pin(async move {
            let mut spans = Vec::new();
            let mut messages = Box::pin(futures::stream::select(message_receiver, ticker));

            while let Some(message) = messages.next().await {
                match message {
                    // Span has finished, add to buffer of pending spans.
                    BatchMessage::ExportSpan(span) => {
                        if spans.len() < config.max_queue_size {
                            spans.push(span);
                        }
                    }
                    // Span batch interval time reached, export current spans.
                    BatchMessage::Tick => {
                        while !spans.is_empty() {
                            let batch = spans.split_off(
                                spans.len().saturating_sub(config.max_export_batch_size),
                            );
                            exporter.export(batch).await;
                        }
                    }
                    // Stream has terminated or processor is shutdown, return to finish execution.
                    BatchMessage::Shutdown => {
                        while !spans.is_empty() {
                            let batch = spans.split_off(
                                spans.len().saturating_sub(config.max_export_batch_size),
                            );
                            exporter.export(batch).await;
                        }
                        exporter.shutdown();
                        break;
                    }
                }
            }
        }))
        .map(|_| ());

        // Return batch processor with link to worker
        BatchSpanProcessor {
            message_sender: Mutex::new(message_sender),
            worker_handle: Some(Box::pin(worker_handle)),
        }
    }

    /// Create a new batch processor builder
    pub fn builder<E, S, SH, SO, I, IO>(
        exporter: E,
        spawn: S,
        interval: I,
    ) -> BatchSpanProcessorBuilder<E, S, I>
    where
        E: SpanExporter,
        S: Fn(BoxFuture<'static, ()>) -> SH,
        SH: Future<Output = SO> + Send + Sync + 'static,
        I: Fn(time::Duration) -> IO,
    {
        BatchSpanProcessorBuilder {
            exporter,
            spawn,
            interval,
            config: Default::default(),
        }
    }

    /// Create a new batch processor builder and set the config value based on environment variables.
    ///
    /// If the value in environment variables is illegal, will fall back to use default value.
    ///
    /// Note that export batch size should be less than or equals to max queue size.
    /// If export batch size is larger than max queue size, we will lower to be the same as max
    /// queue size
    pub fn from_env<E, S, SH, SO, I, IO>(
        exporter: E,
        spawn: S,
        interval: I,
    ) -> BatchSpanProcessorBuilder<E, S, I>
    where
        E: SpanExporter,
        S: Fn(BoxFuture<'static, ()>) -> SH,
        SH: Future<Output = SO> + Send + Sync + 'static,
        I: Fn(time::Duration) -> IO,
    {
        let mut config = BatchConfig::default();
        let schedule_delay = std::env::var(OTEL_BSP_SCHEDULE_DELAY_MILLIS)
            .map(|delay| u64::from_str(&delay).unwrap_or(OTEL_BSP_SCHEDULE_DELAY_MILLIS_DEFAULT))
            .unwrap_or(OTEL_BSP_SCHEDULE_DELAY_MILLIS_DEFAULT);
        config.scheduled_delay = time::Duration::from_millis(schedule_delay);

        let max_queue_size = std::env::var(OTEL_BSP_MAX_QUEUE_SIZE)
            .map(|queue_size| {
                usize::from_str(&queue_size).unwrap_or(OTEL_BSP_MAX_QUEUE_SIZE_DEFAULT)
            })
            .unwrap_or(OTEL_BSP_MAX_QUEUE_SIZE_DEFAULT);
        config.max_queue_size = max_queue_size;

        let max_export_batch_size = std::env::var(OTEL_BSP_MAX_EXPORT_BATCH_SIZE)
            .map(|batch_size| {
                usize::from_str(&batch_size).unwrap_or(OTEL_BSP_MAX_EXPORT_BATCH_SIZE_DEFAULT)
            })
            .unwrap_or(OTEL_BSP_MAX_EXPORT_BATCH_SIZE_DEFAULT);
        // max export batch size must be less or equal to max queue size.
        // we set max export batch size to max queue size if it's larger than max queue size.
        if max_export_batch_size > max_queue_size {
            config.max_export_batch_size = max_queue_size;
        } else {
            config.max_export_batch_size = max_export_batch_size;
        }

        BatchSpanProcessorBuilder {
            config,
            exporter,
            spawn,
            interval,
        }
    }
}

/// Batch span processor configuration
#[derive(Debug)]
pub struct BatchConfig {
    /// The maximum queue size to buffer spans for delayed processing. If the
    /// queue gets full it drops the spans. The default value of is 2048.
    max_queue_size: usize,

    /// The delay interval in milliseconds between two consecutive processing
    /// of batches. The default value is 5 seconds.
    scheduled_delay: time::Duration,

    /// The maximum number of spans to process in a single batch. If there are
    /// more than one batch worth of spans then it processes multiple batches
    /// of spans one batch after the other without any delay. The default value
    /// is 512.
    max_export_batch_size: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        BatchConfig {
            max_queue_size: OTEL_BSP_MAX_QUEUE_SIZE_DEFAULT,
            scheduled_delay: time::Duration::from_millis(OTEL_BSP_SCHEDULE_DELAY_MILLIS_DEFAULT),
            max_export_batch_size: OTEL_BSP_MAX_EXPORT_BATCH_SIZE_DEFAULT,
        }
    }
}

/// A builder for creating [`BatchSpanProcessor`] instances.
///
/// [`BatchSpanProcessor`]: struct.BatchSpanProcessor.html
#[derive(Debug)]
pub struct BatchSpanProcessorBuilder<E, S, I> {
    exporter: E,
    interval: I,
    spawn: S,
    config: BatchConfig,
}

impl<E, S, SH, SO, I, IS, ISI> BatchSpanProcessorBuilder<E, S, I>
where
    E: SpanExporter + 'static,
    S: Fn(BoxFuture<'static, ()>) -> SH,
    SH: Future<Output = SO> + Send + Sync + 'static,
    I: Fn(time::Duration) -> IS,
    IS: Stream<Item = ISI> + Send + 'static,
{
    /// Set max queue size for batches
    pub fn with_max_queue_size(self, size: usize) -> Self {
        let mut config = self.config;
        config.max_queue_size = size;

        BatchSpanProcessorBuilder { config, ..self }
    }

    /// Set scheduled delay for batches
    pub fn with_scheduled_delay(self, delay: time::Duration) -> Self {
        let mut config = self.config;
        config.scheduled_delay = delay;

        BatchSpanProcessorBuilder { config, ..self }
    }

    /// Set max export size for batches, should always less than or equals to max queue size.
    ///
    /// If input is larger than max queue size, will lower it to be equal to max queue size
    pub fn with_max_export_batch_size(self, size: usize) -> Self {
        let mut config = self.config;
        if size > config.max_queue_size {
            config.max_export_batch_size = config.max_queue_size;
        } else {
            config.max_export_batch_size = size;
        }

        BatchSpanProcessorBuilder { config, ..self }
    }

    /// Build a batch processor
    pub fn build(self) -> BatchSpanProcessor {
        BatchSpanProcessor::new(
            Box::new(self.exporter),
            self.spawn,
            self.interval,
            self.config,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::exporter::trace::stdout;
    use crate::sdk::trace::span_processor::{
        OTEL_BSP_MAX_EXPORT_BATCH_SIZE, OTEL_BSP_MAX_QUEUE_SIZE, OTEL_BSP_MAX_QUEUE_SIZE_DEFAULT,
        OTEL_BSP_SCHEDULE_DELAY_MILLIS, OTEL_BSP_SCHEDULE_DELAY_MILLIS_DEFAULT,
    };
    use crate::sdk::trace::BatchSpanProcessor;
    use std::time;

    #[test]
    fn test_build_batch_span_processor_from_env() {
        std::env::set_var(OTEL_BSP_MAX_EXPORT_BATCH_SIZE, "500");
        std::env::set_var(OTEL_BSP_SCHEDULE_DELAY_MILLIS, "I am not number");

        let mut builder = BatchSpanProcessor::from_env(
            stdout::Exporter::new(std::io::stdout(), true),
            tokio::spawn,
            tokio::time::interval,
        );
        // export batch size cannot exceed max queue size
        assert_eq!(builder.config.max_export_batch_size, 500);
        assert_eq!(
            builder.config.scheduled_delay,
            time::Duration::from_millis(OTEL_BSP_SCHEDULE_DELAY_MILLIS_DEFAULT)
        );
        assert_eq!(
            builder.config.max_queue_size,
            OTEL_BSP_MAX_QUEUE_SIZE_DEFAULT
        );

        std::env::set_var(OTEL_BSP_MAX_QUEUE_SIZE, "120");
        builder = BatchSpanProcessor::from_env(
            stdout::Exporter::new(std::io::stdout(), true),
            tokio::spawn,
            tokio::time::interval,
        );

        assert_eq!(builder.config.max_export_batch_size, 120);
        assert_eq!(builder.config.max_queue_size, 120);
    }
}
