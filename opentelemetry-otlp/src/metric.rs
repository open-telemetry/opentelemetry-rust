//! OTEL metric exporter
//!
//! Defines a [MetricsExporter] to send metric data to backend via OTEL protocol.
//!
//! Currently, OTEL metrics exporter only support GRPC connection via tonic on tokio runtime.

use crate::exporter::{
    tonic::{TonicConfig, TonicExporterBuilder},
    ExportConfig,
};
use crate::transform::{record_to_metric, sink, CheckpointedMetrics};
use crate::{Error, OtlpPipeline};
use core::fmt;
use opentelemetry::{global, metrics::Result, runtime::Runtime};
use opentelemetry::{
    sdk::{
        export::metrics::{
            self,
            aggregation::{AggregationKind, Temporality, TemporalitySelector},
            AggregatorSelector, InstrumentationLibraryReader,
        },
        metrics::{
            controllers::{self, BasicController},
            processors,
            sdk_api::Descriptor,
        },
        Resource,
    },
    Context,
};
#[cfg(feature = "grpc-tonic")]
use opentelemetry_proto::tonic::collector::metrics::v1::{
    metrics_service_client::MetricsServiceClient, ExportMetricsServiceRequest,
};
use std::fmt::{Debug, Formatter};
#[cfg(feature = "grpc-tonic")]
use std::str::FromStr;
use std::sync::Mutex;
use std::time;
use std::time::Duration;
use tonic::metadata::KeyAndValueRef;
#[cfg(feature = "grpc-tonic")]
use tonic::transport::Channel;
#[cfg(feature = "grpc-tonic")]
use tonic::Request;

/// Target to which the exporter is going to send metrics, defaults to https://localhost:4317/v1/metrics.
/// Learn about the relationship between this constant and default/spans/logs at
/// <https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/protocol/exporter.md#endpoint-urls-for-otlphttp>
pub const OTEL_EXPORTER_OTLP_METRICS_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_METRICS_ENDPOINT";
/// Max waiting time for the backend to process each metrics batch, defaults to 10s.
pub const OTEL_EXPORTER_OTLP_METRICS_TIMEOUT: &str = "OTEL_EXPORTER_OTLP_METRICS_TIMEOUT";

impl OtlpPipeline {
    /// Create a OTLP metrics pipeline.
    pub fn metrics<AS, TS, RT>(
        self,
        aggregator_selector: AS,
        temporality_selector: TS,
        rt: RT,
    ) -> OtlpMetricPipeline<AS, TS, RT>
    where
        AS: AggregatorSelector,
        TS: TemporalitySelector + Clone,
        RT: Runtime,
    {
        OtlpMetricPipeline {
            rt,
            aggregator_selector,
            temporality_selector,
            exporter_pipeline: None,
            resource: None,
            period: None,
            timeout: None,
        }
    }
}

/// OTLP metrics exporter builder.
#[derive(Debug)]
#[non_exhaustive]
pub enum MetricsExporterBuilder {
    /// Tonic metrics exporter builder
    #[cfg(feature = "grpc-tonic")]
    Tonic(TonicExporterBuilder),
}

impl MetricsExporterBuilder {
    /// Build a OTLP metrics exporter with given configuration.
    pub fn build_metrics_exporter(
        self,
        temporality_selector: Box<dyn TemporalitySelector + Send + Sync>,
    ) -> Result<MetricsExporter> {
        match self {
            #[cfg(feature = "grpc-tonic")]
            MetricsExporterBuilder::Tonic(builder) => Ok(MetricsExporter::new(
                builder.exporter_config,
                builder.tonic_config,
                temporality_selector,
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
pub struct OtlpMetricPipeline<AS, TS, RT> {
    rt: RT,
    aggregator_selector: AS,
    temporality_selector: TS,
    exporter_pipeline: Option<MetricsExporterBuilder>,
    resource: Option<Resource>,
    period: Option<time::Duration>,
    timeout: Option<time::Duration>,
}

impl<AS, TS, RT> OtlpMetricPipeline<AS, TS, RT>
where
    AS: AggregatorSelector + Send + Sync + 'static,
    TS: TemporalitySelector + Clone + Send + Sync + 'static,
    RT: Runtime,
{
    /// Build with resource key value pairs.
    pub fn with_resource(self, resource: Resource) -> Self {
        OtlpMetricPipeline {
            resource: Some(resource),
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

    /// Build push controller.
    pub fn build(self) -> Result<BasicController> {
        let exporter = self
            .exporter_pipeline
            .ok_or(Error::NoExporterBuilder)?
            .build_metrics_exporter(Box::new(self.temporality_selector.clone()))?;

        let mut builder = controllers::basic(processors::factory(
            self.aggregator_selector,
            self.temporality_selector,
        ))
        .with_exporter(exporter);
        if let Some(period) = self.period {
            builder = builder.with_collect_period(period);
        }
        if let Some(timeout) = self.timeout {
            builder = builder.with_collect_timeout(timeout)
        }
        if let Some(resource) = self.resource {
            builder = builder.with_resource(resource);
        }

        let controller = builder.build();
        controller.start(&Context::current(), self.rt)?;

        global::set_meter_provider(controller.clone());

        Ok(controller)
    }
}

impl<AS, TS, RT> fmt::Debug for OtlpMetricPipeline<AS, TS, RT> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("OtlpMetricPipeline")
            .field("exporter_pipeline", &self.exporter_pipeline)
            .field("resource", &self.resource)
            .field("period", &self.period)
            .field("timeout", &self.timeout)
            .finish()
    }
}

enum ExportMsg {
    #[cfg(feature = "grpc-tonic")]
    Export(tonic::Request<ExportMetricsServiceRequest>),
    Shutdown,
}

/// Export metrics in OTEL format.
pub struct MetricsExporter {
    #[cfg(feature = "tokio")]
    sender: Mutex<tokio::sync::mpsc::Sender<ExportMsg>>,
    temporality_selector: Box<dyn TemporalitySelector + Send + Sync>,
    metadata: Option<tonic::metadata::MetadataMap>,
}

impl Debug for MetricsExporter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "grpc-tonic")]
        f.debug_struct("OTLP Metric Exporter")
            .field("grpc_client", &"tonic")
            .finish()
    }
}

impl TemporalitySelector for MetricsExporter {
    fn temporality_for(&self, descriptor: &Descriptor, kind: &AggregationKind) -> Temporality {
        self.temporality_selector.temporality_for(descriptor, kind)
    }
}

impl MetricsExporter {
    /// Create a new OTLP metrics exporter.
    #[cfg(feature = "grpc-tonic")]
    pub fn new(
        config: ExportConfig,
        mut tonic_config: TonicConfig,
        temporality_selector: Box<dyn TemporalitySelector + Send + Sync>,
    ) -> Result<MetricsExporter> {
        let endpoint = match std::env::var(OTEL_EXPORTER_OTLP_METRICS_ENDPOINT) {
            Ok(val) => val,
            Err(_) => format!("{}{}", config.endpoint, "/v1/metrics"),
        };

        let _timeout = match std::env::var(OTEL_EXPORTER_OTLP_METRICS_TIMEOUT) {
            Ok(val) => match u64::from_str(&val) {
                Ok(seconds) => Duration::from_secs(seconds),
                Err(_) => config.timeout,
            },
            Err(_) => config.timeout,
        };

        let endpoint = Channel::from_shared(endpoint).map_err::<crate::Error, _>(Into::into)?;

        #[cfg(all(feature = "tls"))]
        let channel = match tonic_config.tls_config {
            Some(tls_config) => endpoint
                .tls_config(tls_config)
                .map_err::<crate::Error, _>(Into::into)?,
            None => endpoint,
        }
        .timeout(_timeout)
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
            sender: Mutex::new(sender),
            temporality_selector,
            metadata: tonic_config.metadata.take(),
        })
    }
}

impl metrics::MetricsExporter for MetricsExporter {
    fn export(
        &self,
        _cx: &Context,
        res: &Resource,
        reader: &dyn InstrumentationLibraryReader,
    ) -> Result<()> {
        let mut resource_metrics: Vec<CheckpointedMetrics> = Vec::default();
        // transform the metrics into proto. Append the resource and instrumentation library information into it.
        reader.try_for_each(&mut |library, record| {
            record.try_for_each(self, &mut |record| {
                let metrics = record_to_metric(record, self.temporality_selector.as_ref())?;
                resource_metrics.push((res.clone().into(), library.clone(), metrics));
                Ok(())
            })
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
