//! Metrics aggregation
use std::time::SystemTime;

use crate::metrics::sdk_api::Number;
use opentelemetry_api::metrics::Result;

mod temporality;

pub use temporality::*;

/// An interface returned by an [`Aggregator`] containing an interval of metric
/// data.
///
/// [`Aggregator`]: crate::metrics::aggregators::Aggregator
pub trait Aggregation {
    /// A short identifying string to identify the [`Aggregator`] that was used to
    /// produce the aggregation (e.g., [`AggregationKind::SUM`]).
    ///
    /// [`Aggregator`]: crate::metrics::aggregators::Aggregator
    /// [`AggregationKind`]: crate::export::metrics::aggregation::AggregationKind
    fn kind(&self) -> &AggregationKind;
}

/// Sum returns an aggregated sum.
pub trait Sum: Aggregation {
    /// The sum of the currently aggregated metrics
    fn sum(&self) -> Result<Number>;
}

/// Count returns the number of values that were aggregated.
pub trait Count: Aggregation {
    /// The count of the currently aggregated metrics
    fn count(&self) -> Result<u64>;
}

/// LastValue returns the latest value that was aggregated.
pub trait LastValue: Aggregation {
    /// The last value of the currently aggregated metrics
    fn last_value(&self) -> Result<(Number, SystemTime)>;
}

/// Buckets represent histogram buckets boundaries and counts.
///
/// For a Histogram with N defined boundaries, e.g, [x, y, z]. There are N+1
/// counts: [-inf, x), [x, y), [y, z), [z, +inf]
#[derive(Debug)]
pub struct Buckets {
    /// Boundaries are floating point numbers, even when
    /// aggregating integers.
    boundaries: Vec<f64>,

    /// Counts are floating point numbers to account for
    /// the possibility of sampling which allows for
    /// non-integer count values.
    counts: Vec<f64>,
}

impl Buckets {
    /// Create new buckets
    pub fn new(boundaries: Vec<f64>, counts: Vec<f64>) -> Self {
        Buckets { boundaries, counts }
    }

    /// Boundaries of the histogram buckets
    pub fn boundaries(&self) -> &Vec<f64> {
        &self.boundaries
    }

    /// Counts of the histogram buckets
    pub fn counts(&self) -> &Vec<f64> {
        &self.counts
    }
}

/// Histogram returns the count of events in pre-determined buckets.
pub trait Histogram: Sum + Count + Aggregation {
    /// Buckets for this histogram.
    fn histogram(&self) -> Result<Buckets>;
}

/// A short name for the [`Aggregator`] that produces an [`Aggregation`].
///
/// Kind is a string to allow user-defined Aggregators.
///
/// When deciding how to handle an Aggregation, Exporters are encouraged to
/// decide based on conversion to the above interfaces based on strength, not on
/// Kind value, when deciding how to expose metric data.  This enables
/// user-supplied Aggregators to replace builtin Aggregators.
///
/// For example, test for a Histogram before testing for a Sum, and so on.
///
/// [`Aggregator`]: crate::metrics::aggregators::Aggregator
#[derive(Debug, Clone, PartialEq)]
pub struct AggregationKind(&'static str);

impl AggregationKind {
    /// Aggregations that return an aggregated sum.
    pub const SUM: Self = AggregationKind("SUM");

    /// Aggregations that return a distribution
    pub const HISTOGRAM: Self = AggregationKind("HISTOGRAM");

    /// Aggregations that return only the latest value.
    pub const LAST_VALUE: AggregationKind = AggregationKind("LAST_VALUE");

    /// Create a new custom aggregation kind
    pub const fn new(name: &'static str) -> Self {
        AggregationKind(name)
    }
}
