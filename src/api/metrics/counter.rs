//! # Metrics Counter Interface
//!
//! Counters support `add(value, label_set)`. Choose this kind of metric when
//! the value is a quantity, the sum is of primary interest, and
//! the event count and value distribution are not of primary interest.
//!
//! `Counter`s are defined as `monotonic = true` by default, meaning
//! that positive values are expected. `monotonic = true` counters are
//! typically used because they can automatically be interpreted as a rate.
//!
//! When passing `MetricOptions`, counters can be declared as `with_monotonic(false)`,
//! in which case they support positive and negative increments.
//! `monotonic = false` counters are useful to report changes in an
//! accounting scheme, such as the number of bytes allocated and
//! deallocated.
use crate::api::metrics;

/// An interface for recording values where the sum is of primary interest.
pub trait Counter<T, LS>: metrics::Instrument<LS>
where
    T: Into<metrics::value::MeasurementValue>,
    LS: metrics::LabelSet,
{
    /// The handle type for the implementing `Counter`.
    type Handle: CounterHandle<T>;

    /// Creates a `Measurement` object to be used by a `Meter` when batch recording.
    fn measurement(&self, value: T) -> metrics::Measurement<LS>;

    /// Creates a handle for this counter. The labels should contain the
    /// keys and values for each key specified in the `LabelSet`.
    ///
    /// If the labels do not contain a value for the key specified in the
    /// `LabelSet`, then the missing value will be treated as unspecified.
    fn acquire_handle(&self, labels: &LS) -> Self::Handle;

    /// Adds the value to the `Counter`'s sum.
    fn add(&self, value: T, label_set: &LS) {
        self.record_one(value.into(), label_set)
    }
}

/// `CounterHandle` is a handle for `Counter` instances.
///
/// It allows for repeated `add` calls for a pre-determined `LabelSet`.
pub trait CounterHandle<T>: metrics::InstrumentHandle
where
    T: Into<metrics::value::MeasurementValue>,
{
    /// Add works by calling the underlying `record_one` method
    /// available because this trait also implements `InstrumentHandle`.
    fn add(&self, value: T) {
        self.record_one(value.into())
    }
}
