//! Provides an abstraction of several async runtimes
//!
//! This  allows OpenTelemetry to work with any current or future runtime. There are currently
//! builtin implementations for [Tokio] and [async-std].
//!
//! [Tokio]: https://crates.io/crates/tokio
//! [async-std]: https://crates.io/crates/async-std

use futures_util::{future::BoxFuture, stream::Stream};
use std::{fmt::Debug, future::Future};
use thiserror::Error;
use std::{
    sync::{ Arc, Mutex},
    thread,
    time::Duration,
};
use std::pin::Pin;

use futures_util::task::{Context, Poll};
//use std::{future::Future, time::Duration};
use futures_executor;
use crossbeam_channel::{self, Sender as CrossbeamSender, Receiver as CrossbeamReceiver};
//use std::task::{Context, Poll};


/// A runtime is an abstraction of an async runtime like [Tokio] or [async-std]. It allows
/// OpenTelemetry to work with any current and hopefully future runtime implementation.
///
/// [Tokio]: https://crates.io/crates/tokio
/// [async-std]: https://crates.io/crates/async-std
pub trait Runtime: Clone + Send + Sync + 'static {
    /// A future stream, which returns items in a previously specified interval. The item type is
    /// not important.
    type Interval: Stream + Send;

    /// A future, which resolves after a previously specified amount of time. The output type is
    /// not important.
    type Delay: Future + Send + Unpin;

    /// Create a [futures_util::stream::Stream], which returns a new item every
    /// [std::time::Duration].
    fn interval(&self, duration: Duration) -> Self::Interval;

    /// Spawn a new task or thread, which executes the given future.
    ///
    /// # Note
    ///
    /// This is mainly used to run batch span processing in the background. Note, that the function
    /// does not return a handle. OpenTelemetry will use a different way to wait for the future to
    /// finish when TracerProvider gets shutdown. At the moment this happens by blocking the
    /// current thread. This means runtime implementations need to make sure they can still execute
    /// the given future even if the main thread is blocked.
    fn spawn(&self, future: BoxFuture<'static, ()>);

    /// Return a new future, which resolves after the specified [std::time::Duration].
    fn delay(&self, duration: Duration) -> Self::Delay;
}


/// Runtime implementation, which works with Tokio's multi thread runtime.
#[cfg(feature = "rt-tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "rt-tokio")))]
#[derive(Debug, Clone)]
pub struct Tokio;

#[cfg(feature = "rt-tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "rt-tokio")))]
impl Runtime for Tokio {
    type Interval = tokio_stream::wrappers::IntervalStream;
    type Delay = ::std::pin::Pin<Box<tokio::time::Sleep>>;

    fn interval(&self, duration: Duration) -> Self::Interval {
        crate::util::tokio_interval_stream(duration)
    }

    fn spawn(&self, future: BoxFuture<'static, ()>) {
        #[allow(clippy::let_underscore_future)]
        // we don't have to await on the returned future to execute
        let _ = tokio::spawn(future);
    }

    fn delay(&self, duration: Duration) -> Self::Delay {
        Box::pin(tokio::time::sleep(duration))
    }
}

/// Runtime implementation, which works with Tokio's current thread runtime.
#[cfg(feature = "rt-tokio-current-thread")]
#[cfg_attr(docsrs, doc(cfg(feature = "rt-tokio-current-thread")))]
#[derive(Debug, Clone)]
pub struct TokioCurrentThread;

#[cfg(feature = "rt-tokio-current-thread")]
#[cfg_attr(docsrs, doc(cfg(feature = "rt-tokio-current-thread")))]
impl Runtime for TokioCurrentThread {
    type Interval = tokio_stream::wrappers::IntervalStream;
    type Delay = ::std::pin::Pin<Box<tokio::time::Sleep>>;

    fn interval(&self, duration: Duration) -> Self::Interval {
        crate::util::tokio_interval_stream(duration)
    }

    fn spawn(&self, future: BoxFuture<'static, ()>) {
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

    fn delay(&self, duration: Duration) -> Self::Delay {
        Box::pin(tokio::time::sleep(duration))
    }
}

/// Runtime implementation, which works with async-std.
#[cfg(feature = "rt-async-std")]
#[cfg_attr(docsrs, doc(cfg(feature = "rt-async-std")))]
#[derive(Debug, Clone)]
pub struct AsyncStd;

#[cfg(feature = "rt-async-std")]
#[cfg_attr(docsrs, doc(cfg(feature = "rt-async-std")))]
impl Runtime for AsyncStd {
    type Interval = async_std::stream::Interval;
    type Delay = BoxFuture<'static, ()>;

    fn interval(&self, duration: Duration) -> Self::Interval {
        async_std::stream::interval(duration)
    }

    fn spawn(&self, future: BoxFuture<'static, ()>) {
        #[allow(clippy::let_underscore_future)]
        let _ = async_std::task::spawn(future);
    }

    fn delay(&self, duration: Duration) -> Self::Delay {
        Box::pin(async_std::task::sleep(duration))
    }
}

/// `RuntimeChannel` is an extension to [`Runtime`]. Currently, it provides a
/// channel that is used by the [log] and [span] batch processors.
///
/// [log]: crate::logs::BatchLogProcessor
/// [span]: crate::trace::BatchSpanProcessor
pub trait RuntimeChannel: Runtime {
    /// A future stream to receive batch messages from channels.
    type Receiver<T: Debug + Send + 'static>: Stream<Item = T> + Send;
    /// A batch messages sender that can be sent across threads safely.
    type Sender<T: Debug + Send + 'static>: TrySend<Message = T> + Debug;

    /// Return the sender and receiver used to send batch messages.
    fn batch_message_channel<T: Debug + Send + 'static>(
        &self,
        capacity: usize,
    ) -> (Self::Sender<T>, Self::Receiver<T>);
}

/// Error returned by a [`TrySend`] implementation.
#[derive(Debug, Error)]
pub enum TrySendError {
    /// Send failed due to the channel being full.
    #[error("cannot send message to batch processor as the channel is full")]
    ChannelFull,
    /// Send failed due to the channel being closed.
    #[error("cannot send message to batch processor as the channel is closed")]
    ChannelClosed,
    /// Any other send error that isnt covered above.
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

/// TrySend is an abstraction of `Sender` that is capable of sending messages through a reference.
pub trait TrySend: Sync + Send {
    /// The message that will be sent.
    type Message;

    /// Try to send a message batch to a worker thread.
    ///
    /// A failure can be due to either a closed receiver, or a depleted buffer.
    fn try_send(&self, item: Self::Message) -> Result<(), TrySendError>;
}

#[cfg(any(feature = "rt-tokio", feature = "rt-tokio-current-thread"))]
impl<T: Send> TrySend for tokio::sync::mpsc::Sender<T> {
    type Message = T;

    fn try_send(&self, item: Self::Message) -> Result<(), TrySendError> {
        self.try_send(item).map_err(|err| match err {
            tokio::sync::mpsc::error::TrySendError::Full(_) => TrySendError::ChannelFull,
            tokio::sync::mpsc::error::TrySendError::Closed(_) => TrySendError::ChannelClosed,
        })
    }
}

#[cfg(feature = "rt-tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "rt-tokio")))]
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

#[cfg(feature = "rt-tokio-current-thread")]
#[cfg_attr(docsrs, doc(cfg(feature = "rt-tokio-current-thread")))]
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

#[cfg(feature = "rt-async-std")]
impl<T: Send> TrySend for async_std::channel::Sender<T> {
    type Message = T;

    fn try_send(&self, item: Self::Message) -> Result<(), TrySendError> {
        self.try_send(item).map_err(|err| match err {
            async_std::channel::TrySendError::Full(_) => TrySendError::ChannelFull,
            async_std::channel::TrySendError::Closed(_) => TrySendError::ChannelClosed,
        })
    }
}

#[cfg(feature = "rt-async-std")]
#[cfg_attr(docsrs, doc(cfg(feature = "rt-async-std")))]
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

/// stdthreadtuntime
#[derive(Debug, Clone)]
pub struct StdThreadRuntime {
    shutdown_signal: Arc<Mutex<bool>>,
}

impl StdThreadRuntime {
    /// new
    pub fn new() -> Self {
        StdThreadRuntime {
            shutdown_signal: Arc::new(Mutex::new(false)),
        }
    }
}

impl Runtime for StdThreadRuntime {
    type Interval = StdInterval;
    type Delay = StdDelay;

    fn interval(&self, duration: Duration) -> Self::Interval {
        StdInterval::new(duration, self.shutdown_signal.clone())
    }

    fn spawn(&self, future: BoxFuture<'static, ()>) {
        thread::spawn(move || {
            futures_executor::block_on(future);
        });
    }

    fn delay(&self, duration: Duration) -> Self::Delay {
        StdDelay::new(duration)
    }
}

/// stdinterval
#[derive(Debug)]
pub struct StdInterval {
    duration: Duration,
    shutdown_signal: Arc<Mutex<bool>>,
}

impl StdInterval {
    /// new
    pub fn new(duration: Duration, shutdown_signal: Arc<Mutex<bool>>) -> Self {
        StdInterval { duration, shutdown_signal }
    }
}

impl Stream for StdInterval {
    type Item = ();

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if *self.shutdown_signal.lock().unwrap() {
            Poll::Ready(None)
        } else {
            thread::sleep(self.duration);
            cx.waker().wake_by_ref();
            Poll::Ready(Some(()))
        }
    }
}

/// stddelay
#[derive(Debug)]
pub struct StdDelay {
    duration: Duration,
    elapsed: bool,
}

impl StdDelay {
    /// new
    pub fn new(duration: Duration) -> Self {
        StdDelay { duration, elapsed: false }
    }
}

impl Future for StdDelay {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.elapsed {
            Poll::Ready(())
        } else {
            self.elapsed = true;
            cx.waker().wake_by_ref();
            thread::sleep(self.duration);
            Poll::Pending
        }
    }
}


impl RuntimeChannel for StdThreadRuntime {
    type Receiver<T: Debug + Send + 'static> = CrossbeamReceiverStream<T>;
    type Sender<T: Debug + Send + 'static> = Arc<CrossbeamSender<T>>;

    fn batch_message_channel<T: Debug + Send + 'static>(
        &self,
        capacity: usize,
    ) -> (Self::Sender<T>, Self::Receiver<T>) {
        let (sender, receiver) = crossbeam_channel::bounded(capacity);
        (Arc::new(sender), CrossbeamReceiverStream::new(receiver))
    }
}

/// crossbeamreceiverstream
#[derive(Debug)]
pub struct CrossbeamReceiverStream<T> {
    receiver: CrossbeamReceiver<T>,
}

impl<T> CrossbeamReceiverStream<T> {
    /// new
    pub fn new(receiver: CrossbeamReceiver<T>) -> Self {
        CrossbeamReceiverStream { receiver }
    }
}

impl<T: Send + 'static> Stream for CrossbeamReceiverStream<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.receiver.try_recv() {
            Ok(item) => Poll::Ready(Some(item)),
            Err(crossbeam_channel::TryRecvError::Empty) => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Err(crossbeam_channel::TryRecvError::Disconnected) => Poll::Ready(None),
        }
    }
}

impl<T: Send> TrySend for Arc<CrossbeamSender<T>> {
    type Message = T;

    fn try_send(&self, item: Self::Message) -> Result<(), TrySendError> {
        self.send(item).map_err(|_| TrySendError::ChannelClosed)
    }
}


