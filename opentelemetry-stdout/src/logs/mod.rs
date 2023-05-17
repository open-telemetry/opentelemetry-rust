//! # Stdout Log Exporter
//!
//! The stdout [`LogExporter`] writes debug printed [`LogRecord`]s to its configured
//! [`Write`] instance. By default it will write to [`Stdout`].
//!
//! [`LogExporter`]: opentelemetry_sdk::export::logs::LogExporter
//! [`LogRecord`]: opentelemetry_api::logs::LogRecord
//! [`Write`]: std::io::Write
//! [`Stdout`]: std::io::Stdout
// TODO: Add an example for using this exporter.
mod exporter;
mod transform;

pub use exporter::*;
pub use transform::*;
