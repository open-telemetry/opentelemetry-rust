//! Provides an abstraction of several async runtimes
//!
//! This  allows OpenTelemetry to work with any current or future runtime. There are currently
//! builtin implementations for [Tokio] and [async-std].
//!
//! [Tokio]: https://crates.io/crates/tokio
//! [async-std]: https://crates.io/crates/async-std

use futures_util::stream::{unfold, Stream};
use std::{fmt::Debug, future::Future, time::Duration};
use thiserror::Error;

/// A runtime is an abstraction of an async runtime like [Tokio] or [async-std]. It allows
/// OpenTelemetry to work with any current and hopefully future runtime implementations.
///
/// [Tokio]: https://crates.io/crates/tokio
/// [async-std]: https://crates.io/crates/async-std
///
/// # Note
///
/// OpenTelemetry expects a *multithreaded* runtime because its types can move across threads.
/// For this reason, this trait requires the `Send` and `Sync` bounds. Single-threaded runtimes
/// can implement this trait in a way that spawns the tasks on the same thread as the calling code.
#[cfg(feature = "experimental_async_runtime")]
pub trait Runtime: Clone + Send + Sync + 'static {
    /// Spawn a new task or thread, which executes the given future.
    ///
    /// # Note
    ///
    /// This is mainly used to run batch span processing in the background. Note, that the function
    /// does not return a handle. OpenTelemetry will use a different way to wait for the future to
    /// finish when the caller shuts down.
    ///
    /// At the moment, the shutdown happens by blocking the
    /// current thread. This means runtime implementations need to make sure they can still execute
    /// the given future even if the main thread is blocked.
    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static;

    /// Return a future that resolves after the specified [Duration].
    fn delay(&self, duration: Duration) -> impl Future<Output = ()> + Send + 'static;
}

/// Uses the given runtime to produce an interval stream.
#[cfg(feature = "experimental_async_runtime")]
#[allow(dead_code)]
pub(crate) fn to_interval_stream<T: Runtime>(
    runtime: T,
    interval: Duration,
) -> impl Stream<Item = ()> {
    unfold((), move |_| {
        let runtime_cloned = runtime.clone();

        async move {
            runtime_cloned.delay(interval).await;
            Some(((), ()))
        }
    })
}

/// Runtime implementation, which works with Tokio's multi thread runtime.
#[cfg(all(feature = "experimental_async_runtime", feature = "rt-tokio"))]
#[cfg_attr(
    docsrs,
    doc(cfg(all(feature = "experimental_async_runtime", feature = "rt-tokio")))
)]
#[derive(Debug, Clone)]
pub struct Tokio;

#[cfg(all(feature = "experimental_async_runtime", feature = "rt-tokio"))]
#[cfg_attr(
    docsrs,
    doc(cfg(all(feature = "experimental_async_runtime", feature = "rt-tokio")))
)]
impl Runtime for Tokio {
    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        #[allow(clippy::let_underscore_future)]
        // we don't have to await on the returned future to execute
        let _ = tokio::spawn(future);
    }

    fn delay(&self, duration: Duration) -> impl Future<Output = ()> + Send + 'static {
        tokio::time::sleep(duration)
    }
}

/// Runtime implementation, which works with Tokio's current thread runtime.
#[cfg(all(
    feature = "experimental_async_runtime",
    feature = "rt-tokio-current-thread"
))]
#[cfg_attr(
    docsrs,
    doc(cfg(all(
        feature = "experimental_async_runtime",
        feature = "rt-tokio-current-thread"
    )))
)]
#[derive(Debug, Clone)]
pub struct TokioCurrentThread;

#[cfg(all(
    feature = "experimental_async_runtime",
    feature = "rt-tokio-current-thread"
))]
#[cfg_attr(
    docsrs,
    doc(cfg(all(
        feature = "experimental_async_runtime",
        feature = "rt-tokio-current-thread"
    )))
)]
impl Runtime for TokioCurrentThread {
    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        // We cannot force push tracing in current thread tokio scheduler because we rely on
        // BatchSpanProcessor to export spans in a background task, meanwhile we need to block the
        // shutdown function so that the runtime will not finish the blocked task and kill any
        // remaining tasks. But there is only one thread to run task, so it's a deadlock
        //
        // Thus, we spawn the background task in a separate thread.
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("failed to create Tokio current thead runtime for OpenTelemetry batch processing");
            rt.block_on(future);
        });
    }

    fn delay(&self, duration: Duration) -> impl Future<Output = ()> + Send + 'static {
        tokio::time::sleep(duration)
    }
}

/// Runtime implementation, which works with async-std.
#[cfg(all(feature = "experimental_async_runtime", feature = "rt-async-std"))]
#[cfg_attr(
    docsrs,
    doc(cfg(all(feature = "experimental_async_runtime", feature = "rt-async-std")))
)]
#[derive(Debug, Clone)]
pub struct AsyncStd;

#[cfg(all(feature = "experimental_async_runtime", feature = "rt-async-std"))]
#[cfg_attr(
    docsrs,
    doc(cfg(all(feature = "experimental_async_runtime", feature = "rt-async-std")))
)]
impl Runtime for AsyncStd {
    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        #[allow(clippy::let_underscore_future)]
        let _ = async_std::task::spawn(future);
    }

    fn delay(&self, duration: Duration) -> impl Future<Output = ()> + Send + 'static {
        async_std::task::sleep(duration)
    }
}

/// `RuntimeChannel` is an extension to [`Runtime`]. Currently, it provides a
/// channel that is used by the [log] and [span] batch processors.
///
/// [log]: crate::logs::BatchLogProcessor
/// [span]: crate::trace::BatchSpanProcessor
#[cfg(feature = "experimental_async_runtime")]
pub trait RuntimeChannel: Runtime {
    /// A future stream to receive batch messages from channels.
    type Receiver<T: Debug + Send>: Stream<Item = T> + Send;
    /// A batch messages sender that can be sent across threads safely.
    type Sender<T: Debug + Send>: TrySend<Message = T> + Debug;

    /// Return the sender and receiver used to send batch messages.
    fn batch_message_channel<T: Debug + Send>(
        &self,
        capacity: usize,
    ) -> (Self::Sender<T>, Self::Receiver<T>);
}

/// Error returned by a [`TrySend`] implementation.
#[cfg(feature = "experimental_async_runtime")]
#[derive(Debug, Error)]
pub enum TrySendError {
    /// Send failed due to the channel being full.
    #[error("cannot send message to batch processor as the channel is full")]
    ChannelFull,
    /// Send failed due to the channel being closed.
    #[error("cannot send message to batch processor as the channel is closed")]
    ChannelClosed,
    /// Any other send error that isn't covered above.
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

/// TrySend is an abstraction of `Sender` that is capable of sending messages through a reference.
#[cfg(feature = "experimental_async_runtime")]
pub trait TrySend: Sync + Send {
    /// The message that will be sent.
    type Message;

    /// Try to send a message batch to a worker thread.
    ///
    /// A failure can be due to either a closed receiver, or a depleted buffer.
    fn try_send(&self, item: Self::Message) -> Result<(), TrySendError>;
}

#[cfg(all(
    feature = "experimental_async_runtime",
    any(feature = "rt-tokio", feature = "rt-tokio-current-thread")
))]
impl<T: Send> TrySend for tokio::sync::mpsc::Sender<T> {
    type Message = T;

    fn try_send(&self, item: Self::Message) -> Result<(), TrySendError> {
        self.try_send(item).map_err(|err| match err {
            tokio::sync::mpsc::error::TrySendError::Full(_) => TrySendError::ChannelFull,
            tokio::sync::mpsc::error::TrySendError::Closed(_) => TrySendError::ChannelClosed,
        })
    }
}

#[cfg(all(feature = "experimental_async_runtime", feature = "rt-tokio"))]
#[cfg_attr(
    docsrs,
    doc(cfg(all(feature = "experimental_async_runtime", feature = "rt-tokio")))
)]
impl RuntimeChannel for Tokio {
    type Receiver<T: Debug + Send> = tokio_stream::wrappers::ReceiverStream<T>;
    type Sender<T: Debug + Send> = tokio::sync::mpsc::Sender<T>;

    fn batch_message_channel<T: Debug + Send>(
        &self,
        capacity: usize,
    ) -> (Self::Sender<T>, Self::Receiver<T>) {
        let (sender, receiver) = tokio::sync::mpsc::channel(capacity);
        (
            sender,
            tokio_stream::wrappers::ReceiverStream::new(receiver),
        )
    }
}

#[cfg(all(
    feature = "experimental_async_runtime",
    feature = "rt-tokio-current-thread"
))]
#[cfg_attr(
    docsrs,
    doc(cfg(all(
        feature = "experimental_async_runtime",
        feature = "rt-tokio-current-thread"
    )))
)]
impl RuntimeChannel for TokioCurrentThread {
    type Receiver<T: Debug + Send> = tokio_stream::wrappers::ReceiverStream<T>;
    type Sender<T: Debug + Send> = tokio::sync::mpsc::Sender<T>;

    fn batch_message_channel<T: Debug + Send>(
        &self,
        capacity: usize,
    ) -> (Self::Sender<T>, Self::Receiver<T>) {
        let (sender, receiver) = tokio::sync::mpsc::channel(capacity);
        (
            sender,
            tokio_stream::wrappers::ReceiverStream::new(receiver),
        )
    }
}

#[cfg(all(feature = "experimental_async_runtime", feature = "rt-async-std"))]
impl<T: Send> TrySend for async_std::channel::Sender<T> {
    type Message = T;

    fn try_send(&self, item: Self::Message) -> Result<(), TrySendError> {
        self.try_send(item).map_err(|err| match err {
            async_std::channel::TrySendError::Full(_) => TrySendError::ChannelFull,
            async_std::channel::TrySendError::Closed(_) => TrySendError::ChannelClosed,
        })
    }
}

#[cfg(all(feature = "experimental_async_runtime", feature = "rt-async-std"))]
#[cfg_attr(
    docsrs,
    doc(cfg(all(feature = "experimental_async_runtime", feature = "rt-async-std")))
)]
impl RuntimeChannel for AsyncStd {
    type Receiver<T: Debug + Send> = async_std::channel::Receiver<T>;
    type Sender<T: Debug + Send> = async_std::channel::Sender<T>;

    fn batch_message_channel<T: Debug + Send>(
        &self,
        capacity: usize,
    ) -> (Self::Sender<T>, Self::Receiver<T>) {
        async_std::channel::bounded(capacity)
    }
}
