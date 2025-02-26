use std::sync::{Arc, Mutex, Weak};

use crate::error::{OTelSdkError, OTelSdkResult};
use crate::metrics::{
    data::ResourceMetrics, pipeline::Pipeline, reader::MetricReader, InstrumentKind,
};
use crate::metrics::{MetricResult, Temporality};

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

    fn collect(&self, _rm: &mut ResourceMetrics) -> MetricResult<()> {
        Ok(())
    }

    fn force_flush(&self) -> OTelSdkResult {
        Ok(())
    }

    fn shutdown(&self) -> OTelSdkResult {
        let result = self.force_flush();
        {
            let mut is_shutdown = self.is_shutdown.lock().unwrap();
            *is_shutdown = true;
        }
        result.map_err(|e| OTelSdkError::InternalFailure(e.to_string()))
    }

    fn temporality(&self, _kind: InstrumentKind) -> Temporality {
        Temporality::default()
    }
}
