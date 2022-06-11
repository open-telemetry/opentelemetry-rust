use std::{
    collections::HashMap,
    fmt,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};

use futures_channel::{mpsc, oneshot};
use futures_util::{stream, StreamExt};
use opentelemetry_api::{
    global,
    metrics::{noop, Meter, MeterProvider, MetricsError, Result},
    Context, InstrumentationLibrary,
};

use crate::{
    export::metrics::{
        Checkpointer, CheckpointerFactory, InstrumentationLibraryReader, LockedCheckpointer,
        MetricsExporter, Reader,
    },
    metrics::{
        accumulator,
        registry::{self, UniqueInstrumentMeterCore},
        sdk_api::{
            wrap_meter_core, AsyncInstrumentCore, Descriptor, MeterCore, SyncInstrumentCore,
        },
        Accumulator,
    },
    runtime::Runtime,
    Resource,
};

/// DefaultPeriod is used for:
///
/// - the minimum time between calls to `collect`.
/// - the timeout for `export`.
/// - the timeout for `collect`.
const DEFAULT_PERIOD: Duration = Duration::from_secs(10);

/// Returns a new builder using the provided checkpointer factory.
///
/// Use builder options (including optional exporter) to configure a metric
/// export pipeline.
pub fn basic<T>(factory: T) -> BasicControllerBuilder
where
    T: CheckpointerFactory + Send + Sync + 'static,
{
    BasicControllerBuilder {
        checkpointer_factory: Box::new(factory),
        resource: None,
        exporter: None,
        collect_period: None,
        collect_timeout: None,
        push_timeout: None,
    }
}

/// Organizes and synchronizes collection of metric data in both "pull" and
/// "push" configurations.
///
/// This supports two distinct modes:
///
/// - Push and Pull: `start` must be called to begin calling the exporter;
/// `collect` is called periodically after starting the controller.
/// - Pull-Only: `start` is optional in this case, to call `collect`
/// periodically. If `start` is not called, `collect` can be called manually to
/// initiate collection.
///
/// The controller supports mixing push and pull access to metric data using the
/// `InstrumentationLibraryReader` interface.
#[derive(Clone)]
pub struct BasicController(Arc<ControllerInner>);

struct ControllerInner {
    meters: Mutex<HashMap<InstrumentationLibrary, Arc<UniqueInstrumentMeterCore>>>,
    checkpointer_factory: Box<dyn CheckpointerFactory + Send + Sync>,
    resource: Resource,
    exporter: Mutex<Option<Box<dyn MetricsExporter + Send + Sync>>>,
    worker_channel: Mutex<Option<mpsc::Sender<WorkerMessage>>>,
    collect_period: Duration,
    collect_timeout: Duration,
    push_timeout: Duration,
    collected_time: Mutex<Option<SystemTime>>,
}

enum WorkerMessage {
    Tick,
    Shutdown((Context, oneshot::Sender<()>)),
}

impl BasicController {
    /// This begins a ticker that periodically collects and exports metrics with the
    /// configured interval.
    ///
    /// This is required for calling a configured [`MetricsExporter`] (see
    /// [`BasicControllerBuilder::with_exporter`]) and is otherwise optional when
    /// only pulling metric data.
    ///
    /// The passed in context is passed to `collect` and subsequently to
    /// asynchronous instrument callbacks. Returns an error when the controller was
    /// already started.
    ///
    /// Note that it is not necessary to start a controller when only pulling data;
    /// use the `collect` and `try_for_each` methods directly in this case.
    pub fn start<T: Runtime>(&self, cx: &Context, rt: T) -> Result<()> {
        let (message_sender, message_receiver) = mpsc::channel(8);
        let ticker = rt
            .interval(self.0.collect_period)
            .map(|_| WorkerMessage::Tick);

        let exporter = self
            .0
            .exporter
            .lock()
            .map(|mut ex| ex.take())
            .unwrap_or_default();
        let resource = self.resource().clone();
        let reader = self.clone();
        let cx = cx.clone();
        // Spawn worker process via user-defined spawn function.
        rt.spawn(Box::pin(async move {
            let mut messages = Box::pin(stream::select(message_receiver, ticker));
            while let Some(message) = messages.next().await {
                match message {
                    WorkerMessage::Tick => {
                        match reader.checkpoint(&cx) {
                            Ok(_) => {
                                if let Some(exporter) = &exporter {
                                    // TODO timeout
                                    if let Err(err) = exporter.export(&cx, &resource, &reader) {
                                        global::handle_error(err);
                                    }
                                }
                            }
                            Err(err) => global::handle_error(err),
                        };
                    }
                    WorkerMessage::Shutdown((cx, channel)) => {
                        let _ = reader.checkpoint(&cx);
                        if let Some(exporter) = &exporter {
                            let _ = exporter.export(&cx, &resource, &reader);
                        }
                        let _ = channel.send(());
                        break;
                    }
                }
            }
        }));

        *self.0.worker_channel.lock()? = Some(message_sender);

        Ok(())
    }

    /// This waits for the background worker to return and then collects
    /// and exports metrics one last time before returning.
    ///
    /// The passed context is passed to the final `collect` and subsequently to the
    /// final asynchronous instruments.
    ///
    /// Note that `stop` will not cancel an ongoing collection or export.
    pub fn stop(&self, cx: &Context) -> Result<()> {
        self.0
            .worker_channel
            .lock()
            .map_err(Into::into)
            .and_then(|mut worker| {
                if let Some(mut worker) = worker.take() {
                    let (res_sender, res_receiver) = oneshot::channel();
                    if worker
                        .try_send(WorkerMessage::Shutdown((cx.clone(), res_sender)))
                        .is_ok()
                    {
                        futures_executor::block_on(res_receiver)
                            .map_err(|err| MetricsError::Other(err.to_string()))
                    } else {
                        Ok(())
                    }
                } else {
                    Ok(())
                }
            })
    }

    /// true if the controller was started via `start`, indicating that the
    /// current `Reader` is being kept up-to-date.
    pub fn is_running(&self) -> bool {
        self.0
            .worker_channel
            .lock()
            .map(|wc| wc.is_some())
            .unwrap_or(false)
    }

    /// `true` if the collector should collect now, based on the current time, the
    /// last collection time, and the configured period.
    fn should_collect(&self) -> bool {
        self.0
            .collected_time
            .lock()
            .map(|mut collected_time| {
                if self.0.collect_period.is_zero() {
                    return true;
                }
                let now = SystemTime::now();
                if let Some(collected_time) = *collected_time {
                    if now.duration_since(collected_time).unwrap_or_default()
                        < self.0.collect_period
                    {
                        return false;
                    }
                }

                *collected_time = Some(now);
                true
            })
            .unwrap_or(false)
    }

    /// Requests a collection.
    ///
    /// The collection will be skipped if the last collection is aged less than the
    /// configured collection period.
    pub fn collect(&self, cx: &Context) -> Result<()> {
        if self.is_running() {
            // When the ticker is `Some`, there's a component
            // computing checkpoints with the collection period.
            return Err(MetricsError::Other("controller already started".into()));
        }

        if !self.should_collect() {
            return Ok(());
        }

        self.checkpoint(cx)
    }

    /// Get a reference to the current resource.
    pub fn resource(&self) -> &Resource {
        &self.0.resource
    }

    /// Returns a snapshot of current accumulators registered to this controller.
    ///
    /// This briefly locks the controller.
    fn with_accumulator_list<F, T>(&self, mut f: F) -> Result<T>
    where
        F: FnMut(&[&AccumulatorCheckpointer]) -> Result<T>,
    {
        self.0.meters.lock().map_err(Into::into).and_then(|meters| {
            let accs = meters
                .values()
                .filter_map(|unique| {
                    unique
                        .meter_core()
                        .downcast_ref::<AccumulatorCheckpointer>()
                })
                .collect::<Vec<_>>();
            f(&accs)
        })
    }

    /// Calls the accumulator and checkpointer interfaces to
    /// compute the reader.
    fn checkpoint(&self, cx: &Context) -> Result<()> {
        self.with_accumulator_list(|accs| {
            for acc in accs {
                self.checkpoint_single_accumulator(cx, acc)?;
            }

            Ok(())
        })
    }

    fn checkpoint_single_accumulator(
        &self,
        cx: &Context,
        ac: &AccumulatorCheckpointer,
    ) -> Result<()> {
        ac.checkpointer
            .checkpoint(&mut |ckpt: &mut dyn LockedCheckpointer| {
                ckpt.start_collection();
                if !self.0.collect_timeout.is_zero() {
                    // TODO timeouts
                }

                ac.accumulator.collect(cx, ckpt.processor());

                ckpt.finish_collection()
            })
    }
}

impl MeterProvider for BasicController {
    fn versioned_meter(
        &self,
        name: &'static str,
        version: Option<&'static str>,
        schema_url: Option<&'static str>,
    ) -> Meter {
        self.0
            .meters
            .lock()
            .map(|mut meters| {
                let library = InstrumentationLibrary::new(name, version, schema_url);
                let meter_core = meters.entry(library.clone()).or_insert_with(|| {
                    let checkpointer = self.0.checkpointer_factory.checkpointer();
                    Arc::new(registry::unique_instrument_meter_core(
                        AccumulatorCheckpointer {
                            accumulator: accumulator(checkpointer.clone().as_dyn_processor()),
                            checkpointer,
                            library: library.clone(),
                        },
                    ))
                });
                wrap_meter_core(meter_core.clone(), library)
            })
            .unwrap_or_else(|_| {
                noop::NoopMeterProvider::new().versioned_meter(name, version, schema_url)
            })
    }
}

struct AccumulatorCheckpointer {
    accumulator: Accumulator,
    checkpointer: Arc<dyn Checkpointer + Send + Sync>,
    library: InstrumentationLibrary,
}

impl MeterCore for AccumulatorCheckpointer {
    fn new_sync_instrument(
        &self,
        descriptor: Descriptor,
    ) -> Result<Arc<dyn SyncInstrumentCore + Send + Sync>> {
        self.accumulator.new_sync_instrument(descriptor)
    }

    fn new_async_instrument(
        &self,
        descriptor: Descriptor,
    ) -> Result<Arc<dyn AsyncInstrumentCore + Send + Sync>> {
        self.accumulator.new_async_instrument(descriptor)
    }

    fn register_callback(&self, f: Box<dyn Fn(&Context) + Send + Sync>) -> Result<()> {
        self.accumulator.register_callback(f)
    }
}

impl InstrumentationLibraryReader for BasicController {
    fn try_for_each(
        &self,
        f: &mut dyn FnMut(&InstrumentationLibrary, &mut dyn Reader) -> Result<()>,
    ) -> Result<()> {
        let mut res = Ok(());
        self.with_accumulator_list(|acs| {
            for ac_pair in acs {
                if res.is_err() {
                    continue;
                }

                res = ac_pair
                    .checkpointer
                    .checkpoint(&mut |locked| f(&ac_pair.library, locked.reader()))
            }

            Ok(())
        })?;

        res
    }
}

impl fmt::Debug for BasicController {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BasicController")
            .field("resource", &self.0.resource)
            .field("collect_period", &self.0.collect_period)
            .field("collect_timeout", &self.0.collect_timeout)
            .field("push_timeout", &self.0.push_timeout)
            .field("collected_time", &self.0.collect_timeout)
            .finish()
    }
}

/// Options for configuring a [`BasicController`]
pub struct BasicControllerBuilder {
    checkpointer_factory: Box<dyn CheckpointerFactory + Send + Sync>,
    resource: Option<Resource>,
    exporter: Option<Box<dyn MetricsExporter + Send + Sync>>,
    collect_period: Option<Duration>,
    collect_timeout: Option<Duration>,
    push_timeout: Option<Duration>,
}

impl BasicControllerBuilder {
    /// Sets the [`Resource`] used for this controller.
    pub fn with_resource(mut self, resource: Resource) -> Self {
        self.resource = Some(resource);
        self
    }

    /// Sets the exporter used for exporting metric data.
    ///
    /// Note: Exporters such as Prometheus that pull data do not implement
    /// [`MetricsExporter`]. They will directly call `collect` and `try_for_each`.
    pub fn with_exporter(mut self, exporter: impl MetricsExporter + Send + Sync + 'static) -> Self {
        self.exporter = Some(Box::new(exporter));
        self
    }

    /// Sets the interval between calls to `collect` a checkpoint.
    ///
    /// When pulling metrics and not exporting, this is the minimum time between
    /// calls to `collect.In a pull-only configuration, collection is performed on
    /// demand; set this to `0` to always recompute the export record set.
    ///
    /// When exporting metrics, this must be > 0.
    ///
    /// Default value is 10s.
    pub fn with_collect_period(mut self, collect_period: Duration) -> Self {
        self.collect_period = Some(collect_period);
        self
    }

    /// Sets the timeout of the `collect` and subsequent observer instrument
    /// callbacks.
    ///
    /// Default value is 10s. If zero or none, no collect timeout is applied.
    pub fn with_collect_timeout(mut self, collect_timeout: Duration) -> Self {
        self.collect_timeout = Some(collect_timeout);
        self
    }

    /// Sets push controller timeout when a exporter is configured.
    ///
    /// Default value is 10s. If zero, no export timeout is applied.
    pub fn with_push_timeout(mut self, push_timeout: Duration) -> Self {
        self.push_timeout = Some(push_timeout);
        self
    }

    /// Creates a new basic controller.
    pub fn build(self) -> BasicController {
        BasicController(Arc::new(ControllerInner {
            meters: Default::default(),
            checkpointer_factory: self.checkpointer_factory,
            resource: self.resource.unwrap_or_default(),
            exporter: Mutex::new(self.exporter),
            worker_channel: Mutex::new(None),
            collect_period: self.collect_period.unwrap_or(DEFAULT_PERIOD),
            collect_timeout: self.collect_timeout.unwrap_or(DEFAULT_PERIOD),
            push_timeout: self.push_timeout.unwrap_or(DEFAULT_PERIOD),
            collected_time: Default::default(),
        }))
    }
}

impl fmt::Debug for BasicControllerBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BasicControllerBuilder")
            .field("resource", &self.resource)
            .field("collect_period", &self.collect_period)
            .field("collect_timeout", &self.collect_timeout)
            .field("push_timeout", &self.push_timeout)
            .finish()
    }
}
