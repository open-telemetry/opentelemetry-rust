use crate::export::metrics::{AggregatorSelector, Processor};
use crate::metrics::selectors::simple::Selector;

#[derive(Debug)]
pub struct NoopProcessor;

impl Processor for NoopProcessor {
    fn aggregation_selector(&self) -> &dyn AggregatorSelector {
        &Selector::Exact
    }
}
