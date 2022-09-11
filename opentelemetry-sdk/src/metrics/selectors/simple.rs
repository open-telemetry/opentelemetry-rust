//! Simple Metric Selectors
use crate::export::metrics::AggregatorSelector;
use crate::metrics::aggregators::{self, Aggregator};
use crate::metrics::sdk_api::{Descriptor, InstrumentKind};
use std::sync::Arc;

/// This selector is faster and uses less memory than the others in this package.
pub fn inexpensive() -> impl AggregatorSelector {
    InexpensiveSelector
}

#[derive(Debug, Clone)]
struct InexpensiveSelector;

impl AggregatorSelector for InexpensiveSelector {
    fn aggregator_for(&self, descriptor: &Descriptor) -> Option<Arc<dyn Aggregator + Send + Sync>> {
        match descriptor.instrument_kind() {
            InstrumentKind::GaugeObserver => Some(Arc::new(aggregators::last_value())),
            _ => Some(Arc::new(aggregators::sum())),
        }
    }
}

/// A simple aggregator selector that uses histogram aggregators for `Histogram`
/// instruments.
///
/// This selector is a good default choice for most metric exporters.
pub fn histogram(boundaries: impl Into<Vec<f64>>) -> impl AggregatorSelector {
    HistogramSelector(boundaries.into())
}

#[derive(Debug, Clone)]
struct HistogramSelector(Vec<f64>);

impl AggregatorSelector for HistogramSelector {
    fn aggregator_for(&self, descriptor: &Descriptor) -> Option<Arc<dyn Aggregator + Send + Sync>> {
        match descriptor.instrument_kind() {
            InstrumentKind::GaugeObserver => Some(Arc::new(aggregators::last_value())),
            InstrumentKind::Histogram => Some(Arc::new(aggregators::histogram(&self.0))),
            _ => Some(Arc::new(aggregators::sum())),
        }
    }
}
