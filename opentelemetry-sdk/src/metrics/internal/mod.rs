mod aggregator;
mod filter;
mod histogram;
mod last_value;
mod sum;

use core::fmt;
use opentelemetry_api::metrics::Result;
use std::ops::{Add, AddAssign, Sub};

pub(crate) use aggregator::Aggregator;
pub(crate) use filter::new_filter;
pub(crate) use histogram::{new_cumulative_histogram, new_delta_histogram};
pub(crate) use last_value::new_last_value;
pub(crate) use sum::{
    new_cumulative_sum, new_delta_sum, new_precomputed_cumulative_sum, new_precomputed_delta_sum,
};

pub(crate) trait Number<T>:
    Add<Output = T>
    + AddAssign
    + Sub<Output = T>
    + fmt::Debug
    + Clone
    + Copy
    + PartialEq
    + Default
    + Send
    + Sync
    + 'static
{
    fn try_into_float(&self) -> Result<f64>;
}

impl Number<i64> for i64 {
    fn try_into_float(&self) -> Result<f64> {
        // May have precision loss at high values
        Ok(*self as f64)
    }
}
impl Number<u64> for u64 {
    fn try_into_float(&self) -> Result<f64> {
        // May have precision loss at high values
        Ok(*self as f64)
    }
}
impl Number<f64> for f64 {
    fn try_into_float(&self) -> Result<f64> {
        Ok(*self)
    }
}
