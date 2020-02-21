use {
    opentelemetry::exporter::trace::{ExportResult, SpanData, SpanExporter},
    std::{any::Any, sync::Arc},
};

/// Exports opentelemetry tracing spans to Google StackDriver.
#[derive(Debug)]
pub struct StackDriverExporter {}

impl SpanExporter for StackDriverExporter {
    fn export(&self, batch: Vec<Arc<SpanData>>) -> ExportResult {
        ExportResult::Success
    }

    fn shutdown(&self) {}

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
