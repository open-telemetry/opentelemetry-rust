//! # Log Runtime
//! Log runtime is an extension to [`Runtime`]. Currently it provides a channel that used
//! by [`BatchLogProcessor`].
//!
//! [`BatchLogProcessor`]: crate::logs::BatchLogProcessor
//! [`Runtime`]: crate::runtime::Runtime
use crate::logs::BatchMessage;
#[cfg(feature = "rt-async-std")]
use crate::runtime::AsyncStd;
use crate::runtime::Runtime;
#[cfg(feature = "rt-tokio")]
use crate::runtime::Tokio;
#[cfg(feature = "rt-tokio-current-thread")]
use crate::runtime::TokioCurrentThread;
use futures_util::stream::Stream;
use opentelemetry_api::logs::LogError;
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
/// [`BatchLogProcessor`]: crate::logs::BatchLogProcessor
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
