//! # OpenTelemetry Contrib
//!
//! This is a library for extensions that are not part of the core API, but still may be useful for
//! some users.
//!
//! Typically, those include vendor specific propagators.
#[cfg(feature = "datadog")]
pub mod datadog;
