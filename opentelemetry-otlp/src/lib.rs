//! # OpenTelemetry OTLP Exporter
//!
//! The OpenTelemetry OTLP Exporter supports exporting of trace and metric data in the OTLP format.
mod proto;
mod span;
mod transform;

pub use crate::span::{Compression, Credentials, Exporter, ExporterConfig, Protocol};
