//! # OpenTelemetry Metrics Measurement Values
//!
//! All values recorded by `Instrument`s must be formatted as `MeasurementValue`.
//! `Meter`s can record values that impl `Into<MeasurementValue`. The two default
//! types of values recorded are `i64` and `f64`.
use std::sync::atomic;

/// `MeasurementValue` represents either an integer or a floating point value of a measurement. It
/// needs to be accompanied with a value kind or some source that provides a value kind describing
/// this measurement value.
#[derive(Debug)]
pub struct MeasurementValue(atomic::AtomicU64);

impl MeasurementValue {
    /// Convert the underlying `AtomicU64` into a standard `i64`.
    pub fn into_i64(self) -> i64 {
        self.0.into_inner() as i64
    }

    /// Convert the underlying `AtomicU64` into a standard `f64`.
    pub fn into_f64(self) -> f64 {
        f64::from_bits(self.0.into_inner())
    }
}

impl From<i64> for MeasurementValue {
    /// Convert `i64` instances to `MeasurementValue` instances for use by
    /// `Instrument`s.
    fn from(value: i64) -> Self {
        MeasurementValue(atomic::AtomicU64::new(value as u64))
    }
}

impl From<f64> for MeasurementValue {
    /// Convert `f64` instances to `MeasurementValue` instances for use by
    /// `Instrument`s.
    fn from(value: f64) -> Self {
        MeasurementValue(atomic::AtomicU64::new(value.to_bits()))
    }
}
