//! # OTLP - Span Exporter
//!
//! Defines a [SpanExporter] to send trace data via the OpenTelemetry Protocol (OTLP)

use std::fmt::{self, Debug};
use std::time::Duration;

#[cfg(feature = "grpc-tonic")]
use {
    crate::exporter::tonic::{TonicConfig, TonicExporterBuilder},
    opentelemetry_proto::tonic::collector::trace::v1::{
        trace_service_client::TraceServiceClient as TonicTraceServiceClient,
        ExportTraceServiceRequest as TonicRequest,
    },
    tonic::{
        metadata::{KeyAndValueRef, MetadataMap},
        transport::Channel as TonicChannel,
        Request,
    },
};

#[cfg(feature = "grpc-sys")]
use {
    crate::exporter::grpcio::{GrpcioConfig, GrpcioExporterBuilder},
    grpcio::{
        CallOption, Channel as GrpcChannel, ChannelBuilder, ChannelCredentialsBuilder, Environment,
        MetadataBuilder,
    },
    opentelemetry_proto::grpcio::{
        trace_service::ExportTraceServiceRequest as GrpcRequest,
        trace_service_grpc::TraceServiceClient as GrpcioTraceServiceClient,
    },
};

#[cfg(feature = "http-proto")]
use {
    crate::exporter::http::{HttpConfig, HttpExporterBuilder},
    http::{
        header::{HeaderName, HeaderValue, CONTENT_TYPE},
        Method, Uri,
    },
    opentelemetry_http::HttpClient,
    opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest as ProstRequest,
    prost::Message,
    std::convert::TryFrom,
};

#[cfg(any(feature = "grpc-sys", feature = "http-proto"))]
use {std::collections::HashMap, std::sync::Arc};

use crate::exporter::ExportConfig;
use crate::OtlpPipeline;

use opentelemetry::{
    global,
    sdk::{
        self,
        export::trace::{ExportResult, SpanData},
        trace::TraceRuntime,
    },
    trace::{TraceError, TracerProvider},
};

use async_trait::async_trait;

impl OtlpPipeline {
    /// Create a OTLP tracing pipeline.
    pub fn tracing(self) -> OtlpTracePipeline {
        OtlpTracePipeline::default()
    }
}

/// Recommended configuration for an OTLP exporter pipeline.
///
/// ## Examples
///
/// ```no_run
/// let tracing_pipeline = opentelemetry_otlp::new_pipeline().tracing();
/// ```
#[derive(Default, Debug)]
pub struct OtlpTracePipeline {
    exporter_builder: Option<SpanExporterBuilder>,
    trace_config: Option<sdk::trace::Config>,
}

impl OtlpTracePipeline {
    /// Set the trace provider configuration.
    pub fn with_trace_config(mut self, trace_config: sdk::trace::Config) -> Self {
        self.trace_config = Some(trace_config);
        self
    }

    /// Set the OTLP span exporter builder.
    ///
    /// Note that the pipeline will not build the exporter until [`install_batch`] or [`install_simple`]
    /// is called.
    ///
    /// [`install_batch`]: OtlpTracePipeline::install_batch
    /// [`install_simple`]: OtlpTracePipeline::install_simple
    pub fn with_exporter<B: Into<SpanExporterBuilder>>(mut self, pipeline: B) -> Self {
        self.exporter_builder = Some(pipeline.into());
        self
    }

    /// Install the configured span exporter.
    ///
    /// Returns a [`Tracer`] with the name `opentelemetry-otlp` and current crate version.
    ///
    /// [`Tracer`]: opentelemetry::trace::Tracer
    pub fn install_simple(self) -> Result<sdk::trace::Tracer, TraceError> {
        Ok(build_simple_with_exporter(
            self.exporter_builder
                .ok_or(crate::Error::NoExporterBuilder)?
                .build_span_exporter()?,
            self.trace_config,
        ))
    }

    /// Install the configured span exporter and a batch span processor using the
    /// specified runtime.
    ///
    /// Returns a [`Tracer`] with the name `opentelemetry-otlp` and current crate version.
    ///
    /// `install_batch` will panic if not called within a tokio runtime
    ///
    /// [`Tracer`]: opentelemetry::trace::Tracer
    pub fn install_batch<R: TraceRuntime>(
        self,
        runtime: R,
    ) -> Result<sdk::trace::Tracer, TraceError> {
        Ok(build_batch_with_exporter(
            self.exporter_builder
                .ok_or(crate::Error::NoExporterBuilder)?
                .build_span_exporter()?,
            self.trace_config,
            runtime,
        ))
    }
}

fn build_simple_with_exporter(
    exporter: SpanExporter,
    trace_config: Option<sdk::trace::Config>,
) -> sdk::trace::Tracer {
    let mut provider_builder = sdk::trace::TracerProvider::builder().with_simple_exporter(exporter);
    if let Some(config) = trace_config {
        provider_builder = provider_builder.with_config(config);
    }
    let provider = provider_builder.build();
    let tracer =
        provider.versioned_tracer("opentelemetry-otlp", Some(env!("CARGO_PKG_VERSION")), None);
    let _ = global::set_tracer_provider(provider);
    tracer
}

fn build_batch_with_exporter<R: TraceRuntime>(
    exporter: SpanExporter,
    trace_config: Option<sdk::trace::Config>,
    runtime: R,
) -> sdk::trace::Tracer {
    let mut provider_builder =
        sdk::trace::TracerProvider::builder().with_batch_exporter(exporter, runtime);
    if let Some(config) = trace_config {
        provider_builder = provider_builder.with_config(config);
    }
    let provider = provider_builder.build();
    let tracer =
        provider.versioned_tracer("opentelemetry-otlp", Some(env!("CARGO_PKG_VERSION")), None);
    let _ = global::set_tracer_provider(provider);
    tracer
}

/// OTLP span exporter builder.
#[derive(Debug)]
// This enum only used during initialization stage of application. The overhead should be OK.
// Users can also disable the unused features to make the overhead on object size smaller.
#[allow(clippy::large_enum_variant)]
#[non_exhaustive]
pub enum SpanExporterBuilder {
    /// Tonic span exporter builder
    #[cfg(feature = "grpc-tonic")]
    Tonic(TonicExporterBuilder),
    /// Grpc span exporter builder
    #[cfg(feature = "grpc-sys")]
    Grpcio(GrpcioExporterBuilder),
    /// Http span exporter builder
    #[cfg(feature = "http-proto")]
    Http(HttpExporterBuilder),
}

impl SpanExporterBuilder {
    /// Build a OTLP span exporter using the given tonic configuration and exporter configuration.
    pub fn build_span_exporter(self) -> Result<SpanExporter, TraceError> {
        match self {
            #[cfg(feature = "grpc-tonic")]
            SpanExporterBuilder::Tonic(builder) => Ok(match builder.channel {
                Some(channel) => SpanExporter::from_tonic_channel(
                    builder.exporter_config,
                    builder.tonic_config,
                    channel,
                ),
                None => SpanExporter::new_tonic(builder.exporter_config, builder.tonic_config),
            }?),
            #[cfg(feature = "grpc-sys")]
            SpanExporterBuilder::Grpcio(builder) => Ok(SpanExporter::new_grpcio(
                builder.exporter_config,
                builder.grpcio_config,
            )),
            #[cfg(feature = "http-proto")]
            SpanExporterBuilder::Http(builder) => Ok(SpanExporter::new_http(
                builder.exporter_config,
                builder.http_config,
            )?),
        }
    }
}

#[cfg(feature = "grpc-tonic")]
impl From<TonicExporterBuilder> for SpanExporterBuilder {
    fn from(exporter: TonicExporterBuilder) -> Self {
        SpanExporterBuilder::Tonic(exporter)
    }
}

#[cfg(feature = "grpc-sys")]
impl From<GrpcioExporterBuilder> for SpanExporterBuilder {
    fn from(exporter: GrpcioExporterBuilder) -> Self {
        SpanExporterBuilder::Grpcio(exporter)
    }
}

#[cfg(feature = "http-proto")]
impl From<HttpExporterBuilder> for SpanExporterBuilder {
    fn from(exporter: HttpExporterBuilder) -> Self {
        SpanExporterBuilder::Http(exporter)
    }
}

/// OTLP exporter that sends tracing information
pub enum SpanExporter {
    #[cfg(feature = "grpc-tonic")]
    /// Trace Exporter using tonic as grpc layer.
    Tonic {
        /// Duration of timeout when sending spans to backend.
        timeout: Duration,
        /// Additional headers of the outbound requests.
        metadata: Option<MetadataMap>,
        /// The Grpc trace exporter
        trace_exporter: TonicTraceServiceClient<TonicChannel>,
    },
    #[cfg(feature = "grpc-sys")]
    /// Trace Exporter using grpcio as grpc layer
    Grpcio {
        /// Duration of timeout when sending spans to backend.
        timeout: Duration,
        /// Additional headers of the outbound requests.
        headers: Option<HashMap<String, String>>,
        /// The Grpc trace exporter
        trace_exporter: GrpcioTraceServiceClient,
    },
    #[cfg(feature = "http-proto")]
    /// Trace Exporter using HTTP transport
    Http {
        /// Duration of timeout when sending spans to backend.
        timeout: Duration,
        /// Additional headers of the outbound requests.
        headers: Option<HashMap<String, String>>,
        /// The Collector URL
        collector_endpoint: Uri,
        /// The HTTP trace exporter
        trace_exporter: Option<Arc<dyn HttpClient>>,
    },
}

impl Debug for SpanExporter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "grpc-tonic")]
            SpanExporter::Tonic {
                metadata, timeout, ..
            } => f
                .debug_struct("Exporter")
                .field("metadata", &metadata)
                .field("timeout", &timeout)
                .field("trace_exporter", &"TraceServiceClient")
                .finish(),
            #[cfg(feature = "grpc-sys")]
            SpanExporter::Grpcio {
                headers, timeout, ..
            } => f
                .debug_struct("Exporter")
                .field("headers", &headers)
                .field("timeout", &timeout)
                .field("trace_exporter", &"TraceServiceClient")
                .finish(),
            #[cfg(feature = "http-proto")]
            SpanExporter::Http {
                headers, timeout, ..
            } => f
                .debug_struct("Exporter")
                .field("headers", &headers)
                .field("timeout", &timeout)
                .field("trace_exporter", &"TraceServiceClient")
                .finish(),
        }
    }
}

impl SpanExporter {
    /// Builds a new span exporter with the given configuration.
    #[cfg(feature = "grpc-tonic")]
    pub fn new_tonic(
        config: ExportConfig,
        tonic_config: TonicConfig,
    ) -> Result<Self, crate::Error> {
        let endpoint = TonicChannel::from_shared(config.endpoint.clone())?;

        #[cfg(feature = "tls")]
        let channel = match tonic_config.tls_config.as_ref() {
            Some(tls_config) => endpoint.tls_config(tls_config.clone())?,
            None => endpoint,
        }
        .timeout(config.timeout)
        .connect_lazy();

        #[cfg(not(feature = "tls"))]
        let channel = endpoint.timeout(config.timeout).connect_lazy();

        SpanExporter::from_tonic_channel(config, tonic_config, channel)
    }

    /// Builds a new span exporter with given tonic channel.
    ///
    /// This allows users to bring their own custom channel like UDS.
    /// However, users MUST make sure the [`ExportConfig::timeout`] is
    /// the same as the channel's timeout.
    #[cfg(feature = "grpc-tonic")]
    pub fn from_tonic_channel(
        config: ExportConfig,
        tonic_config: TonicConfig,
        channel: tonic::transport::Channel,
    ) -> Result<Self, crate::Error> {
        Ok(SpanExporter::Tonic {
            timeout: config.timeout,
            metadata: tonic_config.metadata,
            trace_exporter: TonicTraceServiceClient::new(channel),
        })
    }

    /// Builds a new span exporter with the given configuration
    #[cfg(feature = "grpc-sys")]
    pub fn new_grpcio(config: ExportConfig, grpcio_config: GrpcioConfig) -> Self {
        let mut builder: ChannelBuilder = ChannelBuilder::new(Arc::new(Environment::new(
            grpcio_config.completion_queue_count,
        )));

        if let Some(compression) = grpcio_config.compression {
            builder = builder.default_compression_algorithm(compression.into());
        }

        let channel: GrpcChannel = match (grpcio_config.credentials, grpcio_config.use_tls) {
            (None, Some(true)) => builder.secure_connect(
                config.endpoint.as_str(),
                ChannelCredentialsBuilder::new().build(),
            ),
            (None, _) => builder.connect(config.endpoint.as_str()),
            (Some(credentials), _) => builder.secure_connect(
                config.endpoint.as_str(),
                ChannelCredentialsBuilder::new()
                    .cert(credentials.cert.into(), credentials.key.into())
                    .build(),
            ),
        };

        SpanExporter::Grpcio {
            trace_exporter: GrpcioTraceServiceClient::new(channel),
            timeout: config.timeout,
            headers: grpcio_config.headers,
        }
    }

    /// Builds a new span exporter with the given configuration
    #[cfg(feature = "http-proto")]
    pub fn new_http(config: ExportConfig, http_config: HttpConfig) -> Result<Self, crate::Error> {
        let url: Uri = config
            .endpoint
            .parse()
            .map_err::<crate::Error, _>(Into::into)?;

        Ok(SpanExporter::Http {
            trace_exporter: http_config.client,
            timeout: config.timeout,
            collector_endpoint: url,
            headers: http_config.headers,
        })
    }
}

#[cfg(feature = "grpc-sys")]
async fn grpcio_send_request(
    trace_exporter: GrpcioTraceServiceClient,
    request: GrpcRequest,
    call_options: CallOption,
) -> ExportResult {
    let receiver = trace_exporter
        .export_async_opt(&request, call_options)
        .map_err::<crate::Error, _>(Into::into)?;
    receiver.await.map_err::<crate::Error, _>(Into::into)?;
    Ok(())
}

#[cfg(feature = "tonic")]
async fn tonic_send_request(
    trace_exporter: TonicTraceServiceClient<TonicChannel>,
    request: Request<TonicRequest>,
) -> ExportResult {
    trace_exporter
        .to_owned()
        .export(request)
        .await
        .map_err::<crate::Error, _>(Into::into)?;

    Ok(())
}

#[cfg(feature = "http-proto")]
async fn http_send_request(
    batch: Vec<SpanData>,
    client: std::sync::Arc<dyn HttpClient>,
    headers: Option<HashMap<String, String>>,
    collector_endpoint: Uri,
) -> ExportResult {
    let req = ProstRequest {
        resource_spans: batch.into_iter().map(Into::into).collect(),
    };

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

    client.send(request).await?;
    Ok(())
}

#[async_trait]
impl opentelemetry::sdk::export::trace::SpanExporter for SpanExporter {
    fn export(
        &mut self,
        batch: Vec<SpanData>,
    ) -> futures::future::BoxFuture<'static, ExportResult> {
        match self {
            #[cfg(feature = "grpc-sys")]
            SpanExporter::Grpcio {
                timeout,
                headers,
                trace_exporter,
            } => {
                let request = GrpcRequest {
                    resource_spans: protobuf::RepeatedField::from_vec(
                        batch.into_iter().map(Into::into).collect(),
                    ),
                    unknown_fields: Default::default(),
                    cached_size: Default::default(),
                };

                let mut call_options = CallOption::default().timeout(*timeout);

                if let Some(headers) = headers.clone() {
                    let mut metadata_builder: MetadataBuilder = MetadataBuilder::new();

                    for (key, value) in headers {
                        let _ = metadata_builder.add_str(key.as_str(), value.as_str());
                    }

                    call_options = call_options.headers(metadata_builder.build());
                }

                Box::pin(grpcio_send_request(
                    trace_exporter.clone(),
                    request,
                    call_options,
                ))
            }

            #[cfg(feature = "grpc-tonic")]
            SpanExporter::Tonic {
                trace_exporter,
                metadata,
                ..
            } => {
                let mut request = Request::new(TonicRequest {
                    resource_spans: batch.into_iter().map(Into::into).collect(),
                });

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

                Box::pin(tonic_send_request(trace_exporter.to_owned(), request))
            }

            #[cfg(feature = "http-proto")]
            SpanExporter::Http {
                trace_exporter,
                collector_endpoint,
                headers,
                ..
            } => {
                if let Some(ref client) = trace_exporter {
                    let client = Arc::clone(client);
                    Box::pin(http_send_request(
                        batch,
                        client,
                        headers.clone(),
                        collector_endpoint.clone(),
                    ))
                } else {
                    Box::pin(std::future::ready(Err(crate::Error::NoHttpClient.into())))
                }
            }
        }
    }
}
