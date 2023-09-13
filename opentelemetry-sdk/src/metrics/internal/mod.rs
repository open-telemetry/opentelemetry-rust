mod aggregate;
mod histogram;
mod last_value;
mod sum;

use core::fmt;
use std::ops::{Add, AddAssign, Sub};

pub(crate) use aggregate::{AggregateBuilder, ComputeAggregation, Measure};

pub(crate) trait Number<T>:
    Add<Output = T>
    + AddAssign
    + Sub<Output = T>
    + PartialOrd
    + fmt::Debug
    + Clone
    + Copy
    + PartialEq
    + Default
    + Send
    + Sync
    + 'static
{
    fn into_float(self) -> f64;
}

impl Number<i64> for i64 {
    fn into_float(self) -> f64 {
        // May have precision loss at high values
        self as f64
    }
}
impl Number<u64> for u64 {
    fn into_float(self) -> f64 {
        // May have precision loss at high values
        self as f64
    }
}
impl Number<f64> for f64 {
    fn into_float(self) -> f64 {
        self
    }
}
