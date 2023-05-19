use crate::{attributes::AttributeSet, metrics::data::Aggregation};
use lazy_static::lazy_static;
use opentelemetry_api::KeyValue;
use std::sync::Arc;

const STREAM_CARDINALITY_LIMIT: u32 = 2000;
lazy_static! {
    pub static ref STREAM_OVERFLOW_ATTRIBUTE_SET: AttributeSet = {
        let key_values: [KeyValue; 1] = [KeyValue::new("otel.metric.overflow", "true")];
        AttributeSet::from(&key_values[..])
    };
}

/// Forms an aggregation from a collection of recorded measurements.
pub(crate) trait Aggregator<T>: Send + Sync {
    /// Records the measurement, scoped by attr, and aggregates it into an aggregation.
    fn aggregate(&self, measurement: T, attrs: AttributeSet);

    /// Returns an Aggregation, for all the aggregated measurements made and ends an aggregation
    /// cycle.
    fn aggregation(&self) -> Option<Box<dyn Aggregation>>;

    /// Used when filtering aggregators
    fn as_precompute_aggregator(&self) -> Option<Arc<dyn PrecomputeAggregator<T>>> {
        None
    }

    /// Checks whether aggregator has hit cardinality limit for metric streams
    fn check_stream_cardinality(&self, size: usize) -> bool {
        size < STREAM_CARDINALITY_LIMIT as usize - 1
    }
}

/// An `Aggregator` that receives values to aggregate that have been pre-computed by the caller.
pub(crate) trait PrecomputeAggregator<T>: Aggregator<T> {
    /// Records measurements scoped by attributes that have been filtered by an
    /// attribute filter.
    ///
    /// Pre-computed measurements of filtered attributes need to be recorded separate
    /// from those that haven't been filtered so they can be added to the non-filtered
    /// pre-computed measurements in a collection cycle and then resets after the
    /// cycle (the non-filtered pre-computed measurements are not reset).
    fn aggregate_filtered(&self, measurement: T, attrs: AttributeSet);
}
