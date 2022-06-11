//! # OpenTelemetry Metrics SDK
use crate::export;
use crate::export::metrics::{LockedProcessor, Processor};
use crate::metrics::{
    aggregators::Aggregator,
    sdk_api::{
        AsyncInstrumentCore, AtomicNumber, Descriptor, InstrumentCore, MeterCore, Number,
        NumberKind, SyncInstrumentCore,
    },
};
use fnv::FnvHasher;
use opentelemetry_api::{
    attributes::{hash_attributes, AttributeSet},
    global,
    metrics::Result,
    Context, KeyValue,
};
use std::{
    any::Any,
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    sync::{Arc, Mutex},
};
pub mod aggregators;
pub mod controllers;
pub mod processors;
pub mod registry;
pub mod sdk_api;
pub mod selectors;

/// Creates a new accumulator builder
pub fn accumulator(processor: Arc<dyn Processor + Send + Sync>) -> Accumulator {
    Accumulator(Arc::new(AccumulatorCore::new(processor)))
}

/// Accumulator implements the OpenTelemetry Meter API. The Accumulator is bound
/// to a single `Processor`.
///
/// The Accumulator supports a collect API to gather and export current data.
/// `Collect` should be arranged according to the processor model. Push-based
/// processors will setup a timer to call `collect` periodically. Pull-based
/// processors will call `collect` when a pull request arrives.
#[derive(Debug, Clone)]
pub struct Accumulator(Arc<AccumulatorCore>);

impl Accumulator {
    /// Traverses the list of active records and observers and
    /// exports data for each active instrument.
    ///
    /// During the collection pass, the [`LockedProcessor`] will receive
    /// one `export` call per current aggregation.
    ///
    /// Returns the number of records that were checkpointed.
    pub fn collect(&self, cx: &Context, locked_processor: &mut dyn LockedProcessor) -> usize {
        self.0.collect(cx, locked_processor)
    }
}

impl MeterCore for Accumulator {
    fn new_sync_instrument(
        &self,
        descriptor: Descriptor,
    ) -> Result<Arc<dyn SyncInstrumentCore + Send + Sync>> {
        Ok(Arc::new(SyncInstrument {
            instrument: Arc::new(BaseInstrument {
                meter: self.clone(),
                descriptor,
            }),
        }))
    }

    fn new_async_instrument(
        &self,
        descriptor: Descriptor,
    ) -> Result<Arc<dyn AsyncInstrumentCore + Send + Sync>> {
        Ok(Arc::new(AsyncInstrument {
            instrument: Arc::new(BaseInstrument {
                meter: self.clone(),
                descriptor,
            }),
        }))
    }

    fn register_callback(&self, f: Box<dyn Fn(&Context) + Send + Sync>) -> Result<()> {
        self.0
            .callbacks
            .lock()
            .map_err(Into::into)
            .map(|mut callbacks| callbacks.push(f))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct MapKey {
    instrument_hash: u64,
}

#[derive(Debug)]
struct AsyncContextKey;

type Callback = Box<dyn Fn(&Context) + Send + Sync>;

struct AccumulatorCore {
    /// A concurrent map of current sync instrument state.
    current: dashmap::DashMap<MapKey, Arc<Record>>,

    /// Async instrument callbacks
    callbacks: Mutex<Vec<Callback>>,

    /// The current epoch number. It is incremented in `collect`.
    current_epoch: AtomicNumber,

    /// The configured processor.
    processor: Arc<dyn Processor + Send + Sync>,
}

impl AccumulatorCore {
    fn new(processor: Arc<dyn Processor + Send + Sync>) -> Self {
        AccumulatorCore {
            current: dashmap::DashMap::new(),
            current_epoch: NumberKind::U64.zero().to_atomic(),
            processor,
            callbacks: Default::default(),
        }
    }

    fn collect(&self, cx: &Context, locked_processor: &mut dyn LockedProcessor) -> usize {
        self.run_async_callbacks(cx);
        let checkpointed = self.collect_instruments(locked_processor);
        self.current_epoch.fetch_add(&NumberKind::U64, &1u64.into());

        checkpointed
    }

    fn run_async_callbacks(&self, cx: &Context) {
        match self.callbacks.lock() {
            Ok(callbacks) => {
                let cx = cx.with_value(AsyncContextKey);
                for f in callbacks.iter() {
                    f(&cx)
                }
            }
            Err(err) => global::handle_error(err),
        }
    }

    fn collect_instruments(&self, locked_processor: &mut dyn LockedProcessor) -> usize {
        let mut checkpointed = 0;

        self.current.retain(|_key, value| {
            let mods = &value.update_count.load();
            let coll = &value.collected_count.load();

            if mods.partial_cmp(&NumberKind::U64, coll) != Some(Ordering::Equal) {
                // Updates happened in this interval,
                // checkpoint and continue.
                checkpointed += self.checkpoint_record(value, locked_processor);
                value.collected_count.store(mods);
            } else {
                // Having no updates since last collection, try to remove if
                // there are no bound handles
                if Arc::strong_count(value) == 1 {
                    // There's a potential race between loading collected count and
                    // loading the strong count in this function.  Since this is the
                    // last we'll see of this record, checkpoint.
                    if mods.partial_cmp(&NumberKind::U64, coll) != Some(Ordering::Equal) {
                        checkpointed += self.checkpoint_record(value, locked_processor);
                    }
                    return false;
                }
            };
            true
        });

        checkpointed
    }

    fn checkpoint_record(
        &self,
        record: &Record,
        locked_processor: &mut dyn LockedProcessor,
    ) -> usize {
        if let (Some(current), Some(checkpoint)) = (&record.current, &record.checkpoint) {
            if let Err(err) = current.synchronized_move(checkpoint, record.instrument.descriptor())
            {
                global::handle_error(err);

                return 0;
            }

            let accumulation = export::metrics::accumulation(
                record.instrument.descriptor(),
                &record.attributes,
                checkpoint,
            );
            if let Err(err) = locked_processor.process(accumulation) {
                global::handle_error(err);
            }

            1
        } else {
            0
        }
    }

    // fn checkpoint_async(
    //     &self,
    //     instrument: &AsyncInstrument,
    //     locked_processor: &mut dyn LockedProcessor,
    // ) -> usize {
    //     instrument.recorders.lock().map_or(0, |mut recorders| {
    //         let mut checkpointed = 0;
    //         match recorders.as_mut() {
    //             None => return checkpointed,
    //             Some(recorders) => {
    //                 recorders.retain(|_key, attribute_recorder| {
    //                     let epoch_diff = self.current_epoch.load().partial_cmp(
    //                         &NumberKind::U64,
    //                         &attribute_recorder.observed_epoch.into(),
    //                     );
    //                     if epoch_diff == Some(Ordering::Equal) {
    //                         if let Some(observed) = &attribute_recorder.observed {
    //                             let accumulation = export::metrics::accumulation(
    //                                 instrument.descriptor(),
    //                                 &attribute_recorder.attributes,
    //                                 &self.resource,
    //                                 observed,
    //                             );
    //
    //                             if let Err(err) = locked_processor.process(accumulation) {
    //                                 global::handle_error(err);
    //                             }
    //                             checkpointed += 1;
    //                         }
    //                     }
    //
    //                     // Retain if this is not second collection cycle with no
    //                     // observations for this AttributeSet.
    //                     epoch_diff == Some(Ordering::Greater)
    //                 });
    //             }
    //         }
    //         if recorders.as_ref().map_or(false, |map| map.is_empty()) {
    //             *recorders = None;
    //         }
    //
    //         checkpointed
    //     })
    // }
}

impl fmt::Debug for AccumulatorCore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AccumulatorCore").finish()
    }
}

#[derive(Debug, Clone)]
struct SyncInstrument {
    instrument: Arc<BaseInstrument>,
}

impl SyncInstrumentCore for SyncInstrument {
    fn record_one(&self, cx: &Context, number: sdk_api::Number, kvs: &'_ [KeyValue]) {
        self.instrument.acquire_handle(kvs).capture_one(cx, number)
    }
}

impl sdk_api::InstrumentCore for SyncInstrument {
    fn descriptor(&self) -> &Descriptor {
        self.instrument.descriptor()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
struct AsyncInstrument {
    instrument: Arc<BaseInstrument>,
}

impl AsyncInstrumentCore for AsyncInstrument {
    fn observe_one(&self, cx: &Context, number: Number, kvs: &'_ [KeyValue]) {
        self.instrument.acquire_handle(kvs).capture_one(cx, number)
    }
}

impl sdk_api::InstrumentCore for AsyncInstrument {
    fn descriptor(&self) -> &Descriptor {
        self.instrument.descriptor()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
struct BaseInstrument {
    meter: Accumulator,
    descriptor: Descriptor,
}

impl BaseInstrument {
    // acquireHandle gets or creates a `*record` corresponding to `kvs`,
    // the input attributes.
    fn acquire_handle(&self, kvs: &[KeyValue]) -> Arc<Record> {
        let mut hasher = FnvHasher::default();
        self.descriptor.attribute_hash().hash(&mut hasher);

        hash_attributes(&mut hasher, kvs.iter().map(|kv| (&kv.key, &kv.value)));

        let map_key = MapKey {
            instrument_hash: hasher.finish(),
        };
        let current = &self.meter.0.current;
        if let Some(existing_record) = current.get(&map_key) {
            return existing_record.value().clone();
        }

        let record = Arc::new(Record {
            update_count: NumberKind::U64.zero().to_atomic(),
            collected_count: NumberKind::U64.zero().to_atomic(),
            attributes: AttributeSet::from_attributes(kvs.iter().cloned()),
            instrument: self.clone(),
            current: self
                .meter
                .0
                .processor
                .aggregator_selector()
                .aggregator_for(&self.descriptor),
            checkpoint: self
                .meter
                .0
                .processor
                .aggregator_selector()
                .aggregator_for(&self.descriptor),
        });
        current.insert(map_key, record.clone());

        record
    }
}

impl InstrumentCore for BaseInstrument {
    fn descriptor(&self) -> &Descriptor {
        &self.descriptor
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// record maintains the state of one metric instrument.  Due
/// the use of lock-free algorithms, there may be more than one
/// `record` in existence at a time, although at most one can
/// be referenced from the `Accumulator.current` map.
#[derive(Debug)]
struct Record {
    /// Incremented on every call to `update`.
    update_count: AtomicNumber,

    /// Set to `update_count` on collection, supports checking for no updates during
    /// a round.
    collected_count: AtomicNumber,

    /// The processed attribute set for this record.
    ///
    /// TODO: look at perf here.
    attributes: AttributeSet,

    /// The corresponding instrument.
    instrument: BaseInstrument,

    /// current implements the actual `record_one` API, depending on the type of
    /// aggregation. If `None`, the metric was disabled by the exporter.
    current: Option<Arc<dyn Aggregator + Send + Sync>>,
    checkpoint: Option<Arc<dyn Aggregator + Send + Sync>>,
}

impl Record {
    fn capture_one(&self, cx: &Context, number: Number) {
        let current = match &self.current {
            Some(current) => current,
            // The instrument is disabled according to the AggregatorSelector.
            None => return,
        };
        if let Err(err) = aggregators::range_test(&number, &self.instrument.descriptor)
            .and_then(|_| current.update(cx, &number, &self.instrument.descriptor))
        {
            global::handle_error(err);
            return;
        }

        // Record was modified, inform the collect() that things need
        // to be collected while the record is still mapped.
        self.update_count.fetch_add(&NumberKind::U64, &1u64.into());
    }
}
