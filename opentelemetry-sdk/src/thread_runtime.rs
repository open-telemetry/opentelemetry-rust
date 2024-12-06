//! # Custom Async Runtime with Worker Pool
//!
//! This module provides a custom implementation of an asynchronous runtime that uses a worker pool
//! to manage and execute tasks. It includes support for creating intervals, delays, and running tasks
//! in a controlled, thread-safe environment without relying on external runtime libraries like Tokio or Async-Std.
//!
//! ## Components
//!
//! - **WorkerPool**: A pool of worker threads responsible for executing submitted tasks.
//! - **TimeSchedulers**: Provides functionality for creating interval streams and delay futures.
//! - **CustomInterval**: Represents a stream that produces items at regular intervals.
//! - **CustomDelay**: Represents a future that resolves after a specified delay.
//! - **CustomThreadRuntime**: The main runtime that integrates the worker pool and time scheduling functionalities.
//!
//! ## Features
//!
//! - Task Management: Efficiently manage and execute tasks using a bounded queue.
//! - Interval Scheduling: Create streams that emit items at fixed intervals.
//! - Delays: Create futures that resolve after a specified duration.
//! - Thread Safety: All components are thread-safe and designed for concurrent usage.
//!
//! ## Example Usage
//!
//! ```rust
//! use std::time::Duration;
//! use futures_util::stream::StreamExt;
//! use opentelemetry_sdk::thread_runtime::CustomThreadRuntime;
//! use opentelemetry_sdk::runtime::Runtime; // Import the Runtime trait
//!
//! #[tokio::main] // Using Tokio as the async runtime for the doctest
//! async fn main() {
//!     let runtime = CustomThreadRuntime::new(4, 10); // 4 threads, queue capacity of 10
//!
//!     // Spawn a simple task
//!     runtime.spawn(Box::pin(async {
//!         println!("Hello from a custom runtime!");
//!     }));
//!
//!     // Create a delay
//!     runtime.delay(Duration::from_secs(1)).await;
//!     println!("1 second delay complete");
//!
//!     // Create an interval
//!     let mut interval = runtime.interval(Duration::from_secs(1));
//!     for _ in 0..3 {
//!         interval.next().await;
//!         println!("Tick");
//!     }
//! }
//! ```
//!
//! ## Notes
//! - This implementation is designed for simplicity and does not support advanced async features like I/O.
//! - The worker pool uses blocking threads, so tasks must be CPU-bound or properly batched.
//! - For production use cases, consider using full-fledged async runtimes like Tokio or Async-Std.

use crate::runtime::{Runtime, RuntimeChannel, TrySend, TrySendError};
use async_compat::Compat;
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
    // Sender for sending tasks to worker threads
    // SyncSender is used to send tasks across threads safely
    task_sender: mpsc::SyncSender<Option<BoxFuture<'static, ()>>>,
    num_threads: usize, // Number of worker threads
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
            task_sender,
            num_threads,
        }
    }

    /// Worker loop that runs tasks in worker threads.
    fn worker_loop(task_receiver: Arc<Mutex<mpsc::Receiver<Option<BoxFuture<'static, ()>>>>>) {
        loop {
            let task = task_receiver.lock().unwrap().recv();
            match task {
                Ok(Some(task)) => {
                    // Execute the task
                    futures_executor::block_on(Compat::new(task));
                }
                Ok(None) => {
                    // Shutdown signal received
                    break;
                }
                Err(_) => {
                    // Channel is closed
                    break;
                }
            }
        }
    }

    fn enqueue_task(&self, future: BoxFuture<'static, ()>) -> Result<(), String> {
        match self.task_sender.try_send(Some(future)) {
            Ok(_) => Ok(()), // Successfully enqueued the task
            Err(mpsc::TrySendError::Full(_)) => {
                // Log dropped due to full queue
                eprintln!("Log dropped: Queue is full.");
                Ok(()) // Drop the log and continue without error
            }
            Err(e) => Err(format!("Failed to enqueue task: {:?}", e)), // Handle other errors
        }
    }

    /// Shutdown the worker pool.
    #[allow(dead_code)]
    fn shutdown(&self) {
        // Signal threads to exit and process any remaining tasks
        for _ in 0..self.num_threads {
            let _ = self.task_sender.send(None);
        }
    }
}

/// TimeSchedulers: Manages interval and delay mechanisms.
struct TimeSchedulers;

impl TimeSchedulers {
    /// Create an interval stream that ticks at a given duration.
    fn create_interval(duration: Duration, worker_pool: &WorkerPool) -> CustomInterval {
        let (sender, receiver) = mpsc::channel();
        let task = async move {
            let mut next_tick = Instant::now();
            loop {
                next_tick += duration;
                let now = Instant::now();
                if next_tick > now {
                    thread::sleep(next_tick - now); // Sleep until the next tick
                }
                if sender.send(()).is_err() {
                    break; // Receiver is dropped
                }
            }
        };
        worker_pool.enqueue_task(Box::pin(task)).unwrap();
        CustomInterval { receiver }
    }

    /// Create a delay future that resolves after the given duration.
    fn create_delay(duration: Duration, worker_pool: &WorkerPool) -> CustomDelay {
        let (sender, receiver) = mpsc::channel();

        // Use the WorkerPool to manage the delay task
        let task = async move {
            thread::sleep(duration); // Sleep for the delay duration
            let _ = sender.send(()); // Notify the receiver once the delay is over
        };

        worker_pool.enqueue_task(Box::pin(task)).unwrap();
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
        TimeSchedulers::create_interval(duration, &self.worker_pool)
    }

    fn spawn(&self, future: BoxFuture<'static, ()>) {
        //TODO - handle result
        let _ = self.worker_pool.enqueue_task(future);
    }

    fn delay(&self, duration: Duration) -> Self::Delay {
        TimeSchedulers::create_delay(duration, &self.worker_pool)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_worker_pool_task_execution() {
        // Create a worker pool with 2 threads and a queue capacity of 5
        let worker_pool = WorkerPool::new(2, 5);
        let results = Arc::new(Mutex::new(Vec::new()));

        for i in 0..5 {
            let results = Arc::clone(&results);
            worker_pool
                .enqueue_task(Box::pin(async move {
                    results.lock().unwrap().push(i);
                }))
                .unwrap();
        }

        // Wait for tasks to complete
        thread::sleep(Duration::from_millis(100));

        let mut results = results.lock().unwrap();
        results.sort();
        assert_eq!(*results, vec![0, 1, 2, 3, 4]);

        worker_pool.shutdown();
    }

    #[test]
    fn test_create_interval() {
        // Create a worker pool
        let worker_pool = WorkerPool::new(1, 5);

        // Create an interval that ticks every 100ms
        let interval = TimeSchedulers::create_interval(Duration::from_millis(100), &worker_pool);
        let mut interval_stream = Box::pin(interval);

        // Capture the ticks
        let mut ticks = 0;
        let start = Instant::now();
        while ticks < 5 {
            if let Poll::Ready(Some(_)) = interval_stream.as_mut().poll_next(
                &mut Context::from_waker(futures_util::task::noop_waker_ref()),
            ) {
                ticks += 1;
            }
        }

        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(500));
        worker_pool.shutdown();
    }

    #[test]
    fn test_create_delay() {
        // Create a worker pool
        let worker_pool = WorkerPool::new(1, 5);

        // Create a delay of 100ms
        let delay = TimeSchedulers::create_delay(Duration::from_millis(100), &worker_pool);
        let mut delay_future = Box::pin(delay);

        // Poll the future and measure the time it takes to complete
        let start = Instant::now();
        while let Poll::Pending = delay_future.as_mut().poll(&mut Context::from_waker(
            futures_util::task::noop_waker_ref(),
        )) {}
        let elapsed = start.elapsed();

        assert!(elapsed >= Duration::from_millis(100));
        worker_pool.shutdown();
    }

    #[test]
    fn test_runtime_spawn() {
        let runtime = CustomThreadRuntime::new(2, 5);
        let results = Arc::new(Mutex::new(Vec::new()));

        for i in 0..5 {
            let results = Arc::clone(&results);
            runtime.spawn(Box::pin(async move {
                results.lock().unwrap().push(i);
            }));
        }

        // Wait for tasks to complete
        thread::sleep(Duration::from_millis(100));

        let mut results = results.lock().unwrap();
        results.sort();
        assert_eq!(*results, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_runtime_delay() {
        let runtime = CustomThreadRuntime::new(1, 5);
        let start = Instant::now();

        let delay = runtime.delay(Duration::from_millis(200));
        futures_executor::block_on(delay);

        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(200));
    }

    use hyper::{body::Bytes, Uri};

    use http_body_util::Full;
    use hyper_util::client::legacy::connect::HttpConnector;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    #[test]
    fn test_hyper_request_with_custom_runtime() {
        // Create a CustomThreadRuntime
        let runtime = CustomThreadRuntime::new(2, 5);

        // Shared result holder for the HTTP response status
        let response_status = Arc::new(Mutex::new(None));
        let status_clone = Arc::clone(&response_status);

        // Spawn a task to make an HTTP request using Hyper
        runtime.spawn(Box::pin(async move {
            // Create a client with the default HTTP connector
            let client =
                Client::builder(TokioExecutor::new()).build::<_, Full<Bytes>>(HttpConnector::new());
            let uri: Uri = "http://example.com".parse().unwrap();

            // Send the request
            match client.get(uri).await {
                Ok(response) => {
                    let status = response.status();
                    *status_clone.lock().unwrap() = Some(status.as_u16());
                }
                Err(err) => {
                    eprintln!("Request failed: {}", err);
                }
            }
        }));

        // Wait for the task to complete
        thread::sleep(Duration::from_secs(2));

        // Check the response status
        let status = response_status.lock().unwrap();
        assert!(status.is_some(), "HTTP request did not complete.");
        assert_eq!(status.unwrap(), 200, "Expected HTTP 200 OK.");
    }

    #[test]
    fn test_reqwest_request_with_custom_runtime() {
        // Create a CustomThreadRuntime
        let runtime = CustomThreadRuntime::new(2, 5);

        // Shared result holder for the HTTP response status
        let response_status = Arc::new(Mutex::new(None));
        let status_clone = Arc::clone(&response_status);

        // Spawn a task to make an HTTP request using reqwest
        runtime.spawn(Box::pin(async move {
            // Build the reqwest client
            let client = reqwest::Client::new();
            let uri = "http://example.com";

            // Send the request
            match client.get(uri).send().await {
                Ok(response) => {
                    let status = response.status().as_u16();
                    *status_clone.lock().unwrap() = Some(status);
                }
                Err(err) => {
                    eprintln!("Request failed: {}", err);
                }
            }
        }));

        // Wait for the task to complete
        thread::sleep(Duration::from_secs(2));

        // Check the response status
        let status = response_status.lock().unwrap();
        assert!(status.is_some(), "HTTP request did not complete.");
        assert_eq!(status.unwrap(), 200, "Expected HTTP 200 OK.");
    }

    #[test]
    fn test_reqwest_blocking_request_with_custom_runtime() {
        // Create a CustomThreadRuntime
        let runtime = CustomThreadRuntime::new(2, 5);

        // Shared result holder for the HTTP response status
        let response_status = Arc::new(Mutex::new(None));
        let status_clone = Arc::clone(&response_status);

        // Spawn a task to make an HTTP request using reqwest::blocking
        runtime.spawn(Box::pin(async move {
            // Perform the blocking operation inside a thread
            let handle = thread::spawn(move || {
                // Build the reqwest blocking client
                let client = reqwest::blocking::Client::new();
                let uri = "http://example.com";

                // Send the request
                match client.get(uri).send() {
                    Ok(response) => Some(response.status().as_u16()),
                    Err(err) => {
                        eprintln!("Blocking request failed: {}", err);
                        None
                    }
                }
            });

            // Wait for the thread to finish and collect the result
            if let Ok(result) = handle.join() {
                *status_clone.lock().unwrap() = result;
            }
        }));

        // Wait for the task to complete
        thread::sleep(Duration::from_secs(2));

        // Check the response status
        let status = response_status.lock().unwrap();
        assert!(status.is_some(), "Blocking HTTP request did not complete.");
        assert_eq!(status.unwrap(), 200, "Expected HTTP 200 OK.");
    }
}
