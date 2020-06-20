use crate::api::{
    labels,
    metrics::{Descriptor, MetricsError, Result},
};
use crate::sdk::{
    export::metrics::{
        self, Accumulation, AggregationSelector, Aggregator, CheckpointSet, Integrator,
        LockedIntegrator, Record,
    },
    Resource,
};
use std::collections::{hash_map::DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::SystemTime;

/// Create a new simple integrator
pub fn simple(
    selector: Box<dyn AggregationSelector + Send + Sync>,
    stateful: bool,
) -> SimpleIntegrator {
    SimpleIntegrator {
        aggregation_selector: selector,
        stateful,
        batch: Mutex::new(SimpleIntegratorBatch::default()),
    }
}

/// Simple metric integration strategy
#[derive(Debug)]
pub struct SimpleIntegrator {
    aggregation_selector: Box<dyn AggregationSelector + Send + Sync>,
    stateful: bool,
    batch: Mutex<SimpleIntegratorBatch>,
}

impl SimpleIntegrator {
    /// Lock this integrator to return a mutable locked integrator
    pub fn lock(&self) -> Result<SimpleLockedIntegrator<'_>> {
        self.batch
            .lock()
            .map_err(From::from)
            .map(|locked| SimpleLockedIntegrator {
                parent: self,
                batch: locked,
            })
    }
}

impl Integrator for SimpleIntegrator {
    fn aggregation_selector(&self) -> &dyn AggregationSelector {
        self.aggregation_selector.as_ref()
    }
}

/// A locked representation of the integrator used where mutable references are necessary.
#[derive(Debug)]
pub struct SimpleLockedIntegrator<'a> {
    parent: &'a SimpleIntegrator,
    batch: MutexGuard<'a, SimpleIntegratorBatch>,
}

impl<'a> LockedIntegrator for SimpleLockedIntegrator<'a> {
    fn process(&mut self, accumulation: Accumulation) -> Result<()> {
        if self.batch.started_collection != self.batch.finished_collection.wrapping_add(1) {
            return Err(MetricsError::InconsistentState);
        }

        let desc = accumulation.descriptor();
        let mut hasher = DefaultHasher::new();
        desc.hash(&mut hasher);
        accumulation.labels().equivalent().hash(&mut hasher);
        // FIXME: convert resource to use labels::Set
        // accumulation.resource().equivalent().hash(&mut hasher);
        let key = BatchKey(hasher.finish());
        let agg = accumulation.aggregator();
        let mut new_agg = None;
        if let Some(value) = self.batch.values.get(&key) {
            // Note: The call to Merge here combines only identical accumulations.
            // It is required even for a stateless Integrator because such identical
            // accumulations may arise in the Meter implementation due to race
            // conditions.
            if self.parent.stateful {
                return value.aggregator.merge(agg.as_ref(), desc);
            } else {
                // FIXME: consider deadlock case for stateless aggregator merging
                // without cloning below.
                return Ok(());
            }
        }
        // If this integrator is stateful, create a copy of the Aggregator for
        // long-term storage.  Otherwise the Meter implementation will checkpoint
        // the aggregator again, overwriting the long-lived state.

        if self.parent.stateful {
            // Note: the call to AggregatorFor() followed by Merge
            // is effectively a Clone() operation.
            new_agg = self.parent.aggregation_selector().aggregator_for(desc);
            if let Some(new_agg) = new_agg.as_ref() {
                new_agg.merge(agg.as_ref(), desc)?;
            }
        }

        self.batch.values.insert(
            key,
            BatchValue {
                // FIXME consider perf of all this
                aggregator: new_agg.unwrap_or_else(|| agg.clone()),
                descriptor: desc.clone(),
                labels: accumulation.labels().clone(),
                resource: accumulation.resource().clone(),
            },
        );

        Ok(())
    }

    fn checkpoint_set(&mut self) -> &mut dyn CheckpointSet {
        &mut *self.batch
    }

    fn start_collection(&mut self) {
        if self.batch.started_collection != 0 {
            self.batch.interval_start = self.batch.interval_end;
        }
        self.batch.started_collection = self.batch.started_collection.wrapping_add(1);
        if !self.parent.stateful {
            self.batch.values.clear();
        }
    }

    fn finished_collection(&mut self) -> Result<()> {
        self.batch.finished_collection = self.batch.finished_collection.wrapping_add(1);
        self.batch.interval_end = SystemTime::now();

        if self.batch.started_collection != self.batch.finished_collection {
            return Err(MetricsError::InconsistentState);
        }

        Ok(())
    }
}

#[derive(Debug)]
struct SimpleIntegratorBatch {
    values: HashMap<BatchKey, BatchValue>,
    interval_start: SystemTime,
    interval_end: SystemTime,
    started_collection: u64,
    finished_collection: u64,
}

impl Default for SimpleIntegratorBatch {
    fn default() -> Self {
        SimpleIntegratorBatch {
            values: HashMap::default(),
            interval_start: SystemTime::now(),
            interval_end: SystemTime::now(),
            started_collection: 0,
            finished_collection: 0,
        }
    }
}

impl CheckpointSet for SimpleIntegratorBatch {
    fn try_for_each(&mut self, f: &mut dyn FnMut(&Record) -> Result<()>) -> Result<()> {
        self.values.iter().try_for_each(|(_key, value)| {
            f(&metrics::record(
                &value.descriptor,
                &value.labels,
                &value.resource,
                &value.aggregator,
                self.interval_start,
                self.interval_end,
            ))
        })
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct BatchKey(u64);

#[derive(Debug)]
struct BatchValue {
    aggregator: Arc<dyn Aggregator + Send + Sync>,
    descriptor: Descriptor,
    labels: labels::Set,
    resource: Resource,
}
