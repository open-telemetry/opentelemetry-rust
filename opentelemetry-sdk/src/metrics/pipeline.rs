use core::fmt;
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use opentelemetry_api::{
    global,
    metrics::{CallbackRegistration, MetricsError, Result, Unit},
    Context,
};

use crate::{
    instrumentation::Scope,
    metrics::{aggregation, data, internal, view::View},
    Resource,
};

use super::{
    data::{Metric, ResourceMetrics, ScopeMetrics, Temporality},
    instrument::{Instrument, InstrumentKind, Stream, StreamId},
    internal::Number,
    reader::{MetricReader, SdkProducer},
};

/// Connects all of the instruments created by a meter provider to a [MetricReader].
///
/// This is the object that will be registered when a meter provider is
/// created.
///
/// As instruments are created the instrument should be checked if it exists in
/// the views of a the Reader, and if so each aggregator should be added to the
/// pipeline.
#[doc(hidden)]
pub struct Pipeline {
    resource: Resource,
    reader: Box<dyn MetricReader>,
    views: Vec<Arc<dyn View>>,
    inner: Box<Mutex<PipelineInner>>,
}

impl fmt::Debug for Pipeline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Pipeline")
    }
}

/// Single or multi-instrument callbacks
type GenericCallback = Arc<dyn Fn() + Send + Sync>;

#[derive(Default)]
struct PipelineInner {
    aggregations: HashMap<Scope, Vec<InstrumentSync>>,
    callbacks: Vec<GenericCallback>,
    multi_callbacks: Vec<Option<GenericCallback>>,
}

impl fmt::Debug for PipelineInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PipelineInner")
            .field("aggregations", &self.aggregations)
            .field("callbacks", &self.callbacks.len())
            .finish()
    }
}

impl Pipeline {
    /// Adds the [InstrumentSync] to pipeline with scope.
    ///
    /// This method is not idempotent. Duplicate calls will result in duplicate
    /// additions, it is the callers responsibility to ensure this is called with
    /// unique values.
    fn add_sync(&self, scope: Scope, i_sync: InstrumentSync) {
        let _ = self.inner.lock().map(|mut inner| {
            inner.aggregations.entry(scope).or_default().push(i_sync);
        });
    }

    /// Registers a single instrument callback to be run when `produce` is called.
    fn add_callback(&self, callback: GenericCallback) {
        let _ = self
            .inner
            .lock()
            .map(|mut inner| inner.callbacks.push(callback));
    }

    /// Registers a multi-instrument callback to be run when `produce` is called.
    fn add_multi_callback(
        &self,
        callback: GenericCallback,
    ) -> Result<impl FnOnce(&Pipeline) -> Result<()>> {
        let mut inner = self.inner.lock()?;
        inner.multi_callbacks.push(Some(callback));
        let idx = inner.multi_callbacks.len() - 1;

        Ok(move |this: &Pipeline| {
            let mut inner = this.inner.lock()?;
            // can't compare trait objects so use index + toumbstones to drop
            inner.multi_callbacks[idx] = None;
            Ok(())
        })
    }

    /// Send accumulated telemetry
    fn force_flush(&self, cx: &Context) -> Result<()> {
        self.reader.force_flush(cx)
    }

    /// Shut down pipeline
    fn shutdown(&self) -> Result<()> {
        self.reader.shutdown()
    }
}

impl SdkProducer for Pipeline {
    /// Returns aggregated metrics from a single collection.
    fn produce(&self, rm: &mut ResourceMetrics) -> Result<()> {
        let inner = self.inner.lock()?;
        for cb in &inner.callbacks {
            // TODO consider parallel callbacks.
            cb();
        }

        for mcb in inner.multi_callbacks.iter().flatten() {
            // TODO consider parallel multi callbacks.
            mcb();
        }

        rm.resource = self.resource.clone();
        rm.scope_metrics.reserve(inner.aggregations.len());

        let mut i = 0;
        for (scope, instruments) in inner.aggregations.iter() {
            let sm = match rm.scope_metrics.get_mut(i) {
                Some(sm) => sm,
                None => {
                    rm.scope_metrics.push(ScopeMetrics::default());
                    rm.scope_metrics.last_mut().unwrap()
                }
            };
            sm.metrics.reserve(instruments.len());

            let mut j = 0;
            for inst in instruments {
                if let Some(data) = inst.aggregator.aggregation() {
                    let m = Metric {
                        name: inst.name.clone(),
                        description: inst.description.clone(),
                        unit: inst.unit.clone(),
                        data,
                    };
                    match sm.metrics.get_mut(j) {
                        Some(old) => *old = m,
                        None => sm.metrics.push(m),
                    };
                    j += 1;
                }
            }

            sm.metrics.truncate(j);
            if !sm.metrics.is_empty() {
                sm.scope = scope.clone();
                i += 1;
            }
        }

        rm.scope_metrics.truncate(i);

        Ok(())
    }
}

trait Aggregator: Send + Sync {
    fn aggregation(&self) -> Option<Box<dyn data::Aggregation>>;
}

impl<T> Aggregator for Arc<dyn internal::Aggregator<T>> {
    fn aggregation(&self) -> Option<Box<dyn data::Aggregation>> {
        self.as_ref().aggregation()
    }
}

/// A synchronization point between a [Pipeline] and an instrument's aggregators.
struct InstrumentSync {
    name: Cow<'static, str>,
    description: Cow<'static, str>,
    unit: Unit,
    aggregator: Box<dyn Aggregator>,
}

impl fmt::Debug for InstrumentSync {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InstrumentSync")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("unit", &self.unit)
            .finish()
    }
}

type Cache<T> = Mutex<HashMap<StreamId, Result<Option<Arc<dyn internal::Aggregator<T>>>>>>;

/// Facilitates inserting of new instruments from a single scope into a pipeline.
struct Inserter<T> {
    /// A cache that holds [Aggregator]s inserted into the underlying reader pipeline.
    ///
    /// This cache ensures no duplicate `Aggregator`s are inserted into the reader
    /// pipeline and if a new request during an instrument creation asks for the same
    /// `Aggregator` the same instance is returned.
    aggregators: Cache<T>,

    /// A cache that holds instrument identifiers for all the instruments a [Meter] has
    /// created.
    ///
    /// It is provided from the `Meter` that owns this inserter. This cache ensures
    /// that during the creation of instruments with the same name but different
    /// options (e.g. description, unit) a warning message is logged.
    views: Arc<Mutex<HashMap<Cow<'static, str>, StreamId>>>,

    pipeline: Arc<Pipeline>,
}

impl<T> Inserter<T>
where
    T: Number<T>,
{
    fn new(p: Arc<Pipeline>, vc: Arc<Mutex<HashMap<Cow<'static, str>, StreamId>>>) -> Self {
        Inserter {
            aggregators: Default::default(),
            views: vc,
            pipeline: Arc::clone(&p),
        }
    }

    /// Inserts the provided instrument into a pipeline.
    ///
    /// All views the pipeline contains are matched against, and any matching view
    /// that creates a unique [Aggregator] will be inserted into the pipeline and
    /// included in the returned list.
    ///
    /// The returned `Aggregator`s are ensured to be deduplicated and unique. If
    /// another view in another pipeline that is cached by this inserter's cache has
    /// already inserted the same `Aggregator` for the same instrument, that
    /// `Aggregator` instance is returned.
    ///
    /// If another instrument has already been inserted by this inserter, or any
    /// other using the same cache, and it conflicts with the instrument being
    /// inserted in this call, an `Aggregator` matching the arguments will still be
    /// returned but a log message will also be logged to the OTel global logger.
    ///
    /// If the passed instrument would result in an incompatible `Aggregator`, an
    /// error is returned and that `Aggregator` is not inserted or returned.
    ///
    /// If an instrument is determined to use a [aggregation::Aggregation::Drop], that instrument is
    /// not inserted nor returned.
    fn instrument(&self, inst: Instrument) -> Result<Vec<Arc<dyn internal::Aggregator<T>>>> {
        let mut matched = false;
        let mut aggs = vec![];
        let mut errs = vec![];
        let kind = match inst.kind {
            Some(kind) => kind,
            None => return Err(MetricsError::Other("instrument must have a kind".into())),
        };

        // The cache will return the same Aggregator instance. Use stream ids to de duplicate.
        let mut seen = HashSet::new();
        for v in &self.pipeline.views {
            let stream = match v.match_inst(&inst) {
                Some(stream) => stream,
                None => continue,
            };
            matched = true;

            let id = self.stream_id(kind, &stream);
            if seen.contains(&id) {
                continue; // This aggregator has already been added
            }

            let agg = match self.cached_aggregator(&inst.scope, kind, stream) {
                Ok(Some(agg)) => agg,
                Ok(None) => continue, // Drop aggregator.
                Err(err) => {
                    errs.push(err);
                    continue;
                }
            };
            seen.insert(id);
            aggs.push(agg);
        }

        if matched {
            if errs.is_empty() {
                return Ok(aggs);
            } else {
                return Err(MetricsError::Other(format!("{errs:?}")));
            }
        }

        // Apply implicit default view if no explicit matched.
        let stream = Stream {
            name: inst.name,
            description: inst.description,
            unit: inst.unit,
            aggregation: None,
            attribute_filter: None,
        };

        match self.cached_aggregator(&inst.scope, kind, stream) {
            Ok(agg) => {
                if errs.is_empty() {
                    if let Some(agg) = agg {
                        aggs.push(agg);
                    }
                    Ok(aggs)
                } else {
                    Err(MetricsError::Other(format!("{errs:?}")))
                }
            }
            Err(err) => {
                errs.push(err);
                Err(MetricsError::Other(format!("{errs:?}")))
            }
        }
    }

    /// Returns the appropriate Aggregator for an instrument
    /// configuration. If the exact instrument has been created within the
    /// inst.Scope, that Aggregator instance will be returned. Otherwise, a new
    /// computed Aggregator will be cached and returned.
    ///
    /// If the instrument configuration conflicts with an instrument that has
    /// already been created (e.g. description, unit, data type) a warning will be
    /// logged at the "Info" level with the global OTel logger. A valid new
    /// Aggregator for the instrument configuration will still be returned without
    /// an error.
    ///
    /// If the instrument defines an unknown or incompatible aggregation, an error
    /// is returned.
    fn cached_aggregator(
        &self,
        scope: &Scope,
        kind: InstrumentKind,
        mut stream: Stream,
    ) -> Result<Option<Arc<dyn internal::Aggregator<T>>>> {
        let agg = if let Some(agg) = stream.aggregation.as_ref() {
            agg
        } else {
            stream.aggregation = Some(self.pipeline.reader.aggregation(kind));
            stream.aggregation.as_ref().unwrap()
        };

        if let Err(err) = is_aggregator_compatible(&kind, agg) {
            return Err(MetricsError::Other(format!(
                "creating aggregator with instrumentKind: {:?}, aggregation {:?}: {:?}",
                kind, stream.aggregation, err,
            )));
        }

        let id = self.stream_id(kind, &stream);
        // If there is a conflict, the specification says the view should
        // still be applied and a warning should be logged.
        self.log_conflict(&id);
        let (id_temporality, id_monotonic) = (id.temporality, id.monotonic);
        let mut cache = self.aggregators.lock()?;
        let cached = cache.entry(id).or_insert_with(|| {
            let mut agg = match self.aggregator(agg, kind, id_temporality, id_monotonic) {
                Ok(Some(agg)) => agg,
                other => return other, // Drop aggregator or error
            };

            if let Some(filter) = &stream.attribute_filter {
                agg = internal::new_filter(agg, Arc::clone(filter));
            }

            self.pipeline.add_sync(
                scope.clone(),
                InstrumentSync {
                    name: stream.name,
                    description: stream.description,
                    unit: stream.unit,
                    aggregator: Box::new(Arc::clone(&agg)),
                },
            );

            Ok(Some(agg))
        });

        cached
            .as_ref()
            .map(|o| o.as_ref().map(Arc::clone))
            .map_err(|e| MetricsError::Other(e.to_string()))
    }

    /// Validates if an instrument with the same name as id has already been created.
    ///
    /// If that instrument conflicts with id, a warning is logged.
    fn log_conflict(&self, id: &StreamId) {
        let _ = self.views.lock().map(|views| {
            if let Some(existing) = views.get(&id.name) {
                if existing == id { return; }
                global::handle_error(MetricsError::Other(format!(
                "duplicate metric stream definitions, names: ({} and {}), descriptions: ({} and {}), units: ({:?} and {:?}), numbers: ({} and {}), aggregations: ({:?} and {:?}), monotonics: ({} and {}), temporalities: ({:?} and {:?})",
                existing.name, id.name,
                existing.description, id.description,
                existing.unit, id.unit,
                existing.number, id.number,
                existing.aggregation, id.aggregation,
                existing.monotonic, id.monotonic,
                existing.temporality, id.temporality)))
            }
        });
    }

    fn stream_id(&self, kind: InstrumentKind, stream: &Stream) -> StreamId {
        let aggregation = stream
            .aggregation
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default();

        StreamId {
            name: stream.name.clone(),
            description: stream.description.clone(),
            unit: stream.unit.clone(),
            aggregation,
            temporality: Some(self.pipeline.reader.temporality(kind)),
            number: Cow::Borrowed(std::any::type_name::<T>()),
            monotonic: matches!(
                kind,
                InstrumentKind::ObservableCounter
                    | InstrumentKind::Counter
                    | InstrumentKind::Histogram
            ),
        }
    }

    /// Returns a new [Aggregator] for the given params.
    ///
    /// If the aggregation is unknown or temporality is invalid, an error is returned.
    fn aggregator(
        &self,
        agg: &aggregation::Aggregation,
        kind: InstrumentKind,
        temporality: Option<Temporality>,
        monotonic: bool,
    ) -> Result<Option<Arc<dyn internal::Aggregator<T>>>> {
        use aggregation::Aggregation;
        match agg {
            Aggregation::Drop => Ok(None),
            Aggregation::LastValue => Ok(Some(internal::new_last_value())),
            Aggregation::Sum => {
                match kind {
                    InstrumentKind::ObservableCounter | InstrumentKind::ObservableUpDownCounter => {
                        // Asynchronous counters and up-down-counters are defined to record
                        // the absolute value of the count:
                        // https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/api.md#asynchronous-counter-creation
                        match temporality {
                            Some(Temporality::Cumulative) => {
                                return Ok(Some(internal::new_precomputed_cumulative_sum(
                                    monotonic,
                                )))
                            }
                            Some(Temporality::Delta) => {
                                return Ok(Some(internal::new_precomputed_delta_sum(monotonic)))
                            }
                            _ => {
                                return Err(MetricsError::Other(format!(
                                    "unrecognized temporality: {:?}",
                                    temporality
                                )))
                            }
                        }
                    }
                    _ => {}
                };

                match temporality {
                    Some(Temporality::Cumulative) => {
                        Ok(Some(internal::new_cumulative_sum(monotonic)))
                    }
                    Some(Temporality::Delta) => Ok(Some(internal::new_delta_sum(monotonic))),
                    _ => Err(MetricsError::Other(format!(
                        "unrecognized temporality: {:?}",
                        temporality
                    ))),
                }
            }
            a @ Aggregation::ExplicitBucketHistogram { .. } => match temporality {
                Some(Temporality::Cumulative) => Ok(Some(internal::new_cumulative_histogram(a))),
                Some(Temporality::Delta) => Ok(Some(internal::new_delta_histogram(a))),
                _ => Err(MetricsError::Other(format!(
                    "unrecognized temporality: {:?}",
                    temporality
                ))),
            },
            _ => Err(MetricsError::Other("unknown aggregation".into())),
        }
    }
}

/// Checks if the aggregation can be used by the instrument.
///
/// Current compatibility:
///
/// | Instrument Kind          | Drop | LastValue | Sum | Histogram | Exponential Histogram |
/// |--------------------------|------|-----------|-----|-----------|-----------------------|
/// | Counter                  | X    |           | X   | X         | X                     |
/// | UpDownCounter            | X    |           | X   |           |                       |
/// | Histogram                | X    |           | X   | X         | X                     |
/// | Observable Counter       | X    |           | X   |           |                       |
/// | Observable UpDownCounter | X    |           | X   |           |                       |
/// | Observable Gauge         | X    | X         |     |           |                       |.
fn is_aggregator_compatible(kind: &InstrumentKind, agg: &aggregation::Aggregation) -> Result<()> {
    use aggregation::Aggregation;
    match agg {
        Aggregation::ExplicitBucketHistogram { .. } => {
            if kind == &InstrumentKind::Counter || kind == &InstrumentKind::Histogram {
                return Ok(());
            }
            // TODO: review need for aggregation check after
            // https://github.com/open-telemetry/opentelemetry-specification/issues/2710
            Err(MetricsError::Other("incompatible aggregation".into()))
        }
        Aggregation::Sum => {
            match kind {
                InstrumentKind::ObservableCounter
                | InstrumentKind::ObservableUpDownCounter
                | InstrumentKind::Counter
                | InstrumentKind::Histogram
                | InstrumentKind::UpDownCounter => Ok(()),
                _ => {
                    // TODO: review need for aggregation check after
                    // https://github.com/open-telemetry/opentelemetry-specification/issues/2710
                    Err(MetricsError::Other("incompatible aggregation".into()))
                }
            }
        }
        Aggregation::LastValue => {
            if kind == &InstrumentKind::ObservableGauge {
                return Ok(());
            }
            // TODO: review need for aggregation check after
            // https://github.com/open-telemetry/opentelemetry-specification/issues/2710
            Err(MetricsError::Other("incompatible aggregation".into()))
        }
        Aggregation::Drop => Ok(()),
        _ => {
            // This is used passed checking for default, it should be an error at this point.
            Err(MetricsError::Other(format!(
                "unknown aggregation {:?}",
                agg
            )))
        }
    }
}

/// The group of pipelines connecting Readers with instrument measurement.
#[derive(Clone, Debug)]
pub(crate) struct Pipelines(Vec<Arc<Pipeline>>);

impl Pipelines {
    pub(crate) fn new(
        res: Resource,
        readers: Vec<Box<dyn MetricReader>>,
        views: Vec<Arc<dyn View>>,
    ) -> Self {
        let mut pipes = Vec::with_capacity(readers.len());
        for r in readers {
            let p = Arc::new(Pipeline {
                resource: res.clone(),
                reader: r,
                views: views.clone(),
                inner: Default::default(),
            });
            p.reader.register_pipeline(Arc::downgrade(&p));
            pipes.push(p);
        }

        Pipelines(pipes)
    }

    pub(crate) fn register_callback<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let cb = Arc::new(callback);
        for pipe in &self.0 {
            pipe.add_callback(cb.clone())
        }
    }

    /// Registers a multi-instrument callback to be run when `produce` is called.
    pub(crate) fn register_multi_callback<F>(&self, f: F) -> Result<Box<dyn CallbackRegistration>>
    where
        F: Fn() + Send + Sync + 'static,
    {
        let cb = Arc::new(f);

        let fns = self
            .0
            .iter()
            .map(|pipe| {
                let pipe = Arc::clone(pipe);
                let unreg = pipe.add_multi_callback(cb.clone())?;
                Ok(Box::new(move || unreg(pipe.as_ref())) as _)
            })
            .collect::<Result<_>>()?;

        Ok(Box::new(Unregister(fns)))
    }

    /// Force flush all pipelines
    pub(crate) fn force_flush(&self, cx: &Context) -> Result<()> {
        let mut errs = vec![];
        for pipeline in &self.0 {
            if let Err(err) = pipeline.force_flush(cx) {
                errs.push(err);
            }
        }

        if errs.is_empty() {
            Ok(())
        } else {
            Err(MetricsError::Other(format!("{errs:?}")))
        }
    }

    /// Shut down all pipelines
    pub(crate) fn shutdown(&self) -> Result<()> {
        let mut errs = vec![];
        for pipeline in &self.0 {
            if let Err(err) = pipeline.shutdown() {
                errs.push(err);
            }
        }

        if errs.is_empty() {
            Ok(())
        } else {
            Err(MetricsError::Other(format!("{errs:?}")))
        }
    }
}

struct Unregister(Vec<Box<dyn FnOnce() -> Result<()> + Send + Sync>>);

impl CallbackRegistration for Unregister {
    fn unregister(&mut self) -> Result<()> {
        let mut errs = vec![];
        while let Some(unreg) = self.0.pop() {
            if let Err(err) = unreg() {
                errs.push(err);
            }
        }

        if errs.is_empty() {
            Ok(())
        } else {
            Err(MetricsError::Other(format!("{errs:?}")))
        }
    }
}

/// resolver facilitates resolving Aggregators an instrument needs to aggregate
/// measurements with while updating all pipelines that need to pull from those
/// aggregations.
pub(crate) struct Resolver<T> {
    inserters: Vec<Inserter<T>>,
}

impl<T> Resolver<T>
where
    T: Number<T>,
{
    pub(crate) fn new(
        pipelines: Arc<Pipelines>,
        view_cache: Arc<Mutex<HashMap<Cow<'static, str>, StreamId>>>,
    ) -> Self {
        let inserters = pipelines.0.iter().fold(Vec::new(), |mut acc, pipe| {
            acc.push(Inserter::new(Arc::clone(pipe), Arc::clone(&view_cache)));
            acc
        });

        Resolver { inserters }
    }

    /// Aggregators returns the Aggregators that must be updated by the instrument
    /// defined by key.
    pub(crate) fn aggregators(
        &self,
        id: Instrument,
    ) -> Result<Vec<Arc<dyn internal::Aggregator<T>>>> {
        let (aggs, errs) =
            self.inserters
                .iter()
                .fold((vec![], vec![]), |(mut aggs, mut errs), inserter| {
                    match inserter.instrument(id.clone()) {
                        Ok(agg) => aggs.extend(agg),
                        Err(err) => errs.push(err),
                    };
                    (aggs, errs)
                });

        if errs.is_empty() {
            Ok(aggs)
        } else {
            Err(MetricsError::Other(format!("{errs:?}")))
        }
    }
}
