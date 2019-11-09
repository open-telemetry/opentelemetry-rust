//! # Metrics Measure Interface
//!
//! `Measure`s support `record(value, label_set)`, signifying that
//! events report individual measurements. This kind of metric
//! should be used when the count or rate of events is meaningful
//! and either:
//!
//! - The sum is of interest in addition to the count (rate)
//! - Quantile information is of interest.
//!
//! `Measure`s are defined as `with_absolute(true)` by default,
//! meaning that negative values are invalid. `absolute = true`
//! measures are typically used to record absolute values such as
//! durations and sizes.
//!
//! When passing `MetricOptions`, measures can be declared as
//! `with_abslute(false)` to indicate support for positive and negative values.
use crate::api::metrics;

/// An interface for recording values where the count or rate of
/// events is meaningful.
pub trait Measure<T, LS>: metrics::Instrument<LS>
where
    T: Into<metrics::value::MeasurementValue>,
    LS: metrics::LabelSet,
{
    /// The handle type for the implementing `Measure`.
    type Handle: MeasureHandle<T>;

    /// Creates a `Measurement` object to be used by a `Meter`
    /// when batch recording.
    fn measurement(&self, value: T) -> metrics::Measurement<LS>;

    /// Creates a handle for this measure. The labels should contain the
    /// keys and values for each key specified in the `LabelSet`.
    ///
    /// If the labels do not contain a value for the key specified in the
    /// `LabelSet`, then the missing value will be treated as unspecified.
    fn acquire_handle(&self, labels: &LS) -> Self::Handle;

    /// Records the passed value to the value of the measure. The labels
    /// should contain the keys and values for each key specified in
    /// the `LabelSet`.
    ///
    /// If the labels do not contain a value for the key specified in the
    /// `LabelSet`, then the missing value will be treated as unspecified.
    fn record(&self, value: T, label_set: &LS) {
        self.record_one(value.into(), label_set)
    }
}

/// `MeasureHandle` is a handle for `Measure` instances.
///
/// It allows for repeated `record` calls for a pre-determined `LabelSet`.
pub trait MeasureHandle<T>: metrics::InstrumentHandle
where
    T: Into<metrics::value::MeasurementValue>,
{
    /// Record works by calling the underlying `record_one` method
    /// available because this trait also implements `InstrumentHandle`.
    fn record(&self, value: T) {
        self.record_one(value.into())
    }
}
