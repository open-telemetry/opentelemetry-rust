use opentelemetry::exporter::trace::{SpanExporter, SpanData, ExportResult};
use std::sync::Arc;

mod proto;

#[derive(Debug)]
pub struct Exporter {

}

impl SpanExporter for Exporter {
    fn export(&self, batch: Vec<Arc<SpanData>>) -> ExportResult {
        unimplemented!()
    }

    fn shutdown(&self) {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
