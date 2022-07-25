//! Metric Aggregators
use core::fmt;
use std::{any::Any, sync::Arc};

use crate::{
    export::metrics::aggregation::Aggregation,
    metrics::sdk_api::{Descriptor, InstrumentKind, Number, NumberKind},
};
use opentelemetry_api::{
    metrics::{MetricsError, Result},
    Context,
};

mod histogram;
mod last_value;
mod sum;

pub use histogram::{histogram, HistogramAggregator};
pub use last_value::{last_value, LastValueAggregator};
pub use sum::{sum, SumAggregator};

/// RangeTest is a common routine for testing for valid input values. This
/// rejects NaN values. This rejects negative values when the metric instrument
/// does not support negative values, including monotonic counter metrics and
/// absolute Histogram metrics.
pub fn range_test(number: &Number, descriptor: &Descriptor) -> Result<()> {
    if descriptor.number_kind() == &NumberKind::F64 && number.is_nan() {
        return Err(MetricsError::NaNInput);
    }

    match descriptor.instrument_kind() {
        InstrumentKind::Counter | InstrumentKind::CounterObserver
            if descriptor.number_kind() == &NumberKind::F64 =>
        {
            if number.is_negative(descriptor.number_kind()) {
                return Err(MetricsError::NegativeInput);
            }
        }
        _ => (),
    };
    Ok(())
}

/// Aggregator implements a specific aggregation behavior, i.e., a behavior to
/// track a sequence of updates to an instrument. Sum-only instruments commonly
/// use a simple Sum aggregator, but for the distribution instruments
/// (Histogram, ValueObserver) there are a number of possible aggregators
/// with different cost and accuracy tradeoffs.
///
/// Note that any Aggregator may be attached to any instrument--this is the
/// result of the OpenTelemetry API/SDK separation. It is possible to attach a
/// Sum aggregator to a Histogram instrument or a MinMaxSumCount aggregator
/// to a Counter instrument.
pub trait Aggregator: fmt::Debug {
    /// The interface to access the current state of this Aggregator.
    fn aggregation(&self) -> &dyn Aggregation;

    /// Update receives a new measured value and incorporates it into the
    /// aggregation. Update calls may be called concurrently.
    ///
    /// `Descriptor::number_kind` should be consulted to determine whether the
    /// provided number is an `i64`, `u64` or `f64`.
    ///
    /// The current Context could be inspected for a `Baggage` or
    /// `SpanContext`.
    fn update(&self, context: &Context, number: &Number, descriptor: &Descriptor) -> Result<()>;

    /// This method is called during collection to finish one period of aggregation
    /// by atomically saving the currently-updating state into the argument
    /// Aggregator.
    ///
    /// `synchronized_move` is called concurrently with `update`. These two methods
    /// must be synchronized with respect to each other, for correctness.
    ///
    /// This method will return an `InconsistentAggregator` error if this
    /// `Aggregator` cannot be copied into the destination due to an incompatible
    /// type.
    ///
    /// This call has no `Context` argument because it is expected to perform only
    /// computation.
    fn synchronized_move(
        &self,
        destination: &Arc<dyn Aggregator + Send + Sync>,
        descriptor: &Descriptor,
    ) -> Result<()>;

    /// This combines the checkpointed state from the argument `Aggregator` into this
    /// `Aggregator`. `merge` is not synchronized with respect to `update` or
    /// `synchronized_move`.
    ///
    /// The owner of an `Aggregator` being merged is responsible for synchronization
    /// of both `Aggregator` states.
    fn merge(&self, other: &(dyn Aggregator + Send + Sync), descriptor: &Descriptor) -> Result<()>;

    /// Returns the implementing aggregator as `Any` for downcasting.
    fn as_any(&self) -> &dyn Any;
}
