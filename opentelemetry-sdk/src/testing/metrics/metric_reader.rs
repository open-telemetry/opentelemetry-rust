use std::sync::{Arc, Mutex, Weak};

use crate::metrics::{
    data::{ResourceMetrics, Temporality},
    pipeline::Pipeline,
    reader::MetricReader,
    InstrumentKind,
};
use opentelemetry::metrics::Result;

#[derive(Debug, Clone)]
pub struct TestMetricReader {
    is_shutdown: Arc<Mutex<bool>>,
}

impl TestMetricReader {
    // Constructor to initialize the TestMetricReader
    pub fn new() -> Self {
        TestMetricReader {
            is_shutdown: Arc::new(Mutex::new(false)),
        }
    }

    // Method to check if the reader is shutdown
    pub fn is_shutdown(&self) -> bool {
        *self.is_shutdown.lock().unwrap()
    }
}

impl Default for TestMetricReader {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricReader for TestMetricReader {
    fn register_pipeline(&self, _pipeline: Weak<Pipeline>) {}

    fn collect(&self, _rm: &mut ResourceMetrics) -> Result<()> {
        Ok(())
    }

    fn force_flush(&self) -> Result<()> {
        Ok(())
    }

    fn shutdown(&self) -> Result<()> {
        let result = self.force_flush();
        {
            let mut is_shutdown = self.is_shutdown.lock().unwrap();
            *is_shutdown = true;
        }
        result
    }

    fn temporality(&self, _kind: InstrumentKind) -> Temporality {
        Temporality::default()
    }
}
