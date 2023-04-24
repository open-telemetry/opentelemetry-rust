use std::sync::Arc;

use opentelemetry_api::KeyValue;

use crate::{attributes::AttributeSet, metrics::data::Aggregation};

use super::{aggregator::PrecomputeAggregator, Aggregator, Number};

/// Returns an [Aggregator] that wraps the passed in aggregator with an
/// attribute filtering function.
///
/// Both pre-computed non-pre-computed [Aggregator]s can be passed in. An
/// appropriate [Aggregator] will be returned for the detected type.
pub(crate) fn new_filter<T: Number<T>>(
    agg: Arc<dyn Aggregator<T>>,
    filter: Arc<dyn Fn(&KeyValue) -> bool + Send + Sync>,
) -> Arc<dyn Aggregator<T>> {
    if let Some(agg) = agg.as_precompute_aggregator() {
        Arc::new(PrecomputeFilter { agg, filter })
    } else {
        Arc::new(Filter { agg, filter })
    }
}

/// Wraps an aggregator with an attribute filter.
///
/// All recorded measurements will have their attributes filtered before they
/// are passed to the underlying aggregator's aggregate method.
///
/// This should not be used to wrap a pre-computed aggregator. Use a
/// [PrecomputedFilter] instead.
struct Filter<T> {
    filter: Arc<dyn Fn(&KeyValue) -> bool + Send + Sync>,
    agg: Arc<dyn Aggregator<T>>,
}

impl<T: Number<T>> Aggregator<T> for Filter<T> {
    fn aggregate(&self, measurement: T, mut attrs: AttributeSet) {
        attrs.retain(self.filter.as_ref());
        self.agg.aggregate(measurement, attrs)
    }

    fn aggregation(&self) -> Option<Box<dyn Aggregation>> {
        self.agg.aggregation()
    }
}

/// An aggregator that applies attribute filter when aggregating for
/// pre-computed aggregations.
///
/// The pre-computed aggregations need to operate normally when no attribute
/// filtering is done (for sums this means setting the value), but when
/// attribute filtering is done it needs to be added to any set value.
struct PrecomputeFilter<T: Number<T>> {
    filter: Arc<dyn Fn(&KeyValue) -> bool + Send + Sync>,
    agg: Arc<dyn PrecomputeAggregator<T>>,
}

impl<T: Number<T>> Aggregator<T> for PrecomputeFilter<T> {
    fn aggregate(&self, measurement: T, mut attrs: AttributeSet) {
        let pre_len = attrs.len();
        attrs.retain(self.filter.as_ref());
        if pre_len == attrs.len() {
            // No filtering done.
            self.agg.aggregate(measurement, attrs)
        } else {
            self.agg.aggregate_filtered(measurement, attrs)
        }
    }

    fn aggregation(&self) -> Option<Box<dyn Aggregation>> {
        self.agg.aggregation()
    }
}
