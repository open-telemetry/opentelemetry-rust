//! # OpenTelemetry Contrib
//!
//! This is a library for extensions that are not part of the core API, but still may be useful for
//! some users.
//!
//! Typically, those include vendor specific propagators.
#![warn(
    future_incompatible,
    missing_debug_implementations,
    missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    rustdoc,
    unreachable_pub,
    unused
)]
#![cfg_attr(test, deny(warnings))]

#[cfg(feature = "datadog")]
pub mod datadog;
