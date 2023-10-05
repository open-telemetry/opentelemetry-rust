use crate::transform::transform_resource_metrics;
use async_trait::async_trait;
use opentelemetry::metrics::{MetricsError, Result};
use opentelemetry_sdk::metrics::reader::DefaultAggregationSelector;
use opentelemetry_sdk::metrics::{
    data::{ResourceMetrics, Temporality},
    exporter::PushMetricsExporter,
    reader::{AggregationSelector, TemporalitySelector},
    Aggregation, InstrumentKind,
};

use crate::tracepoint;
use eventheader::_internal as ehi;
use prost::Message;
use std::fmt::{Debug, Formatter};
use std::pin::Pin;

pub struct MetricsExporter {
    trace_point: Pin<Box<ehi::TracepointState>>,
}

impl MetricsExporter {
    pub fn new() -> MetricsExporter {
        let trace_point = Box::pin(ehi::TracepointState::new(0));
        // This is unsafe because if the code is used in a shared object,
        // the event MUST be unregistered before the shared object unloads.
        unsafe {
            let _result = tracepoint::register(trace_point.as_ref());
        }
        MetricsExporter { trace_point }
    }
}

impl Default for MetricsExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl TemporalitySelector for MetricsExporter {
    fn temporality(&self, kind: InstrumentKind) -> Temporality {
        // TODO: Implement temporality selection feature
        match kind {
            InstrumentKind::Counter
            | InstrumentKind::ObservableCounter
            | InstrumentKind::ObservableGauge
            | InstrumentKind::Histogram => Temporality::Delta,
            InstrumentKind::UpDownCounter | InstrumentKind::ObservableUpDownCounter => {
                Temporality::Cumulative
            }
        }
    }
}

impl AggregationSelector for MetricsExporter {
    fn aggregation(&self, _kind: InstrumentKind) -> Aggregation {
        // TODO: Implement aggregation selection feature
        DefaultAggregationSelector::default().aggregation(_kind)
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
        if self.trace_point.enabled() {
            let proto_message = transform_resource_metrics(metrics);

            let mut byte_array = Vec::new();
            let _encode_result = proto_message
                .encode(&mut byte_array)
                .map_err(|err| MetricsError::Other(err.to_string()))?;
            let _result = tracepoint::write(&self.trace_point, byte_array.as_slice());
        }
        Ok(())
    }

    async fn force_flush(&self) -> Result<()> {
        Ok(()) // In this implementation, flush does nothing
    }

    fn shutdown(&self) -> Result<()> {
        // TracepointState automatically unregisters when dropped
        // https://github.com/microsoft/LinuxTracepoints-Rust/blob/main/eventheader/src/native.rs#L618
        Ok(())
    }
}
