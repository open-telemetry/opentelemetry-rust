//! OpenTelemetry Dynatrace Transform
mod common;

#[cfg(feature = "metrics")]
mod metrics;

#[cfg(feature = "metrics")]
pub use metrics::{DimensionSet, MetricKey, MetricLine};

#[cfg(feature = "metrics")]
pub(crate) use metrics::record_to_metric_line;
