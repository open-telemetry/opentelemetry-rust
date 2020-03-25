use crate::exporter::trace;
use std::any;
use std::sync::Arc;

#[derive(Debug)]
/// Builder
pub struct Builder {}

impl Builder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {}
    }

    /// Build a new exporter
    pub fn init() -> Exporter {
        Exporter {}
    }
}

impl Default for Builder {
    /// Return the default Exporter Builder.
    fn default() -> Self {
        Builder {}
    }
}

#[derive(Debug, Default)]
/// Exporter
pub struct Exporter {}

impl trace::SpanExporter for Exporter {
    /// Export spans to stdout
    fn export(&self, batch: Vec<Arc<trace::SpanData>>) -> trace::ExportResult {
        println!("{:?}", batch);

        trace::ExportResult::Success
    }

    /// Ignored for now.
    fn shutdown(&self) {}

    /// Allows `Exporter` to be downcast from trait object.
    fn as_any(&self) -> &dyn any::Any {
        self
    }
}