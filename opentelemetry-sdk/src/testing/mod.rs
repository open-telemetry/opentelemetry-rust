#[cfg(all(feature = "testing", feature = "trace"))]
#[doc(hidden)]
pub mod trace;

#[cfg(all(feature = "testing", feature = "metrics"))]
#[doc(hidden)]
pub mod metrics;

#[cfg(all(feature = "testing", feature = "logs"))]
/// TBD-1
pub mod logs;
