use crate::runtime::{Runtime, RuntimeChannel, TrySend, TrySendError};
use futures_executor;
use futures_util::{future::BoxFuture, stream::Stream};
use std::{
    fmt::Debug,
    future::Future,
    pin::Pin,
    sync::{mpsc, Arc, Mutex},
    task::{Context, Poll},
    thread,
    time::{Duration, Instant},
};

/// WorkerPool: Manages worker threads to process tasks.
#[derive(Clone, Debug)]
struct WorkerPool {
    task_sender: Arc<Mutex<mpsc::SyncSender<BoxFuture<'static, ()>>>>,
}

impl WorkerPool {
    /// Create a new WorkerPool with the specified number of worker threads.
    fn new(num_threads: usize, queue_capacity: usize) -> Self {
        let (task_sender, task_receiver) = mpsc::sync_channel(queue_capacity);
        let task_receiver = Arc::new(Mutex::new(task_receiver));

        // Spawn worker threads
        for _ in 0..num_threads {
            let task_receiver = Arc::clone(&task_receiver);
            thread::spawn(move || Self::worker_loop(task_receiver));
        }

        WorkerPool {
            task_sender: Arc::new(Mutex::new(task_sender)),
        }
    }

    /// Worker loop that runs tasks in worker threads.
    fn worker_loop(task_receiver: Arc<Mutex<mpsc::Receiver<BoxFuture<'static, ()>>>>) {
        loop {
            let task = task_receiver.lock().unwrap().recv();
            if let Ok(task) = task {
                // Block on task execution.
                futures_executor::block_on(task);
            } else {
                break; // Exit the loop when the sender is closed
            }
        }
    }

    /// Enqueue a new task for execution.
    fn enqueue_task(&self, future: BoxFuture<'static, ()>) {
        let task_sender = Arc::clone(&self.task_sender);
        let sender = task_sender.lock().unwrap();
        sender.send(future).unwrap();
    }

    /// Shutdown the worker pool.
    fn shutdown(&self) {
        // Signal threads to exit and process any remaining tasks
        drop(self.task_sender.lock().unwrap().clone());
    }
}

/// TimeSchedulers: Manages interval and delay mechanisms.
struct TimeSchedulers;

impl TimeSchedulers {
    /// Create an interval stream that ticks at a given duration.
    fn create_interval(duration: Duration) -> CustomInterval {
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let mut next_tick = Instant::now();
            loop {
                next_tick += duration;
                if sender.send(()).is_err() {
                    break;
                }
                let now = Instant::now();
                if next_tick > now {
                    thread::sleep(next_tick - now);
                }
            }
        });
        CustomInterval { receiver }
    }

    /// Create a delay future that resolves after the given duration.
    fn create_delay(duration: Duration) -> CustomDelay {
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            thread::sleep(duration);
            let _ = sender.send(());
        });
        CustomDelay { receiver }
    }
}

/// CustomInterval: A stream that ticks at fixed intervals using a background thread.
#[derive(Debug)]
pub struct CustomInterval {
    receiver: mpsc::Receiver<()>,
}

impl Stream for CustomInterval {
    type Item = ();

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.receiver.recv() {
            Ok(_) => Poll::Ready(Some(())),
            Err(_) => Poll::Ready(None),
        }
    }
}

/// CustomDelay: A future that resolves after a fixed delay using a background thread.
#[derive(Debug)]
pub struct CustomDelay {
    receiver: mpsc::Receiver<()>,
}

impl Future for CustomDelay {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.receiver.recv() {
            Ok(_) => Poll::Ready(()),
            Err(_) => Poll::Ready(()),
        }
    }
}

/// CustomThreadRuntime: Combines worker pool and time schedulers to manage tasks and timers.
#[derive(Debug, Clone)]
pub struct CustomThreadRuntime {
    worker_pool: WorkerPool,
}

impl CustomThreadRuntime {
    /// Create a new CustomThreadRuntime with the specified number of worker threads.
    pub fn new(num_threads: usize, queue_capacity: usize) -> Self {
        CustomThreadRuntime {
            worker_pool: WorkerPool::new(num_threads, queue_capacity),
        }
    }
}

impl Runtime for CustomThreadRuntime {
    type Interval = CustomInterval;
    type Delay = CustomDelay;

    fn interval(&self, duration: Duration) -> Self::Interval {
        TimeSchedulers::create_interval(duration)
    }

    fn spawn(&self, future: BoxFuture<'static, ()>) {
        self.worker_pool.enqueue_task(future);
    }

    fn delay(&self, duration: Duration) -> Self::Delay {
        TimeSchedulers::create_delay(duration)
    }
}

/// Messaging system for sending batch messages.
#[derive(Debug)]
pub struct CustomSender<T: Debug + Send> {
    tx: mpsc::SyncSender<T>,
}

/// Messaging system for receiving batch messages.
#[derive(Debug)]
pub struct CustomReceiver<T: Debug + Send> {
    rx: mpsc::Receiver<T>,
}

impl<T: Debug + Send> TrySend for CustomSender<T> {
    type Message = T;

    fn try_send(&self, item: Self::Message) -> Result<(), TrySendError> {
        self.tx.send(item).map_err(|_| TrySendError::ChannelClosed)
    }
}

impl<T: Debug + Send> Stream for CustomReceiver<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Use `try_recv` instead of `recv` to avoid blocking
        match self.rx.try_recv() {
            Ok(item) => Poll::Ready(Some(item)),
            Err(mpsc::TryRecvError::Empty) => {
                // No message is available yet, so we'll return `Poll::Pending`
                // and recheck after a short sleep to avoid busy-waiting.
                thread::sleep(Duration::from_millis(10)); // Adjust sleep duration if needed
                Poll::Pending
            }
            Err(mpsc::TryRecvError::Disconnected) => Poll::Ready(None), // Channel is closed, terminate the stream
        }
    }
}

impl RuntimeChannel for CustomThreadRuntime {
    type Receiver<T: Debug + Send> = CustomReceiver<T>;
    type Sender<T: Debug + Send> = CustomSender<T>;

    fn batch_message_channel<T: Debug + Send>(
        &self,
        capacity: usize,
    ) -> (Self::Sender<T>, Self::Receiver<T>) {
        // Use mpsc to create a bounded channel
        let (tx, rx) = mpsc::sync_channel(capacity);
        (
            CustomSender { tx },   // Sender part
            CustomReceiver { rx }, // Receiver part
        )
    }
}
