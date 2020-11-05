//! # Opentelemetry exporter contrib
//!
//! This module provides exporter implementations from third party vendors(like datadog). Note that
//! there are some exporters live in their own repo. See [`Opentelemetry`] for a list of those
//! implementations and where to find them.
//!
//! Currently, the following exporters are supported:
//! * `datadog`, enable `datadog` feature to use it.
//!
//! [`Opentelemetry`](https://github.com/open-telemetry/opentelemetry-rust#related-crates)
#[cfg(feature = "datadog")]
#[cfg_attr(docsrs, doc(cfg(feature = "datadog")))]
pub mod datadog;
