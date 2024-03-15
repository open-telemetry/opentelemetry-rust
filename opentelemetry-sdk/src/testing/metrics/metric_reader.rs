use std::sync::{Arc, Mutex, Weak};

use crate::metrics::{
    aggregation::Aggregation,
    data::{ResourceMetrics, Temporality},
    instrument::InstrumentKind,
    pipeline::Pipeline,
    reader::{AggregationSelector, MetricReader, TemporalitySelector},
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

impl MetricReader for TestMetricReader {
    fn register_pipeline(&self, _pipeline: Weak<Pipeline>) {}

    fn collect(&self, _rm: &mut ResourceMetrics) -> Result<()> {
        println!("Collect called..");
        Ok(())
    }

    fn force_flush(&self) -> Result<()> {
        Ok(())
    }

    fn shutdown(&self) -> Result<()> {
        println!(
            "Shutdown called.. on {:?} and pointer for shutdown is {:?}",
            self, self.is_shutdown
        );
        let result = self.force_flush();
        {
            let mut is_shutdown = self.is_shutdown.lock().unwrap();
            *is_shutdown = true;
        }
        println!(
            "Shutdown completed.. on {:?} and pointer for shutdown is {:?}",
            self, self.is_shutdown
        );

        result
    }
}

impl AggregationSelector for TestMetricReader {
    fn aggregation(&self, _kind: InstrumentKind) -> Aggregation {
        Aggregation::Drop
    }
}

impl TemporalitySelector for TestMetricReader {
    fn temporality(&self, _kind: InstrumentKind) -> Temporality {
        Temporality::Cumulative
    }
}
