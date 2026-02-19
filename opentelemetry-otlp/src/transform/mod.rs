//! Common transformation utilities for converting SDK types to protobuf types.

/// Common transformation utilities.
pub mod common;

#[cfg(feature = "metrics")]
/// Metrics transformation utilities.
pub mod metrics;

#[cfg(feature = "trace")]
/// Trace transformation utilities.
pub mod trace;

#[cfg(feature = "logs")]
/// Logs transformation utilities.
pub mod logs;

#[cfg(feature = "zpages")]
/// Tracez transformation utilities for converting span data to tracez data types.
pub mod tracez;

#[cfg(feature = "profiles")]
/// Profiles transformation utilities for converting profiling data to protobuf types.
pub mod profiles;
