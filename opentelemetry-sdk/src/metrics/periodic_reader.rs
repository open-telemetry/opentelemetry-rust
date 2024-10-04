use core::time;
use std::{
    env, fmt,
    sync::{
        atomic::AtomicBool,
        mpsc::{self, Receiver, Sender},
        Arc, Mutex, Weak,
    },
    thread,
    time::{Duration, Instant},
};

use opentelemetry::metrics::{MetricsError, Result};

use crate::{
    metrics::{exporter::PushMetricsExporter, reader::SdkProducer},
    Resource,
};

use super::{
    data::{ResourceMetrics, Temporality},
    instrument::InstrumentKind,
    reader::{MetricReader, TemporalitySelector},
    Pipeline,
};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_INTERVAL: Duration = Duration::from_secs(60);

const METRIC_EXPORT_INTERVAL_NAME: &str = "OTEL_METRIC_EXPORT_INTERVAL";
const METRIC_EXPORT_TIMEOUT_NAME: &str = "OTEL_METRIC_EXPORT_TIMEOUT";

/// Configuration options for [PeriodicReader].
///
/// A periodic reader is a [MetricReader] that collects and exports metric data
/// to the exporter at a defined interval.
///
/// By default, the returned [MetricReader] will collect and export data every
/// 60 seconds, and will cancel export attempts that exceed 30 seconds. The
/// export time is not counted towards the interval between attempts.
///
/// The [collect] method of the returned [MetricReader] continues to gather and
/// return metric data to the user. It will not automatically send that data to
/// the exporter outside of the predefined interval.
///
/// [collect]: MetricReader::collect
#[derive(Debug)]
pub struct PeriodicReaderBuilder<E> {
    interval: Duration,
    timeout: Duration,
    exporter: E,
}

impl<E> PeriodicReaderBuilder<E>
where
    E: PushMetricsExporter,
{
    fn new(exporter: E) -> Self {
        let interval = env::var(METRIC_EXPORT_INTERVAL_NAME)
            .ok()
            .and_then(|v| v.parse().map(Duration::from_millis).ok())
            .unwrap_or(DEFAULT_INTERVAL);
        let timeout = env::var(METRIC_EXPORT_TIMEOUT_NAME)
            .ok()
            .and_then(|v| v.parse().map(Duration::from_millis).ok())
            .unwrap_or(DEFAULT_TIMEOUT);

        PeriodicReaderBuilder {
            interval,
            timeout,
            exporter,
        }
    }

    /// Configures the intervening time between exports for a [PeriodicReader].
    ///
    /// This option overrides any value set for the `OTEL_METRIC_EXPORT_INTERVAL`
    /// environment variable.
    ///
    /// If this option is not used or `interval` is equal to zero, 60 seconds is
    /// used as the default.
    pub fn with_interval(mut self, interval: Duration) -> Self {
        if !interval.is_zero() {
            self.interval = interval;
        }
        self
    }

    /// Configures the time a [PeriodicReader] waits for an export to complete
    /// before canceling it.
    ///
    /// This option overrides any value set for the `OTEL_METRIC_EXPORT_TIMEOUT`
    /// environment variable.
    ///
    /// If this option is not used or `timeout` is equal to zero, 30 seconds is used
    /// as the default.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        if !timeout.is_zero() {
            self.timeout = timeout;
        }
        self
    }

    /// Create a [PeriodicReader] with the given config.
    pub fn build(self) -> PeriodicReader {
        PeriodicReader::new(self.exporter, self.interval, self.timeout)
    }
}

/// A [MetricReader] that continuously collects and exports metric data at a set
/// interval.
///
/// By default it will collect and export data every 60 seconds, and will cancel
/// export attempts that exceed 30 seconds. The export time is not counted
/// towards the interval between attempts.
///
/// The [collect] method of the returned continues to gather and
/// return metric data to the user. It will not automatically send that data to
/// the exporter outside of the predefined interval.
///
/// The [runtime] can be selected based on feature flags set for this crate.
///
/// The exporter can be any exporter that implements [PushMetricsExporter] such
/// as [opentelemetry-otlp].
///
/// [collect]: MetricReader::collect
/// [runtime]: crate::runtime
/// [opentelemetry-otlp]: https://docs.rs/opentelemetry-otlp/latest/opentelemetry_otlp/
///
/// # Example
///
/// ```no_run
/// use opentelemetry_sdk::metrics::PeriodicReader;
/// # fn example<E, R>(get_exporter: impl Fn() -> E, get_runtime: impl Fn() -> R)
/// # where
/// #     E: opentelemetry_sdk::metrics::exporter::PushMetricsExporter,
/// #     R: opentelemetry_sdk::runtime::Runtime,
/// # {
///
/// let exporter = get_exporter(); // set up a push exporter like OTLP
/// let runtime = get_runtime(); // select runtime: e.g. opentelemetry_sdk:runtime::Tokio
///
/// let reader = PeriodicReader::builder(exporter, runtime).build();
/// # drop(reader);
/// # }
/// ```
#[derive(Clone)]
pub struct PeriodicReader {
    exporter: Arc<dyn PushMetricsExporter>,
    inner: Arc<PeriodicReaderInner>,
}

impl PeriodicReader {
    /// Configuration options for a periodic reader
    pub fn builder<E>(exporter: E) -> PeriodicReaderBuilder<E>
    where
        E: PushMetricsExporter,
    {
        PeriodicReaderBuilder::new(exporter)
    }

    fn new<E>(exporter: E, interval: Duration, timeout: Duration) -> Self
    where
        E: PushMetricsExporter,
    {
        let (message_sender, message_receiver): (Sender<Message>, Receiver<Message>) =
            mpsc::channel();
        let reader = PeriodicReader {
            exporter: Arc::new(exporter),
            inner: Arc::new(PeriodicReaderInner {
                message_sender,
                is_shutdown: AtomicBool::new(false),
                producer: Mutex::new(None),
            }),
        };
        let cloned_reader = reader.clone();

        thread::spawn(move || {
            let mut interval_start = Instant::now();
            let mut remaining_interval = interval;
            println!("PeriodicReader Thread started.");
            loop {
                match message_receiver.recv_timeout(remaining_interval) {
                    Ok(Message::Flush(response_sender)) => {
                        println!("Performing ad-hoc export due to Flush.");
                        if let Err(_e) = cloned_reader.collect_and_export(timeout) {
                            response_sender.send(false).unwrap();
                        } else {
                            response_sender.send(true).unwrap();
                        }

                        // Adjust the remaining interval after the flush
                        let elapsed = interval_start.elapsed();
                        if elapsed < interval {
                            remaining_interval = interval - elapsed;
                            println!(
                                "Adjusting remaining interval after Flush: {:?}",
                                remaining_interval
                            );
                        } else {
                            // Reset the interval if the flush finishes after the expected export time
                            // effectively missing the normal export.
                            // Should we attempt to do the missed export immediately?
                            // Or do the next export at the next interval?
                            // Currently this attempts the next export immediately.
                            // i.e calling Flush can affect the regularity.
                            interval_start = Instant::now();
                            remaining_interval = Duration::ZERO;
                        }
                    }
                    Ok(Message::Shutdown(response_sender)) => {
                        // Perform final export and break out of loop and exit the thread
                        println!("Performing final export and shutting down.");
                        if let Err(_e) = cloned_reader.collect_and_export(timeout) {
                            response_sender.send(false).unwrap();
                        } else {
                            response_sender.send(true).unwrap();
                        }
                        break;
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        let export_start = Instant::now();
                        println!("Performing periodic export at {:?}", export_start);
                        cloned_reader.collect_and_export(timeout).unwrap();

                        let time_taken_for_export = export_start.elapsed();
                        if time_taken_for_export > interval {
                            println!("Export took longer than interval.");
                            // if export took longer than interval, do the 
                            // next export immediately.
                            // Alternatively, we could skip the next export
                            // and wait for the next interval.
                            // Or enforce that export timeout is less than interval.
                            // What is the desired behavior?
                            interval_start = Instant::now();
                            remaining_interval = Duration::ZERO;
                        } else {
                        remaining_interval = interval - time_taken_for_export;
                        interval_start = Instant::now();
                        }
                    }
                    Err(_) => {
                        // Some other error. Break out and exit the thread.
                        break;
                    }
                }
            }
            println!("PeriodicReader Thread stopped.");
        });

        reader
    }

    fn collect_and_export(&self, timeout: Duration) -> Result<()> {
        let mut rm = ResourceMetrics {
            resource: Resource::empty(),
            scope_metrics: Vec::new(),
        };
        self.collect(&mut rm)?;

        // TODO: substract the time taken for collect from the timeout
        // collect involves observable callbacks too, which are user 
        // defined and can take arbitrary time.
        futures_executor::block_on(self.exporter.export(&mut rm, timeout))?;
        Ok(())
    }
}

impl fmt::Debug for PeriodicReader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PeriodicReader").finish()
    }
}

struct PeriodicReaderInner {
    message_sender: mpsc::Sender<Message>,
    producer: Mutex<Option<Weak<dyn SdkProducer>>>,
    is_shutdown: AtomicBool,
}

impl PeriodicReaderInner {
    fn update_producer(&self, producer: Weak<dyn SdkProducer>) {
        let mut inner = self.producer.lock().expect("lock poisoned");
        *inner = Some(producer);
    }
}

#[derive(Debug)]
enum Message {
    Flush(Sender<bool>),
    Shutdown(Sender<bool>),
}

impl TemporalitySelector for PeriodicReader {
    fn temporality(&self, kind: InstrumentKind) -> Temporality {
        self.exporter.temporality(kind)
    }
}

impl MetricReader for PeriodicReader {
    fn register_pipeline(&self, pipeline: Weak<Pipeline>) {
        self.inner.update_producer(pipeline);
    }

    fn collect(&self, rm: &mut ResourceMetrics) -> Result<()> {
        if self
            .inner
            .is_shutdown
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            return Err(MetricsError::Other("reader is shut down".into()));
        }

        let producer = self.inner.producer.lock().expect("lock poisoned");
        if let Some(p) = producer.as_ref() {
            p.upgrade()
                .ok_or_else(|| MetricsError::Other("pipeline is dropped".into()))?
                .produce(rm)?;
            Ok(())
        } else {
            return Err(MetricsError::Other("pipeline is not registered".into()));
        }
    }

    fn force_flush(&self) -> Result<()> {
        if self
            .inner
            .is_shutdown
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            return Err(MetricsError::Other("reader is shut down".into()));
        }
        let (response_tx, response_rx) = mpsc::channel();
        self.inner
            .message_sender
            .send(Message::Flush(response_tx))
            .map_err(|e| MetricsError::Other(e.to_string()))?;

        if let Ok(response) = response_rx.recv() {
            if response {
                return Ok(());
            } else {
                return Err(MetricsError::Other("Failed to flush".into()));
            }
        } else {
            return Err(MetricsError::Other("Failed to flush".into()));
        }
    }

    // TODO: Can we offer async version of shutdown
    // so users can await the shutdown completion,
    // and also to avoid blocking the current thread.
    // The default shutdown on drop can still use blocking
    // call.
    // This could be tricky due to https://github.com/seanmonstar/reqwest/issues/1215#issuecomment-796959957
    fn shutdown(&self) -> Result<()> {
        if self
            .inner
            .is_shutdown
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            return Err(MetricsError::Other("reader is already shut down".into()));
        }

        let (response_tx, response_rx) = mpsc::channel();
        self.inner
            .message_sender
            .send(Message::Shutdown(response_tx))
            .map_err(|e| MetricsError::Other(e.to_string()))?;

        if let Ok(response) = response_rx.recv() {
            self.inner
                .is_shutdown
                .store(true, std::sync::atomic::Ordering::Relaxed);
            if response {
                return Ok(());
            } else {
                return Err(MetricsError::Other("Failed to shutdown".into()));
            }
        } else {
            self.inner
                .is_shutdown
                .store(true, std::sync::atomic::Ordering::Relaxed);
            return Err(MetricsError::Other("Failed to shutdown".into()));
        }
    }
}

#[cfg(all(test, feature = "testing"))]
mod tests {
    #[test]
    fn collection_triggered_by_interval() {}
}
