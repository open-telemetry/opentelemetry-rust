use std::sync::Arc;

use opentelemetry_api::metrics::Result;

use crate::{
    export::metrics::{AggregatorSelector, Checkpointer, LockedCheckpointer, Processor},
    metrics::{aggregators::Aggregator, sdk_api::Descriptor},
};

#[derive(Debug)]
struct NoopAggregatorSelector;

impl AggregatorSelector for NoopAggregatorSelector {
    fn aggregator_for(
        &self,
        _descriptor: &Descriptor,
    ) -> Option<Arc<dyn Aggregator + Send + Sync>> {
        None
    }
}

#[derive(Debug)]
pub struct NoopCheckpointer;

impl Processor for NoopCheckpointer {
    fn aggregator_selector(&self) -> &dyn AggregatorSelector {
        &NoopAggregatorSelector
    }
}

impl Checkpointer for NoopCheckpointer {
    fn checkpoint(
        &self,
        _f: &mut dyn FnMut(&mut dyn LockedCheckpointer) -> Result<()>,
    ) -> Result<()> {
        Ok(())
    }
}
