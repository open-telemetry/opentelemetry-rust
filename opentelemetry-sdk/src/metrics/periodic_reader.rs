use std::{
    env, fmt,
    sync::{Arc, Mutex, Weak},
    time::Duration,
};

use futures_channel::{mpsc, oneshot};
use futures_util::{
    future::{self, Either},
    pin_mut,
    stream::{self, FusedStream},
    Stream, StreamExt,
};
use opentelemetry::{
    global,
    metrics::{MetricsError, Result},
};

use crate::runtime::Runtime;
use crate::{
    metrics::{
        exporter::PushMetricsExporter,
        reader::{MetricProducer, SdkProducer},
    },
    Resource,
};

use super::{
    aggregation::Aggregation,
    data::{ResourceMetrics, Temporality},
    instrument::InstrumentKind,
    reader::{AggregationSelector, MetricReader, TemporalitySelector},
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
pub struct PeriodicReaderBuilder<E, RT> {
    interval: Duration,
    timeout: Duration,
    exporter: E,
    producers: Vec<Box<dyn MetricProducer>>,
    runtime: RT,
}

impl<E, RT> PeriodicReaderBuilder<E, RT>
where
    E: PushMetricsExporter,
    RT: Runtime,
{
    fn new(exporter: E, runtime: RT) -> Self {
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
            producers: vec![],
            exporter,
            runtime,
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

    /// Registers a an external [MetricProducer] with this reader.
    ///
    /// The producer is used as a source of aggregated metric data which is
    /// incorporated into metrics collected from the SDK.
    pub fn with_producer(mut self, producer: impl MetricProducer + 'static) -> Self {
        self.producers.push(Box::new(producer));
        self
    }

    /// Create a [PeriodicReader] with the given config.
    pub fn build(self) -> PeriodicReader {
        let (message_sender, message_receiver) = mpsc::channel(256);
        let ticker = self
            .runtime
            .interval(self.interval)
            .map(|_| Message::Export);

        let messages = Box::pin(stream::select(message_receiver, ticker));
        let reader = PeriodicReader {
            exporter: Arc::new(self.exporter),
            inner: Arc::new(Mutex::new(PeriodicReaderInner {
                message_sender,
                sdk_producer: None,
                is_shutdown: false,
                external_producers: self.producers,
            })),
        };

        let runtime = self.runtime.clone();
        self.runtime.spawn(Box::pin(
            PeriodicReaderWorker {
                reader: reader.clone(),
                timeout: self.timeout,
                runtime,
                rm: ResourceMetrics {
                    resource: Resource::empty(),
                    scope_metrics: Vec::new(),
                },
            }
            .run(messages),
        ));

        reader
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
    inner: Arc<Mutex<PeriodicReaderInner>>,
}

impl PeriodicReader {
    /// Configuration options for a periodic reader
    pub fn builder<E, RT>(exporter: E, runtime: RT) -> PeriodicReaderBuilder<E, RT>
    where
        E: PushMetricsExporter,
        RT: Runtime,
    {
        PeriodicReaderBuilder::new(exporter, runtime)
    }
}

impl fmt::Debug for PeriodicReader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PeriodicReader").finish()
    }
}

struct PeriodicReaderInner {
    message_sender: mpsc::Sender<Message>,
    sdk_producer: Option<Weak<dyn SdkProducer>>,
    is_shutdown: bool,
    external_producers: Vec<Box<dyn MetricProducer>>,
}

#[derive(Debug)]
enum Message {
    Export,
    Flush(oneshot::Sender<Result<()>>),
    Shutdown(oneshot::Sender<Result<()>>),
}

struct PeriodicReaderWorker<RT: Runtime> {
    reader: PeriodicReader,
    timeout: Duration,
    runtime: RT,
    rm: ResourceMetrics,
}

impl<RT: Runtime> PeriodicReaderWorker<RT> {
    async fn collect_and_export(&mut self) -> Result<()> {
        self.reader.collect(&mut self.rm)?;
        let export = self.reader.exporter.export(&mut self.rm);
        let timeout = self.runtime.delay(self.timeout);
        pin_mut!(export);
        pin_mut!(timeout);

        match future::select(export, timeout).await {
            Either::Left(_) => Ok(()),
            Either::Right(_) => Err(MetricsError::Other("export timed out".into())),
        }
    }

    async fn process_message(&mut self, message: Message) -> bool {
        match message {
            Message::Export => {
                if let Err(err) = self.collect_and_export().await {
                    global::handle_error(err)
                }
            }
            Message::Flush(ch) => {
                let res = self.collect_and_export().await;
                if ch.send(res).is_err() {
                    global::handle_error(MetricsError::Other("flush channel closed".into()))
                }
            }
            Message::Shutdown(ch) => {
                let res = self.collect_and_export().await;
                let _ = self.reader.exporter.shutdown();
                if ch.send(res).is_err() {
                    global::handle_error(MetricsError::Other("shutdown channel closed".into()))
                }
                return false;
            }
        }

        true
    }

    async fn run(mut self, mut messages: impl Stream<Item = Message> + Unpin + FusedStream) {
        while let Some(message) = messages.next().await {
            if !self.process_message(message).await {
                break;
            }
        }
    }
}

impl AggregationSelector for PeriodicReader {
    fn aggregation(&self, kind: InstrumentKind) -> Aggregation {
        self.exporter.aggregation(kind)
    }
}

impl TemporalitySelector for PeriodicReader {
    fn temporality(&self, kind: InstrumentKind) -> Temporality {
        self.exporter.temporality(kind)
    }
}

impl MetricReader for PeriodicReader {
    fn register_pipeline(&self, pipeline: Weak<Pipeline>) {
        let mut inner = match self.inner.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };

        // Only register once. If producer is already set, do nothing.
        if inner.sdk_producer.is_none() {
            inner.sdk_producer = Some(pipeline);
        } else {
            global::handle_error(MetricsError::Other(
                "duplicate meter registration, did not register manual reader".into(),
            ))
        }
    }

    fn collect(&self, rm: &mut ResourceMetrics) -> Result<()> {
        let inner = self.inner.lock()?;
        if inner.is_shutdown {
            return Err(MetricsError::Other("reader is shut down".into()));
        }

        match &inner.sdk_producer.as_ref().and_then(|w| w.upgrade()) {
            Some(producer) => producer.produce(rm)?,
            None => {
                return Err(MetricsError::Other(
                    "reader is shut down or not registered".into(),
                ))
            }
        };

        let mut errs = vec![];
        for producer in &inner.external_producers {
            match producer.produce() {
                Ok(metrics) => rm.scope_metrics.push(metrics),
                Err(err) => errs.push(err),
            }
        }

        if errs.is_empty() {
            Ok(())
        } else {
            Err(MetricsError::Other(format!("{:?}", errs)))
        }
    }

    fn force_flush(&self) -> Result<()> {
        let mut inner = self.inner.lock()?;
        if inner.is_shutdown {
            return Err(MetricsError::Other("reader is shut down".into()));
        }
        let (sender, receiver) = oneshot::channel();
        inner
            .message_sender
            .try_send(Message::Flush(sender))
            .map_err(|e| MetricsError::Other(e.to_string()))?;

        drop(inner); // don't hold lock when blocking on future

        futures_executor::block_on(receiver)
            .map_err(|err| MetricsError::Other(err.to_string()))
            .and_then(|res| res)
    }

    fn shutdown(&self) -> Result<()> {
        let mut inner = self.inner.lock()?;
        if inner.is_shutdown {
            return Err(MetricsError::Other("reader is already shut down".into()));
        }

        let (sender, receiver) = oneshot::channel();
        inner
            .message_sender
            .try_send(Message::Shutdown(sender))
            .map_err(|e| MetricsError::Other(e.to_string()))?;
        drop(inner); // don't hold lock when blocking on future

        let shutdown_result = futures_executor::block_on(receiver)
            .map_err(|err| MetricsError::Other(err.to_string()))?;

        // Acquire the lock again to set the shutdown flag
        let mut inner = self.inner.lock()?;
        inner.is_shutdown = true;
        drop(inner);

        shutdown_result
    }
}
