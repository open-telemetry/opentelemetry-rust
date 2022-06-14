//! # Opentelemetry exporter contrib
//!
//! This module provides exporters for third party vendor format or experimental propagators that
//! aren't part of Opentelemetry.
//!
//! Currently, the following exporters are supported:
//!
//! * `jaeger_json`, which allows to export traces into files using jaegers json format
//!
//! This module also provides relative types for those exporters.

#[cfg(feature = "jaeger_json_exporter")]
pub mod jaeger_json;
