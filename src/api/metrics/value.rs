use std::sync::atomic;

// `MeasurementValue` represents either an integer or a floating point value of a measurement. It
// needs to be accompanied with a value kind or some source that provides a value kind describing
// this measurement value
#[derive(Debug)]
pub struct MeasurementValue(atomic::AtomicU64);

impl MeasurementValue {
    pub fn into_i64(self) -> i64 {
        self.0.into_inner() as i64
    }

    pub fn into_f64(self) -> f64 {
        f64::from_bits(self.0.into_inner())
    }
}

impl From<i64> for MeasurementValue {
    fn from(value: i64) -> Self {
        MeasurementValue(atomic::AtomicU64::new(value as u64))
    }
}

impl From<f64> for MeasurementValue {
    fn from(value: f64) -> Self {
        MeasurementValue(atomic::AtomicU64::new(value.to_bits()))
    }
}
