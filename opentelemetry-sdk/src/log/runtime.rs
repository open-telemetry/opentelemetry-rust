//! # Log Runtime
//! Log runtime is an extension to [`Runtime`]. Currently it provides a channel that used
//! by [`BatchLognProcessor`].
//!
//! [`BatchLogProcessor`]: crate::log::BatchLogProcessor
//! [`Runtime`]: crate::runtime::Runtime
use crate::log::BatchMessage;
#[cfg(feature = "rt-async-std")]
use crate::runtime::AsyncStd;
use crate::runtime::Runtime;
#[cfg(feature = "rt-tokio")]
use crate::runtime::Tokio;
#[cfg(feature = "rt-tokio-current-thread")]
use crate::runtime::TokioCurrentThread;
use futures_util::stream::Stream;
use opentelemetry_api::log::LogError;
use std::fmt::Debug;

#[cfg(any(
    feature = "rt-tokio",
    feature = "rt-tokio-current-thread",
    feature = "rt-async-std"
))]
const CHANNEL_FULL_ERROR: &str =
    "cannot send log record to the batch log processor because the channel is full";
#[cfg(any(
    feature = "rt-tokio",
    feature = "rt-tokio-current-thread",
    feature = "rt-async-std"
))]
const CHANNEL_CLOSED_ERROR: &str =
    "cannot send log record to the batch log processor because the channel is closed";

/// Log runtime is an extension to [`Runtime`]. Currently it provides a channel that used
/// by [`BatchLogProcessor`].
///
/// [`BatchSpanProcessor`]: crate::log::BatchLogProcessor
/// [`Runtime`]: crate::runtime::Runtime
pub trait LogRuntime: Runtime {
    /// A future stream to receive the batch messages from channels.
    type Receiver: Stream<Item = BatchMessage> + Send;

    /// A batch messages sender that could be sent across thread safely.
    type Sender: TrySend + Debug;

    /// Return the sender and receiver used to send batch message between tasks.
    fn batch_message_channel(&self, capacity: usize) -> (Self::Sender, Self::Receiver);
}

/// TrySend is an abstraction of sender that is capable to send BatchMessage with reference.
pub trait TrySend: Sync + Send {
    /// Try to send one batch message to worker thread.
    ///
    /// It can fail because either the receiver has closed or the buffer is full.
    fn try_send(&self, item: BatchMessage) -> Result<(), LogError>;
}

#[cfg(any(feature = "rt-tokio", feature = "rt-tokio-current-thread"))]
impl TrySend for tokio::sync::mpsc::Sender<BatchMessage> {
    fn try_send(&self, item: BatchMessage) -> Result<(), LogError> {
        self.try_send(item).map_err(|err| match err {
            tokio::sync::mpsc::error::TrySendError::Full(_) => LogError::from(CHANNEL_FULL_ERROR),
            tokio::sync::mpsc::error::TrySendError::Closed(_) => {
                LogError::from(CHANNEL_CLOSED_ERROR)
            }
        })
    }
}

#[cfg(feature = "rt-tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "rt-tokio")))]
impl LogRuntime for Tokio {
    type Receiver = tokio_stream::wrappers::ReceiverStream<BatchMessage>;
    type Sender = tokio::sync::mpsc::Sender<BatchMessage>;

    fn batch_message_channel(&self, capacity: usize) -> (Self::Sender, Self::Receiver) {
        let (sender, receiver) = tokio::sync::mpsc::channel(capacity);
        (
            sender,
            tokio_stream::wrappers::ReceiverStream::new(receiver),
        )
    }
}

#[cfg(feature = "rt-tokio-current-thread")]
#[cfg_attr(docsrs, doc(cfg(feature = "rt-tokio-current-thread")))]
impl LogRuntime for TokioCurrentThread {
    type Receiver = tokio_stream::wrappers::ReceiverStream<BatchMessage>;
    type Sender = tokio::sync::mpsc::Sender<BatchMessage>;

    fn batch_message_channel(&self, capacity: usize) -> (Self::Sender, Self::Receiver) {
        let (sender, receiver) = tokio::sync::mpsc::channel(capacity);
        (
            sender,
            tokio_stream::wrappers::ReceiverStream::new(receiver),
        )
    }
}

#[cfg(feature = "rt-async-std")]
impl TrySend for async_std::channel::Sender<BatchMessage> {
    fn try_send(&self, item: BatchMessage) -> Result<(), LogError> {
        self.try_send(item).map_err(|err| match err {
            async_std::channel::TrySendError::Full(_) => LogError::from(CHANNEL_FULL_ERROR),
            async_std::channel::TrySendError::Closed(_) => LogError::from(CHANNEL_CLOSED_ERROR),
        })
    }
}

#[cfg(feature = "rt-async-std")]
#[cfg_attr(docsrs, doc(cfg(feature = "rt-async-std")))]
impl LogRuntime for AsyncStd {
    type Receiver = async_std::channel::Receiver<BatchMessage>;
    type Sender = async_std::channel::Sender<BatchMessage>;

    fn batch_message_channel(&self, capacity: usize) -> (Self::Sender, Self::Receiver) {
        async_std::channel::bounded(capacity)
    }
}

#[cfg(test)]
// Note that all tests here should be marked as ignore so that it won't be picked up by default We
// need to run those tests one by one as the GlobalLogrProvider is a shared object between
// threads Use cargo test -- --ignored --test-threads=1 to run those tests.
mod tests {
    #[cfg(any(feature = "rt-tokio", feature = "rt-tokio-current-thread"))]
    use crate::log::LogRuntime;
    #[cfg(any(feature = "rt-tokio", feature = "rt-tokio-current-thread"))]
    use crate::runtime;
    #[cfg(any(feature = "rt-tokio", feature = "rt-tokio-current-thread"))]
    use opentelemetry_api::global::*;
    use std::sync::Arc;
    use std::{fmt::Debug, io::Write, sync::Mutex};

    #[derive(Debug)]
    struct AssertWriter {
        buf: Arc<Mutex<Vec<u8>>>,
    }

    #[cfg(any(feature = "rt-tokio", feature = "rt-tokio-current-thread"))]
    impl AssertWriter {
        fn new() -> AssertWriter {
            AssertWriter {
                buf: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn len(&self) -> usize {
            self.buf
                .lock()
                .expect("cannot acquire the lock of assert writer")
                .len()
        }
    }

    impl Write for AssertWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            let mut buffer = self
                .buf
                .lock()
                .expect("cannot acquire the lock of assert writer");
            buffer.write(buf)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            let mut buffer = self
                .buf
                .lock()
                .expect("cannot acquire the lock of assert writer");
            buffer.flush()
        }
    }

    impl Clone for AssertWriter {
        fn clone(&self) -> Self {
            AssertWriter {
                buf: self.buf.clone(),
            }
        }
    }

    #[cfg(any(feature = "rt-tokio", feature = "rt-tokio-current-thread"))]
    fn build_batch_log_emitter_provider<R: LogRuntime>(
        assert_writer: AssertWriter,
        runtime: R,
    ) -> crate::log::LogEmitterProvider {
        use crate::log::LogEmitterProvider;
        let exporter = crate::export::log::stdout::Exporter::new(assert_writer, true);
        LogEmitterProvider::builder()
            .with_batch_exporter(exporter, runtime)
            .build()
    }

    #[cfg(any(feature = "rt-tokio", feature = "rt-tokio-current-thread"))]
    fn build_simple_log_emitter_provider(
        assert_writer: AssertWriter,
    ) -> crate::log::LogEmitterProvider {
        use crate::log::LogEmitterProvider;
        let exporter = crate::export::log::stdout::Exporter::new(assert_writer, true);
        LogEmitterProvider::builder()
            .with_simple_exporter(exporter)
            .build()
    }

    #[cfg(any(feature = "rt-tokio", feature = "rt-tokio-current-thread"))]
    async fn test_create_provider_in_tokio<R: LogRuntime>(runtime: R) -> AssertWriter {
        use crate::log::LogRecord;

        let buffer = AssertWriter::new();
        let provider = build_batch_log_emitter_provider(buffer.clone(), runtime);
        let emitter = provider.log_emitter("opentelemetery");

        emitter.emit(LogRecord::default());

        buffer
    }

    // When using `tokio::spawn` to spawn the worker task in batch processor
    //
    // multiple -> no shut down -> not export
    // multiple -> shut down -> export
    // single -> no shutdown -> not export
    // single -> shutdown -> hang forever

    // When using |fut| tokio::task::spawn_blocking(|| futures::executor::block_on(fut))
    // to spawn the worker task in batch processor
    //
    // multiple -> no shutdown -> hang forever
    // multiple -> shut down -> export
    // single -> shut down -> export
    // single -> no shutdown -> hang forever

    // Test if the multiple thread tokio runtime could exit successfully when not force flushing logs
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[ignore = "requires --test-threads=1"]
    #[cfg(feature = "rt-tokio")]
    async fn test_create_provider_multiple_thread_tokio() {
        let assert_writer = test_create_provider_in_tokio(runtime::Tokio).await;
        assert_eq!(assert_writer.len(), 0);
    }

    // Test if the multiple thread tokio runtime could exit successfully when force flushing logs
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[ignore = "requires --test-threads=1"]
    #[cfg(feature = "rt-tokio")]
    async fn test_create_provider_multiple_thread_tokio_shutdown() {
        let assert_writer = test_create_provider_in_tokio(runtime::Tokio).await;
        assert!(assert_writer.len() > 0);
    }

    // Test use simple processor in single thread tokio runtime.
    // Expected to see the logs being exported to buffer
    #[tokio::test]
    #[ignore = "requires --test-threads=1"]
    #[cfg(feature = "rt-tokio")]
    async fn test_create_provider_single_thread_tokio_with_simple_processor() {
        use crate::log::LogRecord;

        let assert_writer = AssertWriter::new();
        let provider = build_simple_log_emitter_provider(assert_writer.clone());
        let emitter = provider.log_emitter("opentelemetry");

        emitter.emit(LogRecord::default());

        assert!(assert_writer.len() > 0);
    }

    // Test if the single thread tokio runtime could exit successfully when not force flushing logs
    #[tokio::test]
    #[ignore = "requires --test-threads=1"]
    #[cfg(feature = "rt-tokio-current-thread")]
    async fn test_create_provider_single_thread_tokio() {
        let assert_writer = test_create_provider_in_tokio(runtime::TokioCurrentThread).await;
        assert_eq!(assert_writer.len(), 0)
    }

    // Test if the single thread tokio runtime could exit successfully when force flushing logs
    #[tokio::test]
    #[ignore = "requires --test-threads=1"]
    #[cfg(feature = "rt-tokio-current-thread")]
    async fn test_create_provider_single_thread_tokio_shutdown() {
        let assert_writer = test_create_provider_in_tokio(runtime::TokioCurrentThread).await;
        shutdown_tracer_provider();
        assert!(assert_writer.len() > 0);
    }
}
