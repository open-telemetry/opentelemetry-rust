//! In-Memory trace exporter for testing purpose.

/// The `in_memory_exporter` module provides in-memory trace functionalities.
/// For detailed usage and examples, see `in_memory_exporter`.
pub mod in_memory_exporter;
pub use in_memory_exporter::{InMemorySpanExporter, InMemorySpanExporterBuilder};

#[doc(hidden)]
mod span_exporters;
pub use span_exporters::*;
