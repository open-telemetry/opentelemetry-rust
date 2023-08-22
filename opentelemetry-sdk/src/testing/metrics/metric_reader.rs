use std::sync::Weak;

use opentelemetry_api::{metrics::Result, Context};
use crate::metrics::{
    aggregation::Aggregation,
    data::{ResourceMetrics, Temporality},
    instrument::InstrumentKind,
    reader::{AggregationSelector, MetricProducer, MetricReader, TemporalitySelector},
    pipeline::Pipeline,
};


#[derive(Debug)]
pub struct TestMetricReader {}

impl MetricReader for TestMetricReader {
    fn register_pipeline(&self, _pipeline: Weak<Pipeline>) {
    }

    fn register_producer(&self, _producer: Box<dyn MetricProducer>) {
    }

    fn collect(&self, _rm: &mut ResourceMetrics) -> Result<()> {
        Ok(())
    }

    fn force_flush(&self, _cx: &Context) -> Result<()> {
        Ok(())
    }

    fn shutdown(&self) -> Result<()> {
        let cx = Context::new();
        self.force_flush(&cx)
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