//! OTEL metric exporter
//!
//! Defines a [Exporter] to send metric data to backend via OTEL protocol.

#[cfg(feature = "tonic")]
use crate::proto::collector::metrics::v1::metrics_service_client::MetricsServiceClient;
use crate::transform::{record_to_metric, sink, CheckpointedMetrics};
use opentelemetry::metrics::{Descriptor, Result};
use opentelemetry::sdk::{
    export::metrics::{CheckpointSet, ExportKind, ExportKindFor, Exporter},
    InstrumentationLibrary,
};
use std::fmt::{Debug, Formatter};
#[cfg(feature = "tonic")]
use tonic::transport::Channel;
use tonic::Request;

/// Export metrics in OTEL format.
pub struct MetricsExporter {
    #[cfg(feature = "tonic")]
    exporter: MetricsServiceClient<Channel>,

    #[cfg(feature = "tokio")]
    runtime: tokio::runtime::Runtime,

    export_kind_selector: Box<dyn ExportKindFor>,
}

impl Debug for MetricsExporter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "tonic")]
        f.debug_struct("OTLP Metric Exporter")
            .field("grpc_client", &"tonic")
            .finish()
    }
}

impl Default for MetricsExporter {
    fn default() -> Self {
        unimplemented!()
    }
}

impl ExportKindFor for MetricsExporter {
    fn export_kind_for(&self, descriptor: &Descriptor) -> ExportKind {
        self.export_kind_selector.export_kind_for(descriptor)
    }
}

impl Exporter for MetricsExporter {
    fn export(&mut self, checkpoint_set: &mut dyn CheckpointSet) -> Result<()> {
        let mut resource_metrics: Vec<CheckpointedMetrics> = Vec::default();
        // transform the metrics into proto. Append the resource and instrumentation library information into it.
        checkpoint_set.try_for_each(self.export_kind_selector.as_ref(), &mut |record| {
            let metric_result = record_to_metric(record, self.export_kind_selector.as_ref());
            match metric_result {
                Ok(metrics) => {
                    resource_metrics.push((
                        record.resource().clone().into(),
                        InstrumentationLibrary::new(
                            record.descriptor().instrumentation_name(),
                            record.descriptor().instrumentation_version(),
                        ),
                        metrics,
                    ));
                    Ok(())
                }
                Err(err) => Err(err),
            }
        })?;
        let request = Request::new(sink(resource_metrics));

        #[cfg(feature = "tokio")]
            self.runtime
            .block_on(self.exporter.to_owned().export(request))
            .map_err::<crate::Error, _>(Into::into)?;

        Ok(())
    }
}
