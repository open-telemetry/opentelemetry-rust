use crate::exporter::trace;
use std::any;
use std::sync::Arc;

#[derive(Debug)]
/// Exporter
pub struct Exporter {}

#[derive(Debug)]
/// Builder
pub struct Builder {}

impl Default for Builder {
    /// Return the default Exporter Builder.
    fn default() -> Self {
        Builder {}
    }
}

impl Builder {
    /// Create a new exporter from the builder
    pub fn init(self) -> Exporter {
        Exporter {}
    }
}

impl Exporter {
    /// Create a new exporter builder.
    pub fn builder() -> Builder {
        Builder::default()
    }
}

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
