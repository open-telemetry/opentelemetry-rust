//! OTEL metric exporter
//!
//! Defines a [Exporter] to send metric data to backend via OTEL protocol.
//!
//! Currently, OTEL metrics exporter only support GRPC connection via tonic on tokio runtime.

use crate::exporter::{
    tonic::{TonicConfig, TonicExporterBuilder},
    ExportConfig,
};
#[cfg(feature = "tonic")]
use crate::proto::collector::metrics::v1::{
    metrics_service_client::MetricsServiceClient, ExportMetricsServiceRequest,
};
use crate::transform::{record_to_metric, sink, CheckpointedMetrics};
use crate::{Error, OtlpPipeline};
use futures_util::Stream;
use opentelemetry::metrics::{Descriptor, Result};
use opentelemetry::sdk::export::metrics::{AggregatorSelector, ExportKindSelector};
use opentelemetry::sdk::metrics::{PushController, PushControllerWorker};
use opentelemetry::sdk::{
    export::metrics::{CheckpointSet, ExportKind, ExportKindFor, Exporter},
    metrics::selectors,
    InstrumentationLibrary, Resource,
};
use opentelemetry::{global, KeyValue};
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use std::sync::Mutex;
use std::time;
use tonic::metadata::KeyAndValueRef;
#[cfg(feature = "tonic")]
use tonic::transport::Channel;
#[cfg(feature = "tonic")]
use tonic::Request;

impl OtlpPipeline {
    /// Create a OTLP metrics pipeline.
    pub fn metrics<SP, SO, I, IO>(
        self,
        spawn: SP,
        interval: I,
    ) -> OtlpMetricPipeline<selectors::simple::Selector, ExportKindSelector, SP, SO, I, IO>
    where
        SP: Fn(PushControllerWorker) -> SO,
        I: Fn(time::Duration) -> IO,
    {
        OtlpMetricPipeline {
            aggregator_selector: selectors::simple::Selector::Inexpensive,
            export_selector: ExportKindSelector::Cumulative,
            spawn,
            interval,
            exporter_pipeline: None,
            resource: None,
            period: None,
            timeout: None,
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum MetricsExporterBuilder {
    #[cfg(feature = "tonic")]
    Tonic(TonicExporterBuilder),
}

impl MetricsExporterBuilder {
    /// Build a OTLP metrics exporter with given configuration.
    fn build_metrics_exporter<ES>(self, export_selector: ES) -> Result<MetricsExporter>
    where
        ES: ExportKindFor + Sync + Send + 'static,
    {
        match self {
            #[cfg(feature = "tonic")]
            MetricsExporterBuilder::Tonic(builder) => Ok(MetricsExporter::new(
                builder.exporter_config,
                builder.tonic_config,
                export_selector,
            )?),
        }
    }
}

impl From<TonicExporterBuilder> for MetricsExporterBuilder {
    fn from(exporter: TonicExporterBuilder) -> Self {
        MetricsExporterBuilder::Tonic(exporter)
    }
}

/// Pipeline to build OTLP metrics exporter
///
/// Note that currently the OTLP metrics exporter only supports tonic as it's grpc layer and tokio as
/// runtime.
#[derive(Debug)]
pub struct OtlpMetricPipeline<AS, ES, SP, SO, I, IO>
where
    AS: AggregatorSelector + Send + Sync + 'static,
    ES: ExportKindFor + Send + Sync + Clone + 'static,
    SP: Fn(PushControllerWorker) -> SO,
    I: Fn(time::Duration) -> IO,
{
    aggregator_selector: AS,
    export_selector: ES,
    spawn: SP,
    interval: I,
    exporter_pipeline: Option<MetricsExporterBuilder>,
    resource: Option<Resource>,
    period: Option<time::Duration>,
    timeout: Option<time::Duration>,
}

impl<AS, ES, SP, SO, I, IO, IOI> OtlpMetricPipeline<AS, ES, SP, SO, I, IO>
where
    AS: AggregatorSelector + Send + Sync + 'static,
    ES: ExportKindFor + Send + Sync + Clone + 'static,
    SP: Fn(PushControllerWorker) -> SO,
    I: Fn(time::Duration) -> IO,
    IO: Stream<Item = IOI> + Send + 'static,
{
    /// Build with resource key value pairs.
    pub fn with_resource<T: IntoIterator<Item = R>, R: Into<KeyValue>>(self, resource: T) -> Self {
        OtlpMetricPipeline {
            resource: Some(Resource::new(resource.into_iter().map(Into::into))),
            ..self
        }
    }

    /// Build with the exporter
    pub fn with_exporter<B: Into<MetricsExporterBuilder>>(self, pipeline: B) -> Self {
        OtlpMetricPipeline {
            exporter_pipeline: Some(pipeline.into()),
            ..self
        }
    }

    /// Build with the aggregator selector
    pub fn with_aggregator_selector<T>(
        self,
        aggregator_selector: T,
    ) -> OtlpMetricPipeline<T, ES, SP, SO, I, IO>
    where
        T: AggregatorSelector + Send + Sync + 'static,
    {
        OtlpMetricPipeline {
            aggregator_selector,
            export_selector: self.export_selector,
            spawn: self.spawn,
            interval: self.interval,
            exporter_pipeline: self.exporter_pipeline,
            resource: self.resource,
            period: self.period,
            timeout: self.timeout,
        }
    }

    /// Build with spawn function
    pub fn with_spawn(self, spawn: SP) -> Self {
        OtlpMetricPipeline { spawn, ..self }
    }

    /// Build with timeout
    pub fn with_timeout(self, timeout: time::Duration) -> Self {
        OtlpMetricPipeline {
            timeout: Some(timeout),
            ..self
        }
    }

    /// Build with period, your metrics will be exported with this period
    pub fn with_period(self, period: time::Duration) -> Self {
        OtlpMetricPipeline {
            period: Some(period),
            ..self
        }
    }

    /// Build with interval function
    pub fn with_interval(self, interval: I) -> Self {
        OtlpMetricPipeline { interval, ..self }
    }

    /// Build with export kind selector
    pub fn with_export_kind<E>(self, export_selector: E) -> OtlpMetricPipeline<AS, E, SP, SO, I, IO>
    where
        E: ExportKindFor + Send + Sync + Clone + 'static,
    {
        OtlpMetricPipeline {
            aggregator_selector: self.aggregator_selector,
            export_selector,
            spawn: self.spawn,
            interval: self.interval,
            exporter_pipeline: self.exporter_pipeline,
            resource: self.resource,
            period: self.period,
            timeout: self.timeout,
        }
    }

    /// Build push controller.
    pub fn build(self) -> Result<PushController> {
        let exporter = self
            .exporter_pipeline
            .ok_or(Error::NoExporterBuilder)?
            .build_metrics_exporter(self.export_selector.clone())?;

        let mut builder = opentelemetry::sdk::metrics::controllers::push(
            self.aggregator_selector,
            self.export_selector,
            exporter,
            self.spawn,
            self.interval,
        );
        if let Some(period) = self.period {
            builder = builder.with_period(period);
        }
        if let Some(resource) = self.resource {
            builder = builder.with_resource(resource);
        }
        if let Some(timeout) = self.timeout {
            builder = builder.with_timeout(timeout)
        }
        let controller = builder.build();
        global::set_meter_provider(controller.provider());
        Ok(controller)
    }
}

enum ExportMsg {
    #[cfg(feature = "tonic")]
    Export(tonic::Request<ExportMetricsServiceRequest>),
    Shutdown,
}

/// Export metrics in OTEL format.
pub struct MetricsExporter {
    #[cfg(feature = "tokio")]
    sender: Arc<Mutex<tokio::sync::mpsc::Sender<ExportMsg>>>,
    export_kind_selector: Arc<dyn ExportKindFor + Send + Sync>,
    metadata: Option<tonic::metadata::MetadataMap>,
}

impl Debug for MetricsExporter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "tonic")]
        f.debug_struct("OTLP Metric Exporter")
            .field("grpc_client", &"tonic")
            .finish()
    }
}

impl ExportKindFor for MetricsExporter {
    fn export_kind_for(&self, descriptor: &Descriptor) -> ExportKind {
        self.export_kind_selector.export_kind_for(descriptor)
    }
}

impl MetricsExporter {
    /// Create a new OTLP metrics exporter.
    #[cfg(feature = "tonic")]
    pub fn new<T: ExportKindFor + Send + Sync + 'static>(
        config: ExportConfig,
        mut tonic_config: TonicConfig,
        export_selector: T,
    ) -> Result<MetricsExporter> {
        let endpoint =
            Channel::from_shared(config.endpoint).map_err::<crate::Error, _>(Into::into)?;

        #[cfg(all(feature = "tls"))]
        let channel = match tonic_config.tls_config {
            Some(tls_config) => endpoint
                .tls_config(tls_config)
                .map_err::<crate::Error, _>(Into::into)?,
            None => endpoint,
        }
        .timeout(config.timeout)
        .connect_lazy();

        #[cfg(not(feature = "tls"))]
        let channel = endpoint.timeout(config.timeout).connect_lazy();

        let client = MetricsServiceClient::new(channel);

        let (sender, mut receiver) = tokio::sync::mpsc::channel::<ExportMsg>(2);
        tokio::spawn(Box::pin(async move {
            while let Some(msg) = receiver.recv().await {
                match msg {
                    ExportMsg::Shutdown => {
                        break;
                    }
                    ExportMsg::Export(req) => {
                        let _ = client.to_owned().export(req).await;
                    }
                }
            }
        }));

        Ok(MetricsExporter {
            sender: Arc::new(Mutex::new(sender)),
            export_kind_selector: Arc::new(export_selector),
            metadata: tonic_config.metadata.take(),
        })
    }
}

impl Exporter for MetricsExporter {
    fn export(&self, checkpoint_set: &mut dyn CheckpointSet) -> Result<()> {
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
        let mut request = Request::new(sink(resource_metrics));
        if let Some(metadata) = &self.metadata {
            for key_and_value in metadata.iter() {
                match key_and_value {
                    KeyAndValueRef::Ascii(key, value) => {
                        request.metadata_mut().append(key, value.to_owned())
                    }
                    KeyAndValueRef::Binary(key, value) => {
                        request.metadata_mut().append_bin(key, value.to_owned())
                    }
                };
            }
        }
        self.sender
            .lock()
            .map(|sender| {
                let _ = sender.try_send(ExportMsg::Export(request));
            })
            .map_err(|_| Error::PoisonedLock("otlp metric exporter's tonic sender"))?;
        Ok(())
    }
}

impl Drop for MetricsExporter {
    fn drop(&mut self) {
        let _sender_lock_guard = self.sender.lock().map(|sender| {
            let _ = sender.try_send(ExportMsg::Shutdown);
        });
    }
}
