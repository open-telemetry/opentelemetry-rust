//! # Stdout Log Exporter
//!
//! The stdout [`LogExporter`] writes debug printed [`LogRecord`]s to Stdout.
//!
//! [`LogExporter`]: opentelemetry_sdk::logs::LogExporter
//! [`LogRecord`]: opentelemetry::logs::LogRecord
mod exporter;
pub use exporter::*;
