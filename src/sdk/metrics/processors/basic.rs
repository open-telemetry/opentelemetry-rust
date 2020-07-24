use crate::api::{
    labels::{hash_labels, LabelSet},
    metrics::{Descriptor, MetricsError, Result},
};
use crate::sdk::{
    export::metrics::{
        self, Accumulation, Aggregator, AggregatorSelector, CheckpointSet, ExportKind,
        ExportKindSelector, LockedProcessor, Processor, Record, Subtractor,
    },
    metrics::aggregators::SumAggregator,
    Resource,
};
use fnv::FnvHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::SystemTime;

/// Create a new basic processor
pub fn basic(
    aggregator_selector: Box<dyn AggregatorSelector + Send + Sync>,
    export_selector: Box<dyn ExportKindSelector + Send + Sync>,
    memory: bool,
) -> BasicProcessor {
    BasicProcessor {
        aggregator_selector,
        export_selector,
        state: Mutex::new(BasicProcessorState::with_memory(memory)),
    }
}

/// Basic metric integration strategy
#[derive(Debug)]
pub struct BasicProcessor {
    aggregator_selector: Box<dyn AggregatorSelector + Send + Sync>,
    export_selector: Box<dyn ExportKindSelector + Send + Sync>,
    state: Mutex<BasicProcessorState>,
}

impl BasicProcessor {
    /// Lock this processor to return a mutable locked processor
    pub fn lock(&self) -> Result<BasicLockedProcessor<'_>> {
        self.state
            .lock()
            .map_err(From::from)
            .map(|locked| BasicLockedProcessor {
                parent: self,
                state: locked,
            })
    }
}

impl Processor for BasicProcessor {
    fn aggregation_selector(&self) -> &dyn AggregatorSelector {
        self.aggregator_selector.as_ref()
    }
}

/// A locked representation of the processor used where mutable references are necessary.
#[derive(Debug)]
pub struct BasicLockedProcessor<'a> {
    parent: &'a BasicProcessor,
    state: MutexGuard<'a, BasicProcessorState>,
}

impl<'a> LockedProcessor for BasicLockedProcessor<'a> {
    fn process(&mut self, accumulation: Accumulation) -> Result<()> {
        if self.state.started_collection != self.state.finished_collection.wrapping_add(1) {
            return Err(MetricsError::InconsistentState);
        }

        let desc = accumulation.descriptor();
        let mut hasher = FnvHasher::default();
        desc.attribute_hash().hash(&mut hasher);
        hash_labels(&mut hasher, accumulation.labels().into_iter());
        // FIXME: convert resource to use labels::Set
        // accumulation.resource().equivalent().hash(&mut hasher);
        let key = StateKey(hasher.finish());
        let agg = accumulation.aggregator();
        let finished_collection = self.state.finished_collection;
        if let Some(value) = self.state.values.get_mut(&key) {
            // Advance the update sequence number.
            let same_collection = finished_collection == value.updated;
            value.updated = finished_collection;

            // At this point in the code, we have located an existing
            // value for some stateKey.  This can be because:
            //
            // (a) stateful aggregation is being used, the entry was
            // entered during a prior collection, and this is the first
            // time processing an accumulation for this stateKey in the
            // current collection.  Since this is the first time
            // processing an accumulation for this stateKey during this
            // collection, we don't know yet whether there are multiple
            // accumulators at work.  If there are multiple accumulators,
            // they'll hit case (b) the second time through.
            //
            // (b) multiple accumulators are being used, whether stateful
            // or not.
            //
            // Case (a) occurs when the instrument and the exporter
            // require memory to work correctly, either because the
            // instrument reports a PrecomputedSum to a DeltaExporter or
            // the reverse, a non-PrecomputedSum instrument with a
            // CumulativeExporter.  This logic is encapsulated in
            // ExportKind.MemoryRequired(MetricKind).
            //
            // Case (b) occurs when the variable `sameCollection` is true,
            // indicating that the stateKey for Accumulation has already
            // been seen in the same collection.  When this happens, it
            // implies that multiple Accumulators are being used because
            // the Accumulator outputs a maximum of one Accumulation per
            // instrument and label set.
            //
            // The following logic distinguishes between asynchronous and
            // synchronous instruments in order to ensure that the use of
            // multiple Accumulators does not change instrument semantics.
            // To maintain the instrument semantics, multiple synchronous
            // Accumulations should be merged, whereas when multiple
            // asynchronous Accumulations are processed, the last value
            // should be kept.

            if !same_collection {
                // This is the first Accumulation we've seen for this
                // stateKey during this collection.  Just keep a
                // reference to the Accumulator's Aggregator.
                value.current = agg.clone();
                return Ok(());
            }
            if desc.instrument_kind().asynchronous() {
                // The last value across multiple accumulators is taken.
                // Just keep a reference to the Accumulator's Aggregator.
                value.current = agg.clone();
                return Ok(());
            }

            // The above two cases are keeping a reference to the
            // Accumulator's Aggregator.  The remaining cases address
            // synchronous instruments, which always merge multiple
            // Accumulations using `value.delta` for temporary storage.

            if value.delta.is_none() {
                // The temporary `value.delta` may have been allocated
                // already, either in a prior pass through this block of
                // code or in the `!ok` branch above.  It would be
                // allocated in the `!ok` branch if this is stateful
                // PrecomputedSum instrument (in which case the exporter
                // is requesting a delta so we allocate it up front),
                // and it would be allocated in this block when multiple
                // accumulators are used and the first condition is not
                // met.
                value.delta = self.parent.aggregation_selector().aggregator_for(desc);
            }
            if Some(value.current.as_any().type_id())
                != value.delta.as_ref().map(|d| d.as_any().type_id())
            {
                // If the current and delta Aggregators are not the same it
                // implies that multiple Accumulators were used.  The first
                // Accumulation seen for a given stateKey will return in
                // one of the cases above after assigning `value.current
                // = agg` (i.e., after taking a reference to the
                // Accumulator's Aggregator).
                //
                // The second time through this branch copies the
                // Accumulator's Aggregator into `value.delta` and sets
                // `value.current` appropriately to avoid this branch if
                // a third Accumulator is used.
                value
                    .current
                    .synchronized_move(value.delta.as_ref().unwrap(), desc)?;
                value.current = value.delta.clone().unwrap();
            }
            // The two statements above ensures that `value.current` refers
            // to `value.delta` and not to an Accumulator's Aggregator.  Now
            // combine this Accumulation with the prior Accumulation.
            return value.delta.as_ref().unwrap().merge(agg.as_ref(), desc);
        }

        let stateful = self
            .parent
            .export_selector
            .export_kind_for(&desc)
            .memory_required(desc.instrument_kind());

        let mut delta = None;
        let cumulative = if stateful {
            if desc.instrument_kind().precomputed_sum() {
                // If we know we need to compute deltas, allocate two aggregators.
                delta = self.parent.aggregation_selector().aggregator_for(desc);
            }
            // In this case we are not certain to need a delta, only allocate a
            // cumulative aggregator.  We _may_ need a delta accumulator if
            // multiple synchronous Accumulators produce an Accumulation (handled
            // below), which requires merging them into a temporary Aggregator.
            self.parent.aggregation_selector().aggregator_for(desc)
        } else {
            None
        };

        self.state.values.insert(
            key,
            StateValue {
                // FIXME consider perf of all this
                current: agg.clone(),
                delta,
                cumulative,
                stateful,
                updated: finished_collection,
                descriptor: desc.clone(),
                labels: accumulation.labels().clone(),
                resource: accumulation.resource().clone(),
            },
        );

        Ok(())
    }

    fn checkpoint_set(&mut self) -> &mut dyn CheckpointSet {
        &mut *self.state
    }

    fn start_collection(&mut self) {
        if self.state.started_collection != 0 {
            self.state.interval_start = self.state.interval_end;
        }
        self.state.started_collection = self.state.started_collection.wrapping_add(1);
    }

    fn finish_collection(&mut self) -> Result<()> {
        self.state.interval_end = SystemTime::now();
        if self.state.started_collection != self.state.finished_collection.wrapping_add(1) {
            return Err(MetricsError::InconsistentState);
        }
        let finished_collection = self.state.finished_collection;
        self.state.finished_collection = self.state.finished_collection.wrapping_add(1);
        let has_memory = self.state.config.memory;

        let mut result = Ok(());

        self.state.values.retain(|_key, value| {
            // Return early if previous error
            if result.is_err() {
                return true;
            }

            let mkind = value.descriptor.instrument_kind();

            let stale = value.updated != finished_collection;
            let stateless = !value.stateful;

            // The following branch updates stateful aggregators. Skip these updates
            // if the aggregator is not stateful or if the aggregator is stale.
            if stale || stateless {
                // If this processor does not require memory, stale, stateless
                // entries can be removed. This implies that they were not updated
                // over the previous full collection interval.
                if stale && stateless && has_memory {
                    return false;
                }
            }

            // Update Aggregator state to support exporting either a
            // delta or a cumulative aggregation.
            if mkind.precomputed_sum() {
                if let Some(current_subtractor) =
                    value.current.as_any().downcast_ref::<SumAggregator>()
                {
                    // This line is equivalent to:
                    // value.delta = currentSubtractor - value.cumulative
                    if let (Some(cumulative), Some(delta)) =
                        (value.cumulative.as_ref(), value.delta.as_ref())
                    {
                        result = current_subtractor
                            .subtract(cumulative.as_ref(), delta.as_ref(), &value.descriptor)
                            .and_then(|_| {
                                value
                                    .current
                                    .synchronized_move(cumulative, &value.descriptor)
                            });
                    }
                } else {
                    result = Err(MetricsError::NoSubtraction);
                }
            } else {
                // This line is equivalent to:
                // value.cumulative = value.cumulative + value.delta
                if let Some(cumulative) = value.cumulative.as_ref() {
                    result = cumulative.merge(value.current.as_ref(), &value.descriptor)
                }
            }

            true
        });

        result
    }
}

#[derive(Debug, Default)]
struct BasicProcessorConfig {
    /// Memory controls whether the processor remembers metric instruments and label
    /// sets that were previously reported. When Memory is true,
    /// `CheckpointSet::try_for_each` will visit metrics that were not updated in
    /// the most recent interval.
    memory: bool,
}

#[derive(Debug)]
struct BasicProcessorState {
    config: BasicProcessorConfig,
    values: HashMap<StateKey, StateValue>,
    // Note: the timestamp logic currently assumes all exports are deltas.
    process_start: SystemTime,
    interval_start: SystemTime,
    interval_end: SystemTime,
    started_collection: u64,
    finished_collection: u64,
}

impl BasicProcessorState {
    fn with_memory(memory: bool) -> Self {
        let mut state = BasicProcessorState::default();
        state.config.memory = memory;
        state
    }
}

impl Default for BasicProcessorState {
    fn default() -> Self {
        BasicProcessorState {
            config: BasicProcessorConfig::default(),
            values: HashMap::default(),
            process_start: SystemTime::now(),
            interval_start: SystemTime::now(),
            interval_end: SystemTime::now(),
            started_collection: 0,
            finished_collection: 0,
        }
    }
}

impl CheckpointSet for BasicProcessorState {
    fn try_for_each(
        &mut self,
        exporter: &dyn ExportKindSelector,
        f: &mut dyn FnMut(&Record) -> Result<()>,
    ) -> Result<()> {
        if self.started_collection != self.finished_collection {
            return Err(MetricsError::InconsistentState);
        }

        self.values.iter().try_for_each(|(_key, value)| {
            let instrument_kind = value.descriptor.instrument_kind();

            let agg;
            let start;

            // If the processor does not have memory and it was not updated in the
            // prior round, do not visit this value.
            if !self.config.memory && value.updated != self.finished_collection.wrapping_sub(1) {
                return Ok(());
            }

            match exporter.export_kind_for(&value.descriptor) {
                ExportKind::PassThrough => {
                    // No state is required, pass through the checkpointed value.
                    agg = Some(&value.current);

                    if instrument_kind.precomputed_sum() {
                        start = self.process_start;
                    } else {
                        start = self.interval_start;
                    }
                }

                ExportKind::Cumulative => {
                    // If stateful, the sum has been computed.  If stateless, the
                    // input was already cumulative. Either way, use the
                    // checkpointed value:
                    if value.stateful {
                        agg = value.cumulative.as_ref();
                    } else {
                        agg = Some(&value.current);
                    }

                    start = self.process_start;
                }

                ExportKind::Delta => {
                    // Precomputed sums are a special case.
                    if instrument_kind.precomputed_sum() {
                        agg = value.delta.as_ref();
                    } else {
                        agg = Some(&value.current);
                    }

                    start = self.interval_start;
                }
            }

            let res = f(&metrics::record(
                &value.descriptor,
                &value.labels,
                &value.resource,
                agg,
                start,
                self.interval_end,
            ));
            if res == Err(MetricsError::NoDataCollected) {
                Ok(())
            } else {
                res
            }
        })
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct StateKey(u64);

#[derive(Debug)]
struct StateValue {
    /// Indicates the last sequence number when this value had process called by an
    /// accumulator.
    updated: u64,

    /// Indicates that a cumulative aggregation is being maintained, taken from the
    /// process start time.
    stateful: bool,

    // TODO: as seen in lengthy comments below, both the `current` and `delta`
    // fields have multiple uses depending on the specific configuration of
    // instrument, exporter, and accumulator.  It is possible to simplify this
    // situation by declaring explicit fields that are not used with a dual purpose.
    // Improve this situation?
    //
    // 1. "delta" is used to combine deltas from multiple accumulators, and it is
    //    also used to store the output of subtraction when computing deltas of
    //    PrecomputedSum instruments.
    //
    // 2. "current" either refers to the Aggregator passed to process() by a single
    //    accumulator (when either there is just one Accumulator, or the instrument
    //    is Asynchronous), or it refers to "delta", depending on configuration.
    //
    /// Refers to single-accumulator checkpoint or delta.
    current: Arc<dyn Aggregator + Send + Sync>,

    /// Owned if multi accumulator else `None`.
    delta: Option<Arc<dyn Aggregator + Send + Sync>>,

    /// Owned if stateful else `None`.
    cumulative: Option<Arc<dyn Aggregator + Send + Sync>>,

    descriptor: Descriptor,
    labels: LabelSet,
    resource: Resource,
}
