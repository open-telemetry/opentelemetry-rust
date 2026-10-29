#![cfg_attr(docsrs, feature(doc_cfg))]
//! Experimental conversion of OpenTelemetry SDK data into OTLP protobuf export
//! requests and zPages trace data.
//!
//! This crate contains the protocol-specific transformation layer between
//! [`opentelemetry_sdk`] data and the generated types in [`opentelemetry_proto`].
//! It does not provide an exporter or transport. Its public API may change before
//! stabilisation.

#[cfg(any(feature = "trace", feature = "metrics", feature = "logs"))]
mod common;

/// OTLP-encoded resource attributes and their schema URL.
///
/// Prepare this value once from an [`opentelemetry_sdk::Resource`] and reuse it
/// across trace and log export requests.
#[cfg(any(feature = "trace", feature = "logs"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "trace", feature = "logs"))))]
pub use common::tonic::ResourceAttributesWithSchema;

/// Trace export request conversion.
#[cfg(feature = "trace")]
#[cfg_attr(docsrs, doc(cfg(feature = "trace")))]
pub mod trace;

/// Metrics export request conversion.
#[cfg(feature = "metrics")]
#[cfg_attr(docsrs, doc(cfg(feature = "metrics")))]
pub mod metrics;

/// Log export request conversion.
#[cfg(feature = "logs")]
#[cfg_attr(docsrs, doc(cfg(feature = "logs")))]
pub mod logs;

/// zPages trace data conversion.
#[cfg(feature = "zpages")]
#[cfg_attr(docsrs, doc(cfg(feature = "zpages")))]
pub mod tracez;
