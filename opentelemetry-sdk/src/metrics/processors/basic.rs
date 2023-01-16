use crate::{
    export::metrics::{
        self,
        aggregation::{Temporality, TemporalitySelector},
        Accumulation, AggregatorSelector, Checkpointer, CheckpointerFactory, LockedCheckpointer,
        LockedProcessor, Processor, Reader, Record,
    },
    metrics::{aggregators::Aggregator, sdk_api::Descriptor},
};
use core::fmt;
use fnv::FnvHasher;
use opentelemetry_api::{
    attributes::{hash_attributes, AttributeSet},
    metrics::{MetricsError, Result},
};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::SystemTime;

/// Create a new basic processor
pub fn factory<A, T>(aggregator_selector: A, temporality_selector: T) -> BasicProcessorBuilder
where
    A: AggregatorSelector + Send + Sync + 'static,
    T: TemporalitySelector + Send + Sync + 'static,
{
    BasicProcessorBuilder {
        aggregator_selector: Arc::new(aggregator_selector),
        temporality_selector: Arc::new(temporality_selector),
    }
}

pub struct BasicProcessorBuilder {
    aggregator_selector: Arc<dyn AggregatorSelector + Send + Sync>,
    temporality_selector: Arc<dyn TemporalitySelector + Send + Sync>,
}

impl fmt::Debug for BasicProcessorBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BasicProcessorBuilder")
            .finish()
    }
}

impl CheckpointerFactory for BasicProcessorBuilder {
    fn checkpointer(&self) -> Arc<dyn Checkpointer + Send + Sync> {
        Arc::new(BasicProcessor {
            aggregator_selector: Arc::clone(&self.aggregator_selector),
            temporality_selector: Arc::clone(&self.temporality_selector),
            state: Mutex::new(BasicProcessorState::default()),
        })
    }
}

/// Basic metric integration strategy
pub struct BasicProcessor {
    aggregator_selector: Arc<dyn AggregatorSelector + Send + Sync>,
    temporality_selector: Arc<dyn TemporalitySelector + Send + Sync>,
    state: Mutex<BasicProcessorState>,
}

impl Processor for BasicProcessor {
    fn aggregator_selector(&self) -> &dyn AggregatorSelector {
        self.aggregator_selector.as_ref()
    }
}

impl Checkpointer for BasicProcessor {
    fn checkpoint(
        &self,
        f: &mut dyn FnMut(&mut dyn LockedCheckpointer) -> Result<()>,
    ) -> Result<()> {
        self.state.lock().map_err(From::from).and_then(|locked| {
            f(&mut BasicLockedProcessor {
                parent: self,
                state: locked,
            })
        })
    }
}

impl fmt::Debug for BasicProcessor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BasicProcessor")
            .field("state", &self.state)
            .finish()
    }
}

/// A locked representation of the processor used where mutable references are necessary.
#[derive(Debug)]
struct BasicLockedProcessor<'a> {
    parent: &'a BasicProcessor,
    state: MutexGuard<'a, BasicProcessorState>,
}

impl<'a> LockedProcessor for BasicLockedProcessor<'a> {
    fn process(&mut self, accumulation: Accumulation<'_>) -> Result<()> {
        if self.state.started_collection != self.state.finished_collection.wrapping_add(1) {
            return Err(MetricsError::InconsistentState);
        }

        let desc = accumulation.descriptor();
        let mut hasher = FnvHasher::default();
        desc.attribute_hash().hash(&mut hasher);
        hash_attributes(&mut hasher, accumulation.attributes().into_iter());
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
            // implies that multiple Accumulators are being used, or that
            // a single Accumulator has been configured with an attribute key
            // filter.

            if !same_collection {
                if !value.current_owned {
                    // This is the first Accumulation we've seen for this
                    // stateKey during this collection.  Just keep a
                    // reference to the Accumulator's Aggregator. All the other cases
                    // copy Aggregator state.
                    value.current = agg.clone();
                    return Ok(());
                }
                return agg.synchronized_move(&value.current, desc);
            }

            // If the current is not owned, take ownership of a copy
            // before merging below.
            if !value.current_owned {
                let tmp = value.current.clone();
                if let Some(current) = self.parent.aggregator_selector.aggregator_for(desc) {
                    value.current = current;
                    value.current_owned = true;
                    tmp.synchronized_move(&value.current, desc)?;
                }
            }

            // Combine this `Accumulation` with the prior `Accumulation`.
            return value.current.merge(agg.as_ref(), desc);
        }

        let stateful = self
            .parent
            .temporality_selector
            .temporality_for(desc, agg.aggregation().kind())
            .memory_required(desc.instrument_kind());

        let cumulative = if stateful {
            if desc.instrument_kind().precomputed_sum() {
                // If we know we need to compute deltas, allocate one.
                return Err(MetricsError::Other("No cumulative to sum support".into()));
            }
            // Always allocate a cumulative aggregator if stateful
            self.parent.aggregator_selector.aggregator_for(desc)
        } else {
            None
        };

        self.state.values.insert(
            key,
            StateValue {
                descriptor: desc.clone(),
                attributes: accumulation.attributes().clone(),
                current_owned: false,
                current: agg.clone(),
                cumulative,
                stateful,
                updated: finished_collection,
            },
        );

        Ok(())
    }
}

impl LockedCheckpointer for BasicLockedProcessor<'_> {
    fn processor(&mut self) -> &mut dyn LockedProcessor {
        self
    }

    fn reader(&mut self) -> &mut dyn Reader {
        &mut *self.state
    }

    fn start_collection(&mut self) {
        if self.state.started_collection != 0 {
            self.state.interval_start = self.state.interval_end;
        }
        self.state.started_collection = self.state.started_collection.wrapping_add(1);
    }

    fn finish_collection(&mut self) -> Result<()> {
        self.state.interval_end = opentelemetry_api::time::now();
        if self.state.started_collection != self.state.finished_collection.wrapping_add(1) {
            return Err(MetricsError::InconsistentState);
        }
        let finished_collection = self.state.finished_collection;
        self.state.finished_collection = self.state.finished_collection.wrapping_add(1);

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
                if stale && stateless {
                    return false;
                }
                return true;
            }

            // The only kind of aggregators that are not stateless
            // are the ones needing delta to cumulative
            // conversion.  Merge aggregator state in this case.
            if !mkind.precomputed_sum() {
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


#[derive(Debug)]
struct BasicProcessorState {
    values: HashMap<StateKey, StateValue>,
    // Note: the timestamp logic currently assumes all exports are deltas.
    process_start: SystemTime,
    interval_start: SystemTime,
    interval_end: SystemTime,
    started_collection: u64,
    finished_collection: u64,
}

impl Default for BasicProcessorState {
    fn default() -> Self {
        BasicProcessorState {
            values: HashMap::default(),
            process_start: opentelemetry_api::time::now(),
            interval_start: opentelemetry_api::time::now(),
            interval_end: opentelemetry_api::time::now(),
            started_collection: 0,
            finished_collection: 0,
        }
    }
}

impl Reader for BasicProcessorState {
    fn try_for_each(
        &mut self,
        temporality_selector: &dyn TemporalitySelector,
        f: &mut dyn FnMut(&Record<'_>) -> Result<()>,
    ) -> Result<()> {
        if self.started_collection != self.finished_collection {
            return Err(MetricsError::InconsistentState);
        }

        self.values.iter().try_for_each(|(_key, value)| {
            let instrument_kind = value.descriptor.instrument_kind();

            let agg;
            let start;

            match temporality_selector
                .temporality_for(&value.descriptor, value.current.aggregation().kind())
            {
                Temporality::Cumulative => {
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
                Temporality::Delta => {
                    // Precomputed sums are a special case.
                    if instrument_kind.precomputed_sum() {
                        return Err(MetricsError::Other("No cumulative to delta".into()));
                    }

                    if  value.updated != self.finished_collection.wrapping_sub(1) {
                        // skip processing if there is no update in last collection internal and
                        // temporality is Delta
                        return Ok(())
                    }

                    agg = Some(&value.current);
                    start = self.interval_start;
                }
            }

            let res = f(&metrics::record(
                &value.descriptor,
                &value.attributes,
                agg,
                start,
                self.interval_end,
            ));

            if let Err(MetricsError::NoDataCollected) = res {
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
    /// Instrument descriptor
    descriptor: Descriptor,

    /// Instrument attributes
    attributes: AttributeSet,

    /// Indicates the last sequence number when this value had process called by an
    /// accumulator.
    updated: u64,

    /// Indicates that a cumulative aggregation is being maintained, taken from the
    /// process start time.
    stateful: bool,

    /// Indicates that "current" was allocated
    /// by the processor in order to merge results from
    /// multiple `Accumulator`s during a single collection
    /// round, which may happen either because:
    ///
    /// (1) multiple `Accumulator`s output the same `Accumulation`.
    /// (2) one `Accumulator` is configured with dimensionality reduction.
    current_owned: bool,

    /// The output from a single `Accumulator` (if !current_owned) or an
    /// `Aggregator` owned by the processor used to accumulate multiple values in a
    /// single collection round.
    current: Arc<dyn Aggregator + Send + Sync>,

    /// If `Some`, refers to an `Aggregator` owned by the processor used to store
    /// the last cumulative value.
    cumulative: Option<Arc<dyn Aggregator + Send + Sync>>,
}
