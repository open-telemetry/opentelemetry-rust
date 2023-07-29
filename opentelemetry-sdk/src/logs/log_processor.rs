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
use opentelemetry_api::logs::Severity;
use opentelemetry_api::{
    global,
    logs::{LogError, LogResult},
};
use std::thread;
use std::{
    fmt::{self, Debug, Formatter},
    time::Duration,
};

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
    sender: crossbeam_channel::Sender<Option<LogData>>,
    shutdown: crossbeam_channel::Receiver<()>,
}

impl SimpleLogProcessor {
    pub(crate) fn new(mut exporter: Box<dyn LogExporter>) -> Self {
        let (log_tx, log_rx) = crossbeam_channel::unbounded();
        let (shutdown_tx, shutdown_rx) = crossbeam_channel::bounded(0);

        let _ = thread::Builder::new()
            .name("opentelemetry-log-exporter".to_string())
            .spawn(move || {
                while let Ok(Some(log)) = log_rx.recv() {
                    if let Err(err) = futures_executor::block_on(exporter.export(vec![log])) {
                        global::handle_error(err);
                    }
                }

                exporter.shutdown();

                if let Err(err) = shutdown_tx.send(()) {
                    global::handle_error(LogError::from(format!(
                        "could not send shutdown: {:?}",
                        err
                    )));
                }
            });

        SimpleLogProcessor {
            sender: log_tx,
            shutdown: shutdown_rx,
        }
    }
}

impl LogProcessor for SimpleLogProcessor {
    fn emit(&self, data: LogData) {
        if let Err(err) = self.sender.send(Some(data)) {
            global::handle_error(LogError::from(format!("error processing log {:?}", err)));
        }
    }

    fn force_flush(&self) -> LogResult<()> {
        // Ignored since all logs in Simple Processor will be exported as they ended.
        Ok(())
    }

    fn shutdown(&mut self) -> LogResult<()> {
        if self.sender.send(None).is_ok() {
            if let Err(err) = self.shutdown.recv() {
                global::handle_error(LogError::from(format!(
                    "error shutting down log processor: {:?}",
                    err
                )))
            }
        }
        Ok(())
    }

    #[cfg(feature = "logs_level_enabled")]
    fn event_enabled(&self, _level: Severity, _target: &str, _name: &str) -> bool {
        true
    }
}

/// A [`LogProcessor`] that asynchronously buffers log records and reports
/// them at a preconfigured interval.
pub struct BatchLogProcessor<R: RuntimeChannel<BatchMessage>> {
    message_sender: R::Sender,
}

impl<R: RuntimeChannel<BatchMessage>> Debug for BatchLogProcessor<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("BatchLogProcessor")
            .field("message_sender", &self.message_sender)
            .finish()
    }
}

impl<R: RuntimeChannel<BatchMessage>> LogProcessor for BatchLogProcessor<R> {
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

impl<R: RuntimeChannel<BatchMessage>> BatchLogProcessor<R> {
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
    R: RuntimeChannel<BatchMessage>,
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
        BatchConfig {
            max_queue_size: 2_048,
            scheduled_delay: Duration::from_millis(1_000),
            max_export_batch_size: 512,
            max_export_timeout: Duration::from_millis(30_000),
        }
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
    R: RuntimeChannel<BatchMessage>,
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

    /// Build a batch processor
    pub fn build(self) -> BatchLogProcessor<R> {
        BatchLogProcessor::new(Box::new(self.exporter), self.config, self.runtime)
    }
}

/// Messages sent between application thread and batch log processor's work thread.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum BatchMessage {
    /// Export logs, usually called when the log is emitted.
    ExportLog(LogData),
    /// Flush the current buffer to the backend, it can be triggered by
    /// pre configured interval or a call to `force_push` function.
    Flush(Option<oneshot::Sender<ExportResult>>),
    /// Shut down the worker thread, push all logs in buffer to the backend.
    Shutdown(oneshot::Sender<ExportResult>),
}
