use crate::transform::transform::transform_resource_metrics;
use async_trait::async_trait;

use opentelemetry_api::metrics::{MetricsError, Result};
use opentelemetry_sdk::{
    metrics::{
        data::{ResourceMetrics, Temporality},
        exporter::PushMetricsExporter,
        reader::{
            AggregationSelector,
            TemporalitySelector,
        },
        Aggregation, InstrumentKind,
    },
};

use std::fmt::{Debug, Formatter};
use prost::Message;
use crate::tracepoint;

#[derive(Clone, Copy)]
pub struct MetricsExporter { }

impl MetricsExporter {

    pub fn new() -> MetricsExporter {
        // This is unsafe because if the code is used in a shared object (DLL),
        // the event MUST be unregistered before the shared object unloads.
        unsafe {
            let result = tracepoint::register();
            if result != 0 {
                println!("Tracepoint failed to register.");
            }
        }
        MetricsExporter {  }
    }
}

impl TemporalitySelector for MetricsExporter {
    fn temporality(&self, _kind: InstrumentKind) -> Temporality {
        Temporality::Cumulative
    }
}

impl AggregationSelector for MetricsExporter {
    fn aggregation(&self, _kind: InstrumentKind) -> Aggregation {
        Aggregation::Sum
    }
}

impl Debug for MetricsExporter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("user_events metrics exporter")
    }
}

#[async_trait]
impl PushMetricsExporter for MetricsExporter {
    async fn export(&self, metrics: &mut ResourceMetrics) -> Result<()> {
        if tracepoint::enabled() {
            let proto_message = transform_resource_metrics(metrics);
            
            let mut byte_array = Vec::new();
            let encode_result = proto_message.encode(&mut byte_array).map_err(|err| MetricsError::Other(err.to_string()));
            if let Err(error) = encode_result {
                return Err(error);
            }
            let result = tracepoint::write(byte_array.as_slice());
            if result != 0 {
                return Err(MetricsError::Other("Tracepoint failed to write.".into()));
            }
        }
        Ok(())
    }

    async fn force_flush(&self) -> Result<()> {
        Ok(()) // In this implementation, flush does nothing
    }

    async fn shutdown(&self) -> Result<()> {
        let result = tracepoint::unregister();
        if result != 0 {
            println!("Tracepoint failed to unregister.");
        }
        Ok(())
    }
}