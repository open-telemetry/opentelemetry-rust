//! OTEL metric exporter
//!
//! Defines a [MetricsExporter] to send metric data to backend via OTEL protocol.
//!
//! Currently, OTEL metrics exporter only support GRPC connection via tonic on tokio runtime.

use crate::exporter::tonic::TonicExporterBuilder;
use crate::transform::sink;
use crate::{Error, OtlpPipeline};
use async_trait::async_trait;
#[cfg(feature = "grpc-sys")]
use opentelemetry_proto::grpcio::metrics::Metric;
use core::fmt;
use opentelemetry_api::{
    global,
    metrics::{MetricsError, Result},
};
#[cfg(feature = "grpc-tonic")]
use opentelemetry_proto::tonic::collector::metrics::v1::{
    metrics_service_client::MetricsServiceClient, ExportMetricsServiceRequest,
};
use opentelemetry_sdk::{
    metrics::{
        data::{ResourceMetrics, Temporality},
        exporter::PushMetricsExporter,
        reader::{
            AggregationSelector, DefaultAggregationSelector, DefaultTemporalitySelector,
            TemporalitySelector,
        },
        Aggregation, InstrumentKind, MeterProvider, PeriodicReader,
    },
    runtime::Runtime,
    Resource,
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
#[cfg(feature = "http-proto")]
use {
    crate::exporter::http::{HttpExporterBuilder},
    http::{
        header::{HeaderName, HeaderValue, CONTENT_TYPE},
        Method, Uri,
    },
    opentelemetry_http::HttpClient,
    opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest as ProstRequest,
    prost::Message,
    std::convert::TryFrom,
    std::collections::HashMap,
    std::sync::Arc,
};

/// Target to which the exporter is going to send metrics, defaults to https://localhost:4317/v1/metrics.
/// Learn about the relationship between this constant and default/spans/logs at
/// <https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/protocol/exporter.md#endpoint-urls-for-otlphttp>
pub const OTEL_EXPORTER_OTLP_METRICS_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_METRICS_ENDPOINT";
/// Max waiting time for the backend to process each metrics batch, defaults to 10s.
pub const OTEL_EXPORTER_OTLP_METRICS_TIMEOUT: &str = "OTEL_EXPORTER_OTLP_METRICS_TIMEOUT";

impl OtlpPipeline {
    /// Create a OTLP metrics pipeline.
    pub fn metrics<RT>(self, rt: RT) -> OtlpMetricPipeline<RT>
    where
        RT: Runtime,
    {
        OtlpMetricPipeline {
            rt,
            aggregator_selector: None,
            temporality_selector: None,
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
    #[cfg(feature = "http-proto")]
    Http(HttpExporterBuilder),
}

impl MetricsExporterBuilder {
    /// Build a OTLP metrics exporter with given configuration.
    pub fn build_metrics_exporter(
        self,
        temporality_selector: Box<dyn TemporalitySelector>,
        aggregation_selector: Box<dyn AggregationSelector>,
    ) -> Result<MetricsExporter> {
        match self {
            #[cfg(feature = "grpc-tonic")]
            MetricsExporterBuilder::Tonic(builder) => Ok(MetricsExporter::new(
                builder,
                temporality_selector,
                aggregation_selector,
            )?),
            #[cfg(feature = "http-proto")]
            MetricsExporterBuilder::Http(builder) => Ok(MetricsExporter::new_http(
                builder,
                temporality_selector,
                aggregation_selector,
            )?),
        }
        
    }
}

impl From<TonicExporterBuilder> for MetricsExporterBuilder {
    fn from(exporter: TonicExporterBuilder) -> Self {
        MetricsExporterBuilder::Tonic(exporter)
    }
}

impl From<HttpExporterBuilder> for MetricsExporterBuilder {
    fn from(exporter: HttpExporterBuilder) -> Self {
        MetricsExporterBuilder::Http(exporter)
    }
}

/// Pipeline to build OTLP metrics exporter
///
/// Note that currently the OTLP metrics exporter only supports tonic as it's grpc layer and tokio as
/// runtime.
pub struct OtlpMetricPipeline<RT> {
    rt: RT,
    aggregator_selector: Option<Box<dyn AggregationSelector>>,
    temporality_selector: Option<Box<dyn TemporalitySelector>>,
    exporter_pipeline: Option<MetricsExporterBuilder>,
    resource: Option<Resource>,
    period: Option<time::Duration>,
    timeout: Option<time::Duration>,
}

impl<RT> OtlpMetricPipeline<RT>
where
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

    /// Build with the given temporality selector
    pub fn with_temporality_selector<T: TemporalitySelector + 'static>(self, selector: T) -> Self {
        OtlpMetricPipeline {
            temporality_selector: Some(Box::new(selector)),
            ..self
        }
    }

    /// Build with the given aggregation selector
    pub fn with_aggregation_selector<T: AggregationSelector + 'static>(self, selector: T) -> Self {
        OtlpMetricPipeline {
            aggregator_selector: Some(Box::new(selector)),
            ..self
        }
    }

    /// Build MeterProvider
    pub fn build(self) -> Result<MeterProvider> {
        let exporter = self
            .exporter_pipeline
            .ok_or(Error::NoExporterBuilder)?
            .build_metrics_exporter(
                self.temporality_selector
                    .unwrap_or_else(|| Box::new(DefaultTemporalitySelector::new())),
                self.aggregator_selector
                    .unwrap_or_else(|| Box::new(DefaultAggregationSelector::new())),
            )?;

        let mut builder = PeriodicReader::builder(exporter, self.rt);

        if let Some(period) = self.period {
            builder = builder.with_interval(period);
        }
        if let Some(timeout) = self.timeout {
            builder = builder.with_timeout(timeout)
        }

        let reader = builder.build();

        let mut provider = MeterProvider::builder().with_reader(reader);

        if let Some(resource) = self.resource {
            provider = provider.with_resource(resource);
        }

        let provider = provider.build();

        global::set_meter_provider(provider.clone());

        Ok(provider)
    }
}

impl<RT> fmt::Debug for OtlpMetricPipeline<RT> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("OtlpMetricPipeline")
            .field("exporter_pipeline", &self.exporter_pipeline)
            .field("resource", &self.resource)
            .field("period", &self.period)
            .field("timeout", &self.timeout)
            .finish()
    }
}

pub enum ExportMsg {
    #[cfg(feature = "grpc-tonic")]
    Export(tonic::Request<ExportMetricsServiceRequest>),
    Shutdown,
}

/// Export metrics in OTEL format.
pub enum MetricsExporter {
    #[cfg(feature = "tokio")]
    Tonic {
        sender: Mutex<tokio::sync::mpsc::Sender<ExportMsg>>,
        metadata: Option<tonic::metadata::MetadataMap>,
        temporality_selector: Box<dyn TemporalitySelector>,
        aggregation_selector: Box<dyn AggregationSelector>,
    },
    #[cfg(feature = "http-proto")]
    Http {
        timeout: Duration,
        headers: Option<HashMap<String, String>>,
        collector_endpoint: Uri,
        metrics_exporter: Option<Arc<dyn HttpClient>>,
        temporality_selector: Box<dyn TemporalitySelector>,
        aggregation_selector: Box<dyn AggregationSelector>,
    }
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
    fn temporality(&self, kind: InstrumentKind) -> Temporality {
        match self {
            #[cfg(feature = "grpc-tonic")]
            MetricsExporter::Tonic {
                sender, metadata, temporality_selector, ..
            } =>
            temporality_selector.temporality(kind),

            #[cfg(feature = "http-proto")]
            MetricsExporter::Http {
                timeout, headers, collector_endpoint, metrics_exporter, temporality_selector, ..
            } =>
            temporality_selector.temporality(kind),
        }
    }
}

impl AggregationSelector for MetricsExporter {
    fn aggregation(&self, kind: InstrumentKind) -> Aggregation {
        match self {
            #[cfg(feature = "grpc-tonic")]
            MetricsExporter::Tonic {
                sender, metadata, temporality_selector, aggregation_selector
            } =>
            aggregation_selector.aggregation(kind),

            #[cfg(feature = "http-proto")]
            MetricsExporter::Http {
                timeout, headers, collector_endpoint, metrics_exporter, temporality_selector, aggregation_selector
            } =>
            aggregation_selector.aggregation(kind),
        }
    }
}

impl MetricsExporter {
    /// Create a new OTLP metrics exporter.
    #[cfg(feature = "grpc-tonic")]
    pub fn new(
        export_builder: TonicExporterBuilder,
        temporality_selector: Box<dyn TemporalitySelector>,
        aggregation_selector: Box<dyn AggregationSelector>,
    ) -> Result<MetricsExporter> {
        let config = export_builder.exporter_config;
        let mut tonic_config = export_builder.tonic_config;

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

        let (sender, receiver) = tokio::sync::mpsc::channel::<ExportMsg>(2);
        tokio::spawn(async move {
            match export_builder.interceptor {
                Some(interceptor) => {
                    let client = MetricsServiceClient::with_interceptor(channel, interceptor);
                    export_sink(client, receiver).await
                }
                None => {
                    let client = MetricsServiceClient::new(channel);
                    export_sink(client, receiver).await
                }
            }
        });

        Ok(MetricsExporter::Tonic {
            sender: Mutex::new(sender),
            temporality_selector,
            aggregation_selector,
            metadata: tonic_config.metadata.take(),
        })
    }

    #[cfg(feature = "http-proto")]
    pub fn new_http(
        export_builder: HttpExporterBuilder,
        temporality_selector: Box<dyn TemporalitySelector>,
        aggregation_selector: Box<dyn AggregationSelector>,
        ) -> Result<MetricsExporter> {
            let config = export_builder.exporter_config;
            let http_config = export_builder.http_config;
            let _endpoint = match std::env::var(OTEL_EXPORTER_OTLP_METRICS_ENDPOINT) {
                Ok(val) => val,
                Err(_) => format!("{}{}", config.endpoint, "/v1/metrics"),
            };

            let url: Uri = config
            .endpoint
            .parse()
            .map_err::<crate::Error, _>(Into::into)?;

            let _timeout = match std::env::var(OTEL_EXPORTER_OTLP_METRICS_TIMEOUT) {
                Ok(val) => match u64::from_str(&val) {
                    Ok(seconds) => Duration::from_secs(seconds),
                    Err(_) => config.timeout,
                },
                Err(_) => config.timeout,
            };

            Ok(MetricsExporter::Http {
                metrics_exporter: http_config.client,
                timeout: config.timeout,
                collector_endpoint: url,
                headers: http_config.headers,
                temporality_selector,
                aggregation_selector,
            })

    }
}

#[cfg(feature = "http-proto")]
async fn http_send_request(
    metrics: &ResourceMetrics,
    client: std::sync::Arc<dyn HttpClient>,
    headers: Option<HashMap<String, String>>,
    collector_endpoint: Uri,
) -> Result<()> { 
    let req = sink(metrics);
    let mut buf = vec![];
    req.encode(&mut buf)
    .map_err::<crate::Error, _>(Into::into)?;
    let mut request = http::Request::builder()
        .method(Method::POST)
        .uri(collector_endpoint)
        .header(CONTENT_TYPE, "application/x-protobuf")
        .body(buf)
        .map_err::<crate::Error, _>(Into::into)?;

        if let Some(headers) = headers {
            for (k, val) in headers {
                let value =
                    HeaderValue::from_str(val.as_ref()).map_err::<crate::Error, _>(Into::into)?;
                let key = HeaderName::try_from(&k).map_err::<crate::Error, _>(Into::into)?;
                request.headers_mut().insert(key, value);
            }
        }
        client.send(request)
        .await
        .map_err( |_| Error::PoisonedLock("Error sending to collector"))?;
        Ok(())
}

use tonic::codegen::{Body, Bytes, StdError};
async fn export_sink<T>(
    mut client: MetricsServiceClient<T>,
    mut receiver: tokio::sync::mpsc::Receiver<ExportMsg>,
) where
    T: tonic::client::GrpcService<tonic::body::BoxBody>,
    T::Error: Into<StdError>,
    T::ResponseBody: Body<Data = Bytes> + Send + 'static,
    <T::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    while let Some(msg) = receiver.recv().await {
        match msg {
            ExportMsg::Shutdown => {
                break;
            }
            ExportMsg::Export(req) => {
                let _r = client.export(req).await;
            }
        }
    }
}

#[async_trait]
impl PushMetricsExporter for MetricsExporter {
    async fn export(&self, metrics: &mut ResourceMetrics) -> Result<()> {
        match self {
            #[cfg(feature = "grpc-tonic")]
            MetricsExporter::Tonic {
                sender, metadata, temporality_selector, aggregation_selector
            } => {
                let mut request = Request::new(sink(metrics));
                if let Some(metadata) = metadata {
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
                sender
                .lock()
                .map(|sender| {
                    let _ = sender.try_send(ExportMsg::Export(request));
                })
                .map_err(|_| Error::PoisonedLock("otlp metric exporter's tonic sender"))?;
                Ok(())
            },
            #[cfg(feature = "http-proto")]
            MetricsExporter::Http {
                timeout, headers, collector_endpoint, metrics_exporter, temporality_selector, aggregation_selector
            } => {
                if let Some(ref client) = metrics_exporter {
                    let client = Arc::clone(client);
                    http_send_request(
                        metrics,
                        client,
                        headers.clone(),
                        collector_endpoint.clone(),
                     ).await?;
                } else {
                }
                Ok(())
            }
        }
    }

    async fn force_flush(&self) -> Result<()> {
        // this component is stateless
        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        match self {
            #[cfg(feature = "grpc-tonic")]
            MetricsExporter::Tonic {
                sender, ..
             } =>  {
                let sender_lk = sender.lock()?;
                sender_lk
                .try_send(ExportMsg::Shutdown)
                .map_err(|e| MetricsError::Other(format!("error shutting down otlp {e}")))
            },
            #[cfg(feature = "http-proto")]
            MetricsExporter::Http {
                ..
            } => {
                Ok(())
            }
        }
    }
}
