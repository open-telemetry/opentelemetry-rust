use std::{
    env, fmt,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex, Weak,
    },
    thread,
    time::{Duration, Instant},
};

use opentelemetry::{otel_debug, otel_error, otel_info, otel_warn};

use crate::{
    error::{ShutdownError, ShutdownResult},
    metrics::{exporter::PushMetricExporter, reader::SdkProducer, MetricError, MetricResult},
    Resource,
};

use super::{
    data::ResourceMetrics, instrument::InstrumentKind, reader::MetricReader, Pipeline, Temporality,
};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_INTERVAL: Duration = Duration::from_secs(60);

const METRIC_EXPORT_INTERVAL_NAME: &str = "OTEL_METRIC_EXPORT_INTERVAL";
const METRIC_EXPORT_TIMEOUT_NAME: &str = "OTEL_METRIC_EXPORT_TIMEOUT";

/// Configuration options for [PeriodicReader].
#[derive(Debug)]
pub struct PeriodicReaderBuilder<E> {
    interval: Duration,
    timeout: Duration,
    exporter: E,
}

impl<E> PeriodicReaderBuilder<E>
where
    E: PushMetricExporter,
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

    /// Configures the timeout for an export to complete. PeriodicReader itself
    /// does not enforce timeout. Instead timeout is passed on to the exporter
    /// for each export attempt.
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

/// A [MetricReader] that continuously collects and exports metrics at a set
/// interval.
///
/// By default, `PeriodicReader` will collect and export metrics every 60
/// seconds. The export time is not counted towards the interval between
/// attempts. `PeriodicReader` itself does not enforce a timeout. Instead, the
/// timeout is passed on to the configured exporter for each export attempt.
///
/// `PeriodicReader` spawns a background thread to handle the periodic
/// collection and export of metrics. The background thread will continue to run
/// until `shutdown()` is called.
///
/// When using this reader with the OTLP Exporter, the following exporter
/// features are supported:
/// - `grpc-tonic`: This requires `MeterProvider` to be created within a tokio
///   runtime.
/// - `reqwest-blocking-client`: Works with a regular `main` or `tokio::main`.
///
/// In other words, other clients like `reqwest` and `hyper` are not supported.
///
/// # Example
///
/// ```no_run
/// use opentelemetry_sdk::metrics::PeriodicReader;
/// # fn example<E>(get_exporter: impl Fn() -> E)
/// # where
/// #     E: opentelemetry_sdk::metrics::exporter::PushMetricExporter,
/// # {
///
/// let exporter = get_exporter(); // set up a push exporter
///
/// let reader = PeriodicReader::builder(exporter).build();
/// # drop(reader);
/// # }
/// ```
#[derive(Clone)]
pub struct PeriodicReader {
    inner: Arc<PeriodicReaderInner>,
}

impl PeriodicReader {
    /// Configuration options for a periodic reader with own thread
    pub fn builder<E>(exporter: E) -> PeriodicReaderBuilder<E>
    where
        E: PushMetricExporter,
    {
        PeriodicReaderBuilder::new(exporter)
    }

    fn new<E>(exporter: E, interval: Duration, timeout: Duration) -> Self
    where
        E: PushMetricExporter,
    {
        let (message_sender, message_receiver): (Sender<Message>, Receiver<Message>) =
            mpsc::channel();
        let exporter_arc = Arc::new(exporter);
        let reader = PeriodicReader {
            inner: Arc::new(PeriodicReaderInner {
                message_sender,
                producer: Mutex::new(None),
                exporter: exporter_arc.clone(),
            }),
        };
        let cloned_reader = reader.clone();

        let result_thread_creation = thread::Builder::new()
            .name("OpenTelemetry.Metrics.PeriodicReader".to_string())
            .spawn(move || {
                let mut interval_start = Instant::now();
                let mut remaining_interval = interval;
                otel_info!(
                    name: "PeriodReaderThreadStarted",
                    interval_in_millisecs = interval.as_millis(),
                    timeout_in_millisecs = timeout.as_millis()
                );
                loop {
                    otel_debug!(
                        name: "PeriodReaderThreadLoopAlive", message = "Next export will happen after interval, unless flush or shutdown is triggered.", interval_in_millisecs = remaining_interval.as_millis()
                    );
                    match message_receiver.recv_timeout(remaining_interval) {
                        Ok(Message::Flush(response_sender)) => {
                            otel_debug!(
                                name: "PeriodReaderThreadExportingDueToFlush"
                            );
                            if let Err(_e) = cloned_reader.collect_and_export(timeout) {
                                response_sender.send(false).unwrap();
                            } else {
                                response_sender.send(true).unwrap();
                            }

                            // Adjust the remaining interval after the flush
                            let elapsed = interval_start.elapsed();
                            if elapsed < interval {
                                remaining_interval = interval - elapsed;
                                otel_debug!(
                                    name: "PeriodReaderThreadAdjustingRemainingIntervalAfterFlush",
                                    remaining_interval = remaining_interval.as_secs()
                                );
                            } else {
                                otel_debug!(
                                    name: "PeriodReaderThreadAdjustingExportAfterFlush",
                                );
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
                            otel_debug!(name: "PeriodReaderThreadExportingDueToShutdown");
                            let export_result = cloned_reader.collect_and_export(timeout);
                            let shutdown_result = exporter_arc.shutdown();
                            otel_debug!(
                                name: "PeriodReaderInvokedExporterShutdown",
                                shutdown_result = format!("{:?}", shutdown_result)
                            );
                            if export_result.is_err() || shutdown_result.is_err() {
                                response_sender.send(false).unwrap();
                            } else {
                                response_sender.send(true).unwrap();
                            }

                            otel_debug!(
                                name: "PeriodReaderThreadExiting",
                                reason = "ShutdownRequested"
                            );
                            break;
                        }
                        Err(mpsc::RecvTimeoutError::Timeout) => {
                            let export_start = Instant::now();
                            otel_debug!(
                                name: "PeriodReaderThreadExportingDueToTimer"
                            );

                            if let Err(_e) = cloned_reader.collect_and_export(timeout) {
                                otel_debug!(
                                    name: "PeriodReaderThreadExportingDueToTimerFailed"
                                );
                            }

                            let time_taken_for_export = export_start.elapsed();
                            if time_taken_for_export > interval {
                                otel_debug!(
                                    name: "PeriodReaderThreadExportTookLongerThanInterval"
                                );
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
                        Err(mpsc::RecvTimeoutError::Disconnected) => {
                            // Channel disconnected, only thing to do is break
                            // out (i.e exit the thread)
                            otel_debug!(
                                name: "PeriodReaderThreadExiting",
                                reason = "MessageSenderDisconnected"
                            );
                            break;
                        }
                    }
                }
                otel_info!(
                    name: "PeriodReaderThreadStopped"
                );
            });

        // TODO: Should we fail-fast here and bubble up the error to user?
        #[allow(unused_variables)]
        if let Err(e) = result_thread_creation {
            otel_error!(
                name: "PeriodReaderThreadStartError",
                message = "Failed to start PeriodicReader thread. Metrics will not be exported.",
                error = format!("{:?}", e)
            );
        }
        reader
    }

    fn collect_and_export(&self, timeout: Duration) -> MetricResult<()> {
        self.inner.collect_and_export(timeout)
    }
}

impl fmt::Debug for PeriodicReader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PeriodicReader").finish()
    }
}

struct PeriodicReaderInner {
    exporter: Arc<dyn PushMetricExporter>,
    message_sender: mpsc::Sender<Message>,
    producer: Mutex<Option<Weak<dyn SdkProducer>>>,
}

impl PeriodicReaderInner {
    fn register_pipeline(&self, producer: Weak<dyn SdkProducer>) {
        let mut inner = self.producer.lock().expect("lock poisoned");
        *inner = Some(producer);
    }

    fn temporality(&self, _kind: InstrumentKind) -> Temporality {
        self.exporter.temporality()
    }

    fn collect(&self, rm: &mut ResourceMetrics) -> MetricResult<()> {
        let producer = self.producer.lock().expect("lock poisoned");
        if let Some(p) = producer.as_ref() {
            p.upgrade()
                .ok_or_else(|| MetricError::Other("pipeline is dropped".into()))?
                .produce(rm)?;
            Ok(())
        } else {
            Err(MetricError::Other("pipeline is not registered".into()))
        }
    }

    fn collect_and_export(&self, timeout: Duration) -> MetricResult<()> {
        // TODO: Reuse the internal vectors. Or refactor to avoid needing any
        // owned data structures to be passed to exporters.
        let mut rm = ResourceMetrics {
            resource: Resource::empty(),
            scope_metrics: Vec::new(),
        };

        // Measure time taken for collect, and subtract it from the timeout.
        let current_time = Instant::now();
        let collect_result = self.collect(&mut rm);
        let time_taken_for_collect = current_time.elapsed();
        let _timeout = if time_taken_for_collect > timeout {
            Duration::from_secs(0)
        } else {
            timeout - time_taken_for_collect
        };
        #[allow(clippy::question_mark)]
        if let Err(e) = collect_result {
            otel_warn!(
                name: "PeriodReaderCollectError",
                error = format!("{:?}", e)
            );
            return Err(e);
        }

        if rm.scope_metrics.is_empty() {
            otel_debug!(name: "NoMetricsCollected");
            return Ok(());
        }

        let metrics_count = rm.scope_metrics.iter().fold(0, |count, scope_metrics| {
            count + scope_metrics.metrics.len()
        });
        otel_debug!(name: "PeriodicReaderMetricsCollected", count = metrics_count, time_taken_in_millis = time_taken_for_collect.as_millis());

        // Relying on futures executor to execute async call.
        // TODO: Pass timeout to exporter
        let exporter_result = futures_executor::block_on(self.exporter.export(&mut rm));
        #[allow(clippy::question_mark)]
        if let Err(e) = exporter_result {
            otel_warn!(
                name: "PeriodReaderExportError",
                error = format!("{:?}", e)
            );
            return Err(e);
        }

        Ok(())
    }

    fn force_flush(&self) -> MetricResult<()> {
        // TODO: Better message for this scenario.
        // Flush and Shutdown called from 2 threads Flush check shutdown
        // flag before shutdown thread sets it. Both threads attempt to send
        // message to the same channel. Case1: Flush thread sends message first,
        // shutdown thread sends message next. Flush would succeed, as
        // background thread won't process shutdown message until flush
        // triggered export is done. Case2: Shutdown thread sends message first,
        // flush thread sends message next. Shutdown would succeed, as
        // background thread would process shutdown message first. The
        // background exits so it won't receive the flush message. ForceFlush
        // returns Failure, but we could indicate specifically that shutdown has
        // completed. TODO is to see if this message can be improved.

        let (response_tx, response_rx) = mpsc::channel();
        self.message_sender
            .send(Message::Flush(response_tx))
            .map_err(|e| MetricError::Other(e.to_string()))?;

        if let Ok(response) = response_rx.recv() {
            // TODO: call exporter's force_flush method.
            if response {
                Ok(())
            } else {
                Err(MetricError::Other("Failed to flush".into()))
            }
        } else {
            Err(MetricError::Other("Failed to flush".into()))
        }
    }

    fn shutdown(&self) -> ShutdownResult {
        // TODO: See if this is better to be created upfront.
        let (response_tx, response_rx) = mpsc::channel();
        self.message_sender
            .send(Message::Shutdown(response_tx))
            .map_err(|e| ShutdownError::InternalFailure(e.to_string()))?;

        // TODO: Make this timeout configurable.
        match response_rx.recv_timeout(Duration::from_secs(5)) {
            Ok(response) => {
                if response {
                    Ok(())
                } else {
                    Err(ShutdownError::InternalFailure("Failed to shutdown".into()))
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                Err(ShutdownError::Timeout(Duration::from_secs(5)))
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                Err(ShutdownError::InternalFailure("Failed to shutdown".into()))
            }
        }
    }
}

#[derive(Debug)]
enum Message {
    Flush(Sender<bool>),
    Shutdown(Sender<bool>),
}

impl MetricReader for PeriodicReader {
    fn register_pipeline(&self, pipeline: Weak<Pipeline>) {
        self.inner.register_pipeline(pipeline);
    }

    fn collect(&self, rm: &mut ResourceMetrics) -> MetricResult<()> {
        self.inner.collect(rm)
    }

    fn force_flush(&self) -> MetricResult<()> {
        self.inner.force_flush()
    }

    // TODO: Offer an async version of shutdown so users can await the shutdown
    // completion, and avoid blocking the thread. The default shutdown on drop
    // can still use blocking call. If user already explicitly called shutdown,
    // drop won't call shutdown again.
    fn shutdown(&self) -> ShutdownResult {
        self.inner.shutdown()
    }

    /// To construct a [MetricReader][metric-reader] when setting up an SDK,
    /// The output temporality (optional), a function of instrument kind.
    /// This function SHOULD be obtained from the exporter.
    ///
    /// If not configured, the Cumulative temporality SHOULD be used.
    ///  
    /// [metric-reader]: https://github.com/open-telemetry/opentelemetry-specification/blob/0a78571045ca1dca48621c9648ec3c832c3c541c/specification/metrics/sdk.md#metricreader
    fn temporality(&self, kind: InstrumentKind) -> Temporality {
        kind.temporality_preference(self.inner.temporality(kind))
    }
}

#[cfg(all(test, feature = "testing"))]
mod tests {
    use super::PeriodicReader;
    use crate::{
        error::ShutdownResult,
        metrics::{
            data::ResourceMetrics, exporter::PushMetricExporter, reader::MetricReader,
            InMemoryMetricExporter, MetricError, MetricResult, SdkMeterProvider, Temporality,
        },
        Resource,
    };
    use async_trait::async_trait;
    use opentelemetry::metrics::MeterProvider;
    use std::{
        sync::{
            atomic::{AtomicBool, AtomicUsize, Ordering},
            mpsc, Arc,
        },
        time::Duration,
    };

    // use below command to run all tests
    // cargo test metrics::periodic_reader::tests --features=testing,spec_unstable_metrics_views -- --nocapture

    #[derive(Debug, Clone)]
    struct MetricExporterThatFailsOnlyOnFirst {
        count: Arc<AtomicUsize>,
    }

    impl Default for MetricExporterThatFailsOnlyOnFirst {
        fn default() -> Self {
            MetricExporterThatFailsOnlyOnFirst {
                count: Arc::new(AtomicUsize::new(0)),
            }
        }
    }

    impl MetricExporterThatFailsOnlyOnFirst {
        fn get_count(&self) -> usize {
            self.count.load(Ordering::Relaxed)
        }
    }

    #[async_trait]
    impl PushMetricExporter for MetricExporterThatFailsOnlyOnFirst {
        async fn export(&self, _metrics: &mut ResourceMetrics) -> MetricResult<()> {
            if self.count.fetch_add(1, Ordering::Relaxed) == 0 {
                Err(MetricError::Other("export failed".into()))
            } else {
                Ok(())
            }
        }

        async fn force_flush(&self) -> MetricResult<()> {
            Ok(())
        }

        fn shutdown(&self) -> ShutdownResult {
            Ok(())
        }

        fn temporality(&self) -> Temporality {
            Temporality::Cumulative
        }
    }

    #[derive(Debug, Clone, Default)]
    struct MockMetricExporter {
        is_shutdown: Arc<AtomicBool>,
    }

    #[async_trait]
    impl PushMetricExporter for MockMetricExporter {
        async fn export(&self, _metrics: &mut ResourceMetrics) -> MetricResult<()> {
            Ok(())
        }

        async fn force_flush(&self) -> MetricResult<()> {
            Ok(())
        }

        fn shutdown(&self) -> ShutdownResult {
            self.is_shutdown.store(true, Ordering::Relaxed);
            Ok(())
        }

        fn temporality(&self) -> Temporality {
            Temporality::Cumulative
        }
    }

    #[test]
    fn collection_triggered_by_interval_multiple() {
        // Arrange
        let interval = std::time::Duration::from_millis(1);
        let exporter = InMemoryMetricExporter::default();
        let reader = PeriodicReader::builder(exporter.clone())
            .with_interval(interval)
            .build();
        let i = Arc::new(AtomicUsize::new(0));
        let i_clone = i.clone();

        // Act
        let meter_provider = SdkMeterProvider::builder().with_reader(reader).build();
        let meter = meter_provider.meter("test");
        let _counter = meter
            .u64_observable_counter("testcounter")
            .with_callback(move |_| {
                i_clone.fetch_add(1, Ordering::Relaxed);
            })
            .build();

        // Sleep for a duration 5X (plus liberal buffer to account for potential
        // CI slowness) the interval to ensure multiple collection.
        // Not a fan of such tests, but this seems to be the only way to test
        // if periodic reader is doing its job.
        // TODO: Decide if this should be ignored in CI
        std::thread::sleep(interval * 5 * 20);

        // Assert
        assert!(i.load(Ordering::Relaxed) >= 5);
    }

    #[test]
    fn shutdown_repeat() {
        // Arrange
        let interval = std::time::Duration::from_millis(1);
        let exporter = InMemoryMetricExporter::default();
        let reader = PeriodicReader::builder(exporter.clone())
            .with_interval(interval)
            .build();

        let meter_provider = SdkMeterProvider::builder().with_reader(reader).build();
        let result = meter_provider.shutdown();
        assert!(result.is_ok());

        // calling shutdown again should return Err
        let result = meter_provider.shutdown();
        assert!(result.is_err());

        // calling shutdown again should return Err
        let result = meter_provider.shutdown();
        assert!(result.is_err());
    }

    #[test]
    fn flush_after_shutdown() {
        // Arrange
        let interval = std::time::Duration::from_millis(1);
        let exporter = InMemoryMetricExporter::default();
        let reader = PeriodicReader::builder(exporter.clone())
            .with_interval(interval)
            .build();

        let meter_provider = SdkMeterProvider::builder().with_reader(reader).build();
        let result = meter_provider.force_flush();
        assert!(result.is_ok());

        let result = meter_provider.shutdown();
        assert!(result.is_ok());

        // calling force_flush after shutdown should return Err
        let result = meter_provider.force_flush();
        assert!(result.is_err());
    }

    #[test]
    fn flush_repeat() {
        // Arrange
        let interval = std::time::Duration::from_millis(1);
        let exporter = InMemoryMetricExporter::default();
        let reader = PeriodicReader::builder(exporter.clone())
            .with_interval(interval)
            .build();

        let meter_provider = SdkMeterProvider::builder().with_reader(reader).build();
        let result = meter_provider.force_flush();
        assert!(result.is_ok());

        // calling force_flush again should return Ok
        let result = meter_provider.force_flush();
        assert!(result.is_ok());
    }

    #[test]
    fn periodic_reader_without_pipeline() {
        // Arrange
        let interval = std::time::Duration::from_millis(1);
        let exporter = InMemoryMetricExporter::default();
        let reader = PeriodicReader::builder(exporter.clone())
            .with_interval(interval)
            .build();

        let rm = &mut ResourceMetrics {
            resource: Resource::empty(),
            scope_metrics: Vec::new(),
        };
        // Pipeline is not registered, so collect should return an error
        let result = reader.collect(rm);
        assert!(result.is_err());

        // Pipeline is not registered, so flush should return an error
        let result = reader.force_flush();
        assert!(result.is_err());

        // Adding reader to meter provider should register the pipeline
        // TODO: This part might benefit from a different design.
        let meter_provider = SdkMeterProvider::builder()
            .with_reader(reader.clone())
            .build();

        // Now collect and flush should succeed
        let result = reader.collect(rm);
        assert!(result.is_ok());

        let result = meter_provider.force_flush();
        assert!(result.is_ok());
    }

    #[test]
    fn exporter_failures_are_handled() {
        // create a mock exporter that fails 1st time and succeeds 2nd time
        // Validate using this exporter that periodic reader can handle exporter failure
        // and continue to export metrics.
        // Arrange
        let interval = std::time::Duration::from_millis(10);
        let exporter = MetricExporterThatFailsOnlyOnFirst::default();
        let reader = PeriodicReader::builder(exporter.clone())
            .with_interval(interval)
            .build();

        let meter_provider = SdkMeterProvider::builder().with_reader(reader).build();
        let meter = meter_provider.meter("test");
        let counter = meter.u64_counter("sync_counter").build();
        counter.add(1, &[]);
        let _obs_counter = meter
            .u64_observable_counter("testcounter")
            .with_callback(move |observer| {
                observer.observe(1, &[]);
            })
            .build();

        // Sleep for a duration much longer than the interval to trigger
        // multiple exports, including failures.
        // Not a fan of such tests, but this seems to be the
        // only way to test if periodic reader is doing its job. TODO: Decide if
        // this should be ignored in CI
        std::thread::sleep(Duration::from_millis(500));

        // Assert that atleast 2 exports are attempted given the 1st one fails.
        assert!(exporter.get_count() >= 2);
    }

    #[test]
    fn shutdown_passed_to_exporter() {
        // Arrange
        let exporter = MockMetricExporter::default();
        let reader = PeriodicReader::builder(exporter.clone()).build();

        let meter_provider = SdkMeterProvider::builder().with_reader(reader).build();
        let meter = meter_provider.meter("test");
        let counter = meter.u64_counter("sync_counter").build();
        counter.add(1, &[]);

        // shutdown the provider, which should call shutdown on periodic reader
        // which in turn should call shutdown on exporter.
        let result = meter_provider.shutdown();
        assert!(result.is_ok());
        assert!(exporter.is_shutdown.load(Ordering::Relaxed));
    }

    #[test]
    fn collection() {
        collection_triggered_by_interval_helper();
        collection_triggered_by_flush_helper();
        collection_triggered_by_shutdown_helper();
        collection_triggered_by_drop_helper();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn collection_from_tokio_multi_with_one_worker() {
        collection_triggered_by_interval_helper();
        collection_triggered_by_flush_helper();
        collection_triggered_by_shutdown_helper();
        collection_triggered_by_drop_helper();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn collection_from_tokio_with_two_worker() {
        collection_triggered_by_interval_helper();
        collection_triggered_by_flush_helper();
        collection_triggered_by_shutdown_helper();
        collection_triggered_by_drop_helper();
    }

    #[tokio::test(flavor = "current_thread")]
    async fn collection_from_tokio_current() {
        collection_triggered_by_interval_helper();
        collection_triggered_by_flush_helper();
        collection_triggered_by_shutdown_helper();
        collection_triggered_by_drop_helper();
    }

    fn collection_triggered_by_interval_helper() {
        collection_helper(|_| {
            // Sleep for a duration longer than the interval to ensure at least one collection
            // Not a fan of such tests, but this seems to be the only way to test
            // if periodic reader is doing its job.
            // TODO: Decide if this should be ignored in CI
            std::thread::sleep(Duration::from_millis(500));
        });
    }

    fn collection_triggered_by_flush_helper() {
        collection_helper(|meter_provider| {
            meter_provider.force_flush().expect("flush should succeed");
        });
    }

    fn collection_triggered_by_shutdown_helper() {
        collection_helper(|meter_provider| {
            meter_provider.shutdown().expect("shutdown should succeed");
        });
    }

    fn collection_triggered_by_drop_helper() {
        collection_helper(|meter_provider| {
            drop(meter_provider);
        });
    }

    fn collection_helper(trigger: fn(SdkMeterProvider)) {
        // Arrange
        let interval = std::time::Duration::from_millis(10);
        let exporter = InMemoryMetricExporter::default();
        let reader = PeriodicReader::builder(exporter.clone())
            .with_interval(interval)
            .build();
        let (sender, receiver) = mpsc::channel();

        let meter_provider = SdkMeterProvider::builder().with_reader(reader).build();
        let meter = meter_provider.meter("test");
        let _counter = meter
            .u64_observable_counter("testcounter")
            .with_callback(move |observer| {
                observer.observe(1, &[]);
                sender.send(()).expect("channel should still be open");
            })
            .build();

        // Act
        trigger(meter_provider);

        // Assert
        receiver
            .recv_timeout(Duration::ZERO)
            .expect("message should be available in channel, indicating a collection occurred, which should trigger observable callback");

        let exported_metrics = exporter
            .get_finished_metrics()
            .expect("this should not fail");
        assert!(
            !exported_metrics.is_empty(),
            "Metrics should be available in exporter."
        );
    }

    async fn some_async_function() -> u64 {
        // No dependency on any particular async runtime.
        std::thread::sleep(std::time::Duration::from_millis(1));
        1
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn async_inside_observable_callback_from_tokio_multi_with_one_worker() {
        async_inside_observable_callback_helper();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn async_inside_observable_callback_from_tokio_multi_with_two_worker() {
        async_inside_observable_callback_helper();
    }

    #[tokio::test(flavor = "current_thread")]
    async fn async_inside_observable_callback_from_tokio_current_thread() {
        async_inside_observable_callback_helper();
    }

    #[test]
    fn async_inside_observable_callback_from_regular_main() {
        async_inside_observable_callback_helper();
    }

    fn async_inside_observable_callback_helper() {
        let interval = std::time::Duration::from_millis(10);
        let exporter = InMemoryMetricExporter::default();
        let reader = PeriodicReader::builder(exporter.clone())
            .with_interval(interval)
            .build();

        let meter_provider = SdkMeterProvider::builder().with_reader(reader).build();
        let meter = meter_provider.meter("test");
        let _gauge = meter
            .u64_observable_gauge("my_observable_gauge")
            .with_callback(|observer| {
                // using futures_executor::block_on intentionally and avoiding
                // any particular async runtime.
                let value = futures_executor::block_on(some_async_function());
                observer.observe(value, &[]);
            })
            .build();

        meter_provider.force_flush().expect("flush should succeed");
        let exported_metrics = exporter
            .get_finished_metrics()
            .expect("this should not fail");
        assert!(
            !exported_metrics.is_empty(),
            "Metrics should be available in exporter."
        );
    }

    async fn some_tokio_async_function() -> u64 {
        // Tokio specific async function
        tokio::time::sleep(Duration::from_millis(1)).await;
        1
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]

    async fn tokio_async_inside_observable_callback_from_tokio_multi_with_one_worker() {
        tokio_async_inside_observable_callback_helper(true);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn tokio_async_inside_observable_callback_from_tokio_multi_with_two_worker() {
        tokio_async_inside_observable_callback_helper(true);
    }

    #[tokio::test(flavor = "current_thread")]
    #[ignore] //TODO: Investigate if this can be fixed.
    async fn tokio_async_inside_observable_callback_from_tokio_current_thread() {
        tokio_async_inside_observable_callback_helper(true);
    }

    #[test]
    fn tokio_async_inside_observable_callback_from_regular_main() {
        tokio_async_inside_observable_callback_helper(false);
    }

    fn tokio_async_inside_observable_callback_helper(use_current_tokio_runtime: bool) {
        let interval = std::time::Duration::from_millis(10);
        let exporter = InMemoryMetricExporter::default();
        let reader = PeriodicReader::builder(exporter.clone())
            .with_interval(interval)
            .build();

        let meter_provider = SdkMeterProvider::builder().with_reader(reader).build();
        let meter = meter_provider.meter("test");

        if use_current_tokio_runtime {
            let rt = tokio::runtime::Handle::current().clone();
            let _gauge = meter
                .u64_observable_gauge("my_observable_gauge")
                .with_callback(move |observer| {
                    // call tokio specific async function from here
                    let value = rt.block_on(some_tokio_async_function());
                    observer.observe(value, &[]);
                })
                .build();
            // rt here is a reference to the current tokio runtime.
            // Dropping it occurs when the tokio::main itself ends.
        } else {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let _gauge = meter
                .u64_observable_gauge("my_observable_gauge")
                .with_callback(move |observer| {
                    // call tokio specific async function from here
                    let value = rt.block_on(some_tokio_async_function());
                    observer.observe(value, &[]);
                })
                .build();
            // rt is not dropped here as it is moved to the closure,
            // and is dropped only when MeterProvider itself is dropped.
            // This works when called from normal main.
        };

        meter_provider.force_flush().expect("flush should succeed");
        let exported_metrics = exporter
            .get_finished_metrics()
            .expect("this should not fail");
        assert!(
            !exported_metrics.is_empty(),
            "Metrics should be available in exporter."
        );
    }
}
