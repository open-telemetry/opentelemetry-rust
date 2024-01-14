use crate::{
    export::logs::{ExportResult, LogData, LogExporter},
    runtime::{RuntimeChannel, TrySend},
};
use futures_channel::oneshot;
use futures_util::{
    future::{self, Either},
    {pin_mut, stream, StreamExt as _},
};
#[cfg(feature = "logs_level_enabled")]
use opentelemetry::logs::Severity;
use opentelemetry::{
    global,
    logs::{LogError, LogResult},
};
use std::{env, sync::Mutex};
use std::{
    fmt::{self, Debug, Formatter},
    str::FromStr,
    time::Duration,
};

/// Delay interval between two consecutive exports.
const OTEL_BLRP_SCHEDULE_DELAY: &str = "OTEL_BLRP_SCHEDULE_DELAY";
/// Default delay interval between two consecutive exports.
const OTEL_BLRP_SCHEDULE_DELAY_DEFAULT: u64 = 1_000;
/// Maximum allowed time to export data.
const OTEL_BLRP_EXPORT_TIMEOUT: &str = "OTEL_BLRP_EXPORT_TIMEOUT";
/// Default maximum allowed time to export data.
const OTEL_BLRP_EXPORT_TIMEOUT_DEFAULT: u64 = 30_000;
/// Maximum queue size.
const OTEL_BLRP_MAX_QUEUE_SIZE: &str = "OTEL_BLRP_MAX_QUEUE_SIZE";
/// Default maximum queue size.
const OTEL_BLRP_MAX_QUEUE_SIZE_DEFAULT: usize = 2_048;
/// Maximum batch size, must be less than or equal to OTEL_BLRP_MAX_QUEUE_SIZE.
const OTEL_BLRP_MAX_EXPORT_BATCH_SIZE: &str = "OTEL_BLRP_MAX_EXPORT_BATCH_SIZE";
/// Default maximum batch size.
const OTEL_BLRP_MAX_EXPORT_BATCH_SIZE_DEFAULT: usize = 512;

/// The interface for plugging into a [`Logger`].
///
/// [`Logger`]: crate::logs::Logger
pub trait LogProcessor: Send + Sync + Debug {
    /// Called when a log record is ready to processed and exported.
    fn emit(&self, data: LogData);
    /// Force the logs lying in the cache to be exported.
    fn force_flush(&self) -> LogResult<()>;
    /// Shuts down the processor.
    fn shutdown(&mut self) -> LogResult<()>;
    #[cfg(feature = "logs_level_enabled")]
    /// Check if logging is enabled
    fn event_enabled(&self, level: Severity, target: &str, name: &str) -> bool;
}

/// A [`LogProcessor`] that exports synchronously when logs are emitted.
///
/// # Examples
///
/// Note that the simple processor exports synchronously every time a log is
/// emitted. If you find this limiting, consider the batch processor instead.
#[derive(Debug)]
pub struct SimpleLogProcessor {
    exporter: Mutex<Box<dyn LogExporter>>,
}

impl SimpleLogProcessor {
    pub(crate) fn new(exporter: Box<dyn LogExporter>) -> Self {
        SimpleLogProcessor {
            exporter: Mutex::new(exporter),
        }
    }
}

impl LogProcessor for SimpleLogProcessor {
    fn emit(&self, data: LogData) {
        let result = self
            .exporter
            .lock()
            .map_err(|_| LogError::Other("simple logprocessor mutex poison".into()))
            .and_then(|mut exporter| futures_executor::block_on(exporter.export(vec![data])));
        if let Err(err) = result {
            global::handle_error(err);
        }
    }

    fn force_flush(&self) -> LogResult<()> {
        Ok(())
    }

    fn shutdown(&mut self) -> LogResult<()> {
        if let Ok(mut exporter) = self.exporter.lock() {
            exporter.shutdown();
            Ok(())
        } else {
            Err(LogError::Other(
                "simple logprocessor mutex poison during shutdown".into(),
            ))
        }
    }

    #[cfg(feature = "logs_level_enabled")]
    fn event_enabled(&self, _level: Severity, _target: &str, _name: &str) -> bool {
        true
    }
}

/// A [`LogProcessor`] that asynchronously buffers log records and reports
/// them at a pre-configured interval.
pub struct BatchLogProcessor<R: RuntimeChannel> {
    message_sender: R::Sender<BatchMessage>,
}

impl<R: RuntimeChannel> Debug for BatchLogProcessor<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("BatchLogProcessor")
            .field("message_sender", &self.message_sender)
            .finish()
    }
}

impl<R: RuntimeChannel> LogProcessor for BatchLogProcessor<R> {
    fn emit(&self, data: LogData) {
        let result = self.message_sender.try_send(BatchMessage::ExportLog(data));

        if let Err(err) = result {
            global::handle_error(LogError::Other(err.into()));
        }
    }

    #[cfg(feature = "logs_level_enabled")]
    fn event_enabled(&self, _level: Severity, _target: &str, _name: &str) -> bool {
        true
    }

    fn force_flush(&self) -> LogResult<()> {
        let (res_sender, res_receiver) = oneshot::channel();
        self.message_sender
            .try_send(BatchMessage::Flush(Some(res_sender)))
            .map_err(|err| LogError::Other(err.into()))?;

        futures_executor::block_on(res_receiver)
            .map_err(|err| LogError::Other(err.into()))
            .and_then(std::convert::identity)
    }

    fn shutdown(&mut self) -> LogResult<()> {
        let (res_sender, res_receiver) = oneshot::channel();
        self.message_sender
            .try_send(BatchMessage::Shutdown(res_sender))
            .map_err(|err| LogError::Other(err.into()))?;

        futures_executor::block_on(res_receiver)
            .map_err(|err| LogError::Other(err.into()))
            .and_then(std::convert::identity)
    }
}

impl<R: RuntimeChannel> BatchLogProcessor<R> {
    pub(crate) fn new(mut exporter: Box<dyn LogExporter>, config: BatchConfig, runtime: R) -> Self {
        let (message_sender, message_receiver) =
            runtime.batch_message_channel(config.max_queue_size);
        let ticker = runtime
            .interval(config.scheduled_delay)
            .map(|_| BatchMessage::Flush(None));
        let timeout_runtime = runtime.clone();

        // Spawn worker process via user-defined spawn function.
        runtime.spawn(Box::pin(async move {
            let mut logs = Vec::new();
            let mut messages = Box::pin(stream::select(message_receiver, ticker));

            while let Some(message) = messages.next().await {
                match message {
                    // Log has finished, add to buffer of pending logs.
                    BatchMessage::ExportLog(log) => {
                        logs.push(log);

                        if logs.len() == config.max_export_batch_size {
                            let result = export_with_timeout(
                                config.max_export_timeout,
                                exporter.as_mut(),
                                &timeout_runtime,
                                logs.split_off(0),
                            )
                            .await;

                            if let Err(err) = result {
                                global::handle_error(err);
                            }
                        }
                    }
                    // Log batch interval time reached or a force flush has been invoked, export current spans.
                    BatchMessage::Flush(res_channel) => {
                        let result = export_with_timeout(
                            config.max_export_timeout,
                            exporter.as_mut(),
                            &timeout_runtime,
                            logs.split_off(0),
                        )
                        .await;

                        if let Some(channel) = res_channel {
                            if let Err(result) = channel.send(result) {
                                global::handle_error(LogError::from(format!(
                                    "failed to send flush result: {:?}",
                                    result
                                )));
                            }
                        } else if let Err(err) = result {
                            global::handle_error(err);
                        }
                    }
                    // Stream has terminated or processor is shutdown, return to finish execution.
                    BatchMessage::Shutdown(ch) => {
                        let result = export_with_timeout(
                            config.max_export_timeout,
                            exporter.as_mut(),
                            &timeout_runtime,
                            logs.split_off(0),
                        )
                        .await;

                        exporter.shutdown();

                        if let Err(result) = ch.send(result) {
                            global::handle_error(LogError::from(format!(
                                "failed to send batch processor shutdown result: {:?}",
                                result
                            )));
                        }

                        break;
                    }
                }
            }
        }));

        // Return batch processor with link to worker
        BatchLogProcessor { message_sender }
    }

    /// Create a new batch processor builder
    pub fn builder<E>(exporter: E, runtime: R) -> BatchLogProcessorBuilder<E, R>
    where
        E: LogExporter,
    {
        BatchLogProcessorBuilder {
            exporter,
            config: BatchConfig::default(),
            runtime,
        }
    }
}

async fn export_with_timeout<R, E>(
    time_out: Duration,
    exporter: &mut E,
    runtime: &R,
    batch: Vec<LogData>,
) -> ExportResult
where
    R: RuntimeChannel,
    E: LogExporter + ?Sized,
{
    if batch.is_empty() {
        return Ok(());
    }

    let export = exporter.export(batch);
    let timeout = runtime.delay(time_out);
    pin_mut!(export);
    pin_mut!(timeout);
    match future::select(export, timeout).await {
        Either::Left((export_res, _)) => export_res,
        Either::Right((_, _)) => ExportResult::Err(LogError::ExportTimedOut(time_out)),
    }
}

/// Batch log processor configuration
#[derive(Debug)]
pub struct BatchConfig {
    /// The maximum queue size to buffer logs for delayed processing. If the
    /// queue gets full it drops the logs. The default value of is 2048.
    max_queue_size: usize,

    /// The delay interval in milliseconds between two consecutive processing
    /// of batches. The default value is 1 second.
    scheduled_delay: Duration,

    /// The maximum number of logs to process in a single batch. If there are
    /// more than one batch worth of logs then it processes multiple batches
    /// of logs one batch after the other without any delay. The default value
    /// is 512.
    max_export_batch_size: usize,

    /// The maximum duration to export a batch of data.
    max_export_timeout: Duration,
}

impl Default for BatchConfig {
    fn default() -> Self {
        let mut config = BatchConfig {
            max_queue_size: OTEL_BLRP_MAX_QUEUE_SIZE_DEFAULT,
            scheduled_delay: Duration::from_millis(OTEL_BLRP_SCHEDULE_DELAY_DEFAULT),
            max_export_batch_size: OTEL_BLRP_MAX_EXPORT_BATCH_SIZE_DEFAULT,
            max_export_timeout: Duration::from_millis(OTEL_BLRP_EXPORT_TIMEOUT_DEFAULT),
        };

        if let Some(max_queue_size) = env::var(OTEL_BLRP_MAX_QUEUE_SIZE)
            .ok()
            .and_then(|queue_size| usize::from_str(&queue_size).ok())
        {
            config.max_queue_size = max_queue_size;
        }

        if let Some(max_export_batch_size) = env::var(OTEL_BLRP_MAX_EXPORT_BATCH_SIZE)
            .ok()
            .and_then(|batch_size| usize::from_str(&batch_size).ok())
        {
            config.max_export_batch_size = max_export_batch_size;
        }

        // max export batch size must be less or equal to max queue size.
        // we set max export batch size to max queue size if it's larger than max queue size.
        if config.max_export_batch_size > config.max_queue_size {
            config.max_export_batch_size = config.max_queue_size;
        }

        if let Some(scheduled_delay) = env::var(OTEL_BLRP_SCHEDULE_DELAY)
            .ok()
            .or_else(|| env::var("OTEL_BLRP_SCHEDULE_DELAY_MILLIS").ok())
            .and_then(|delay| u64::from_str(&delay).ok())
        {
            config.scheduled_delay = Duration::from_millis(scheduled_delay);
        }

        if let Some(max_export_timeout) = env::var(OTEL_BLRP_EXPORT_TIMEOUT)
            .ok()
            .or_else(|| env::var("OTEL_BLRP_EXPORT_TIMEOUT_MILLIS").ok())
            .and_then(|s| u64::from_str(&s).ok())
        {
            config.max_export_timeout = Duration::from_millis(max_export_timeout);
        }

        config
    }
}

impl BatchConfig {
    /// Set max_queue_size for [`BatchConfig`].
    /// It's the maximum queue size to buffer logs for delayed processing.
    /// If the queue gets full it will drop the logs.
    /// The default value of is 2048.
    pub fn with_max_queue_size(mut self, max_queue_size: usize) -> Self {
        self.max_queue_size = max_queue_size;
        self
    }

    /// Set scheduled_delay for [`BatchConfig`].
    /// It's the delay interval in milliseconds between two consecutive processing of batches.
    /// The default value is 1000 milliseconds.
    pub fn with_scheduled_delay(mut self, scheduled_delay: Duration) -> Self {
        self.scheduled_delay = scheduled_delay;
        self
    }

    /// Set max_export_timeout for [`BatchConfig`].
    /// It's the maximum duration to export a batch of data.
    /// The default value is 30000 milliseconds.
    pub fn with_max_export_timeout(mut self, max_export_timeout: Duration) -> Self {
        self.max_export_timeout = max_export_timeout;
        self
    }

    /// Set max_export_batch_size for [`BatchConfig`].
    /// It's the maximum number of logs to process in a single batch. If there are
    /// more than one batch worth of logs then it processes multiple batches
    /// of logs one batch after the other without any delay.
    /// The default value is 512.
    pub fn with_max_export_batch_size(mut self, max_export_batch_size: usize) -> Self {
        self.max_export_batch_size = max_export_batch_size;
        self
    }
}

/// A builder for creating [`BatchLogProcessor`] instances.
///
#[derive(Debug)]
pub struct BatchLogProcessorBuilder<E, R> {
    exporter: E,
    config: BatchConfig,
    runtime: R,
}

impl<E, R> BatchLogProcessorBuilder<E, R>
where
    E: LogExporter + 'static,
    R: RuntimeChannel,
{
    /// Set max queue size for batches
    pub fn with_max_queue_size(self, size: usize) -> Self {
        let mut config = self.config;
        config.max_queue_size = size;

        BatchLogProcessorBuilder { config, ..self }
    }

    /// Set scheduled delay for batches
    pub fn with_scheduled_delay(self, delay: Duration) -> Self {
        let mut config = self.config;
        config.scheduled_delay = delay;

        BatchLogProcessorBuilder { config, ..self }
    }

    /// Set max timeout for exporting.
    pub fn with_max_timeout(self, timeout: Duration) -> Self {
        let mut config = self.config;
        config.max_export_timeout = timeout;

        BatchLogProcessorBuilder { config, ..self }
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

        BatchLogProcessorBuilder { config, ..self }
    }

    /// Set the BatchConfig for [`BatchLogProcessorBuilder`]
    pub fn with_batch_config(self, config: BatchConfig) -> Self {
        BatchLogProcessorBuilder { config, ..self }
    }

    /// Build a batch processor
    pub fn build(self) -> BatchLogProcessor<R> {
        BatchLogProcessor::new(Box::new(self.exporter), self.config, self.runtime)
    }
}

/// Messages sent between application thread and batch log processor's work thread.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
enum BatchMessage {
    /// Export logs, usually called when the log is emitted.
    ExportLog(LogData),
    /// Flush the current buffer to the backend, it can be triggered by
    /// pre configured interval or a call to `force_push` function.
    Flush(Option<oneshot::Sender<ExportResult>>),
    /// Shut down the worker thread, push all logs in buffer to the backend.
    Shutdown(oneshot::Sender<ExportResult>),
}

#[cfg(all(test, feature = "testing", feature = "logs"))]
mod tests {
    use super::{
        BatchLogProcessor, OTEL_BLRP_EXPORT_TIMEOUT, OTEL_BLRP_MAX_EXPORT_BATCH_SIZE,
        OTEL_BLRP_MAX_QUEUE_SIZE, OTEL_BLRP_SCHEDULE_DELAY,
    };
    use crate::{
        logs::{
            log_processor::{
                OTEL_BLRP_EXPORT_TIMEOUT_DEFAULT, OTEL_BLRP_MAX_EXPORT_BATCH_SIZE_DEFAULT,
                OTEL_BLRP_MAX_QUEUE_SIZE_DEFAULT, OTEL_BLRP_SCHEDULE_DELAY_DEFAULT,
            },
            BatchConfig,
        },
        runtime,
        testing::logs::InMemoryLogsExporter,
    };
    use std::time::Duration;

    #[test]
    fn test_default_const_values() {
        assert_eq!(OTEL_BLRP_SCHEDULE_DELAY, "OTEL_BLRP_SCHEDULE_DELAY");
        assert_eq!(OTEL_BLRP_SCHEDULE_DELAY_DEFAULT, 1_000);
        assert_eq!(OTEL_BLRP_EXPORT_TIMEOUT, "OTEL_BLRP_EXPORT_TIMEOUT");
        assert_eq!(OTEL_BLRP_EXPORT_TIMEOUT_DEFAULT, 30_000);
        assert_eq!(OTEL_BLRP_MAX_QUEUE_SIZE, "OTEL_BLRP_MAX_QUEUE_SIZE");
        assert_eq!(OTEL_BLRP_MAX_QUEUE_SIZE_DEFAULT, 2_048);
        assert_eq!(
            OTEL_BLRP_MAX_EXPORT_BATCH_SIZE,
            "OTEL_BLRP_MAX_EXPORT_BATCH_SIZE"
        );
        assert_eq!(OTEL_BLRP_MAX_EXPORT_BATCH_SIZE_DEFAULT, 512);
    }

    #[test]
    fn test_default_batch_config_adheres_to_specification() {
        let config = BatchConfig::default();

        assert_eq!(
            config.scheduled_delay,
            Duration::from_millis(OTEL_BLRP_SCHEDULE_DELAY_DEFAULT)
        );
        assert_eq!(
            config.max_export_timeout,
            Duration::from_millis(OTEL_BLRP_EXPORT_TIMEOUT_DEFAULT)
        );
        assert_eq!(config.max_queue_size, OTEL_BLRP_MAX_QUEUE_SIZE_DEFAULT);
        assert_eq!(
            config.max_export_batch_size,
            OTEL_BLRP_MAX_EXPORT_BATCH_SIZE_DEFAULT
        );
    }

    #[test]
    fn test_batch_config_configurable_by_env_vars() {
        let env_vars = vec![
            (OTEL_BLRP_SCHEDULE_DELAY, Some("2000")),
            (OTEL_BLRP_EXPORT_TIMEOUT, Some("60000")),
            (OTEL_BLRP_MAX_QUEUE_SIZE, Some("4096")),
            (OTEL_BLRP_MAX_EXPORT_BATCH_SIZE, Some("1024")),
        ];

        let config = temp_env::with_vars(env_vars, BatchConfig::default);

        assert_eq!(config.scheduled_delay, Duration::from_millis(2000));
        assert_eq!(config.max_export_timeout, Duration::from_millis(60000));
        assert_eq!(config.max_queue_size, 4096);
        assert_eq!(config.max_export_batch_size, 1024);
    }

    #[test]
    fn test_batch_config_configurable_by_env_vars_millis() {
        let env_vars = vec![
            ("OTEL_BLRP_SCHEDULE_DELAY_MILLIS", Some("3000")),
            ("OTEL_BLRP_EXPORT_TIMEOUT_MILLIS", Some("70000")),
        ];

        let config = temp_env::with_vars(env_vars, BatchConfig::default);

        assert_eq!(config.scheduled_delay, Duration::from_millis(3000));
        assert_eq!(config.max_export_timeout, Duration::from_millis(70000));
        assert_eq!(config.max_queue_size, OTEL_BLRP_MAX_QUEUE_SIZE_DEFAULT);
        assert_eq!(
            config.max_export_batch_size,
            OTEL_BLRP_MAX_EXPORT_BATCH_SIZE_DEFAULT
        );
    }

    #[test]
    fn test_batch_config_configurable_by_env_vars_precedence() {
        let env_vars = vec![
            (OTEL_BLRP_SCHEDULE_DELAY, Some("2000")),
            ("OTEL_BLRP_SCHEDULE_DELAY_MILLIS", Some("3000")),
            (OTEL_BLRP_EXPORT_TIMEOUT, Some("60000")),
            ("OTEL_BLRP_EXPORT_TIMEOUT_MILLIS", Some("70000")),
        ];

        let config = temp_env::with_vars(env_vars, BatchConfig::default);

        assert_eq!(config.scheduled_delay, Duration::from_millis(2000));
        assert_eq!(config.max_export_timeout, Duration::from_millis(60000));
        assert_eq!(config.max_queue_size, OTEL_BLRP_MAX_QUEUE_SIZE_DEFAULT);
        assert_eq!(
            config.max_export_batch_size,
            OTEL_BLRP_MAX_EXPORT_BATCH_SIZE_DEFAULT
        );
    }

    #[test]
    fn test_batch_config_max_export_batch_size_validation() {
        let env_vars = vec![
            (OTEL_BLRP_MAX_QUEUE_SIZE, Some("256")),
            (OTEL_BLRP_MAX_EXPORT_BATCH_SIZE, Some("1024")),
        ];

        let config = temp_env::with_vars(env_vars, BatchConfig::default);

        assert_eq!(config.max_queue_size, 256);
        assert_eq!(config.max_export_batch_size, 256);
        assert_eq!(
            config.scheduled_delay,
            Duration::from_millis(OTEL_BLRP_SCHEDULE_DELAY_DEFAULT)
        );
        assert_eq!(
            config.max_export_timeout,
            Duration::from_millis(OTEL_BLRP_EXPORT_TIMEOUT_DEFAULT)
        );
    }

    #[test]
    fn test_batch_config_with_fields() {
        let batch = BatchConfig::default()
            .with_max_export_batch_size(1)
            .with_scheduled_delay(Duration::from_millis(2))
            .with_max_export_timeout(Duration::from_millis(3))
            .with_max_queue_size(4);

        assert_eq!(batch.max_export_batch_size, 1);
        assert_eq!(batch.scheduled_delay, Duration::from_millis(2));
        assert_eq!(batch.max_export_timeout, Duration::from_millis(3));
        assert_eq!(batch.max_queue_size, 4);
    }

    #[test]
    fn test_build_batch_log_processor_builder() {
        let mut env_vars = vec![
            (OTEL_BLRP_MAX_EXPORT_BATCH_SIZE, Some("500")),
            (OTEL_BLRP_SCHEDULE_DELAY, Some("I am not number")),
            (OTEL_BLRP_EXPORT_TIMEOUT, Some("2046")),
        ];
        temp_env::with_vars(env_vars.clone(), || {
            let builder =
                BatchLogProcessor::builder(InMemoryLogsExporter::default(), runtime::Tokio);

            assert_eq!(builder.config.max_export_batch_size, 500);
            assert_eq!(
                builder.config.scheduled_delay,
                Duration::from_millis(OTEL_BLRP_SCHEDULE_DELAY_DEFAULT)
            );
            assert_eq!(
                builder.config.max_queue_size,
                OTEL_BLRP_MAX_QUEUE_SIZE_DEFAULT
            );
            assert_eq!(
                builder.config.max_export_timeout,
                Duration::from_millis(2046)
            );
        });

        env_vars.push((OTEL_BLRP_MAX_QUEUE_SIZE, Some("120")));

        temp_env::with_vars(env_vars, || {
            let builder =
                BatchLogProcessor::builder(InMemoryLogsExporter::default(), runtime::Tokio);
            assert_eq!(builder.config.max_export_batch_size, 120);
            assert_eq!(builder.config.max_queue_size, 120);
        });
    }
}
