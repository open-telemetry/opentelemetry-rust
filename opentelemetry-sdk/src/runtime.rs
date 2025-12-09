//! Provides an abstraction of several async runtimes
//!
//! This  allows OpenTelemetry to work with any current or future runtime. There is currently
//! built-in implementation for [Tokio].
//!
//! [Tokio]: https://crates.io/crates/tokio

use futures_util::stream::{unfold, Stream};
use std::{fmt::Debug, future::Future, time::Duration};
use thiserror::Error;

/// A runtime is an abstraction of an async runtime like [Tokio]. It allows
/// OpenTelemetry to work with any current and hopefully future runtime implementations.
///
/// [Tokio]: https://crates.io/crates/tokio
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
    /// This is mainly used to run batch span processing in the background. The mechanism used
    /// to provide the spawn may be relatively heavyweight.  Note, that the function
    /// does not return a handle. OpenTelemetry will use a different way to wait for the future to
    /// finish when the caller shuts down.
    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static;

    /// Block the current thread waiting for a future to complete.
    fn block_on<F, T>(&self, future: F) -> T
    where
        F: Future<Output = T> + Send,
        T: Send;

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

/// Runtime implementation for Tokio.
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
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            match handle.runtime_flavor() {
                tokio::runtime::RuntimeFlavor::CurrentThread => {
                    // Single-threaded runtime: spawn on a separate OS thread with its
                    // own tokio runtime to avoid deadlocks. We can't use the existing
                    // handle because current_thread runtimes can only be driven from
                    // their original thread.
                    // We can't drive the future from a thread _without_ tokio as
                    // it may be doing tokio specific things (like wait).
                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .expect("failed to create tokio runtime");
                        rt.block_on(future);
                    });
                }
                _ => {
                    // Multi-threaded runtime: use tokio::spawn
                    #[allow(clippy::let_underscore_future)]
                    let _ = tokio::spawn(future);
                }
            }
        } else {
            // No tokio runtime context: create a new runtime on an OS thread.
            // As above, we can't drive the future without tokio, as it may be
            // be doing tokio-specific things like like tokio::time::sleep.
            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("failed to create tokio runtime");
                rt.block_on(future);
            });
        }
    }

    fn block_on<F, T>(&self, future: F) -> T
    where
        F: Future<Output = T> + Send,
        T: Send,
    {
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            match handle.runtime_flavor() {
                tokio::runtime::RuntimeFlavor::CurrentThread => {
                    // Single-threaded runtime: block on a separate thread with its
                    // own tokio runtime to avoid deadlocks.
                    std::thread::scope(|s| {
                        s.spawn(move || {
                            let rt = tokio::runtime::Builder::new_current_thread()
                                .enable_all()
                                .build()
                                .expect("failed to create tokio runtime");
                            rt.block_on(future)
                        })
                        .join()
                        .unwrap()
                    })
                }
                _ => {
                    // Multi-threaded runtime: block directly
                    futures_executor::block_on(future)
                }
            }
        } else {
            // No tokio runtime context: create a runtime to handle the future
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("failed to create tokio runtime");
            rt.block_on(future)
        }
    }

    fn delay(&self, duration: Duration) -> impl Future<Output = ()> + Send + 'static {
        tokio::time::sleep(duration)
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

#[cfg(all(feature = "experimental_async_runtime", feature = "rt-tokio"))]
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

/// Runtime implementation for synchronous execution environments.
///
/// This runtime can be used when executing in a non-async environment.
/// The runtime methods will perform their operations synchronously.
#[cfg(feature = "experimental_async_runtime")]
#[derive(Debug, Clone, Copy)]
pub struct NoAsync;

#[cfg(feature = "experimental_async_runtime")]
impl Runtime for NoAsync {
    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        std::thread::spawn(move || {
            futures_executor::block_on(future);
        });
    }

    fn block_on<F, T>(&self, future: F) -> T
    where
        F: Future<Output = T> + Send,
        T: Send,
    {
        futures_executor::block_on(future)
    }

    // Needed because async fn would borrow `self`, violating the `'static` requirement.
    #[allow(clippy::manual_async_fn)]
    fn delay(&self, duration: Duration) -> impl Future<Output = ()> + Send + 'static {
        async move {
            std::thread::sleep(duration);
        }
    }
}
