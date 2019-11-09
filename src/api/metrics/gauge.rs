//! # Metrics Gauge Interface
//!
//! `Gauge`s support `set(value, label_set)`.  `Gauge` metrics express
//! a pre-calculated value that is either `set` by explicit instrumentation
//! or observed through a callback. Generally, this kind of metric should
//! be used when the metric cannot be expressed as a sum or because the
//! measurement interval is arbitrary. Use this kind of metric when the
//! measurement is not a quantity, and the sum and event count are not of
//! interest.
//!
//! `Gauge`s are defined as `monotonic = false` by default, meaning that new
//! values are permitted to make positive or negative changes to the
//! gauge. There is no restriction on the sign of the input for gauges.
//!
//! As an option, gauges can be declared as `with_monotonic(true)`, in which case
//! successive values are expected to rise monotonically. `monotonic = true`
//! gauges are useful in reporting computed cumulative sums, allowing an
//! application to compute a current value and report it, without
//! remembering the last-reported value in order to report an increment.
use crate::api::metrics;

/// An interface for recording values where the metric cannot be expressed
/// as a sum or because the measurement interval is arbitrary.
pub trait Gauge<T, LS>: metrics::Instrument<LS>
where
    T: Into<metrics::value::MeasurementValue>,
    LS: metrics::LabelSet,
{
    /// The handle type for the implementing `Gauge`.
    type Handle: GaugeHandle<T>;

    /// Creates a `Measurement` object to be used by a `Meter` when batch recording.
    fn measurement(&self, value: T) -> metrics::Measurement<LS>;

    /// Creates a handle for this gauge. The labels should contain the
    /// keys and values for each key specified in the `LabelSet`.
    ///
    /// If the labels do not contain a value for the key specified in the
    /// `LabelSet`, then the missing value will be treated as unspecified.
    fn acquire_handle(&self, labels: &LS) -> Self::Handle;

    /// Assigns the passed value to the value of the gauge. The labels
    /// should contain the keys and values for each key specified in
    /// the `LabelSet`.
    ///
    /// If the labels do not contain a value for the key specified in the
    /// `LabelSet`, then the missing value will be treated as unspecified.
    fn set(&self, value: T, label_set: &LS) {
        self.record_one(value.into(), label_set)
    }
}

/// `GaugeHandle` is a handle for `Gauge` instances.
///
/// It allows for repeated `set` calls for a pre-determined `LabelSet`.
pub trait GaugeHandle<T>: metrics::InstrumentHandle
where
    T: Into<metrics::value::MeasurementValue>,
{
    /// Set works by calling the underlying `record_one` method
    /// available because this trait also implements `InstrumentHandle`.
    fn set(&self, value: T) {
        self.record_one(value.into())
    }
}
