use crate::api::{
    labels,
    metrics::{Descriptor, Result},
};
use crate::sdk::{
    export::metrics::{
        self, AggregationSelector, Aggregator, CheckpointSet, Integrator, LockedIntegrator, Record,
    },
    Resource,
};
// use dashmap::DashMap;
use std::collections::{hash_map::DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, MutexGuard};

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
    fn process(&mut self, record: Record) -> Result<()> {
        let desc = record.descriptor();
        let mut hasher = DefaultHasher::new();
        desc.hash(&mut hasher);
        record.labels().equivalent().hash(&mut hasher);
        // FIXME: convert resource to use labels::Set
        // record.resource().equivalent().hash(&mut hasher);
        let key = BatchKey(hasher.finish());
        let agg = record.aggregator();
        let mut new_agg = None;
        if let Some(value) = self.batch.0.get(&key) {
            // Note: The call to Merge here combines only
            // identical records.  It is required even for a
            // stateless Integrator because such identical records
            // may arise in the Meter implementation due to race
            // conditions.
            if self.parent.stateful {
                return value.aggregator.merge(agg.as_ref(), desc);
            } else {
                // FIXME: consider deadlock case for stateless aggregator merging
                // without cloning below.
                return Ok(());
            }
        }
        // If this integrator is stateful, create a copy of the
        // Aggregator for long-term storage.  Otherwise the
        // Meter implementation will checkpoint the aggregator
        // again, overwriting the long-lived state.

        if self.parent.stateful {
            // Note: the call to AggregatorFor() followed by Merge
            // is effectively a Clone() operation.
            new_agg = self.parent.aggregation_selector().aggregator_for(desc);
            if let Some(new_agg) = new_agg.as_ref() {
                new_agg.merge(agg.as_ref(), desc)?;
            }
        }

        self.batch.0.insert(
            key,
            BatchValue {
                // FIXME consider perf of all this
                aggregator: new_agg.unwrap_or_else(|| agg.clone()),
                descriptor: desc.clone(),
                labels: record.labels().clone(),
                resource: record.resource().clone(),
            },
        );

        Ok(())
    }

    fn checkpoint_set(&mut self) -> &mut dyn CheckpointSet {
        &mut *self.batch
    }

    fn finished_collection(&mut self) {
        if !self.parent.stateful {
            self.batch.0.clear();
        }
    }
}

#[derive(Debug, Default)]
struct SimpleIntegratorBatch(HashMap<BatchKey, BatchValue>);

impl CheckpointSet for SimpleIntegratorBatch {
    fn try_for_each(&mut self, f: &mut dyn FnMut(&Record) -> Result<()>) -> Result<()> {
        self.0.iter().try_for_each(|(_key, value)| {
            f(&metrics::record(
                &value.descriptor,
                &value.labels,
                &value.resource,
                &value.aggregator,
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
