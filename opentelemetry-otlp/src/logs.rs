//! OTLP - Log Exporter
//!
//! Defines a [LogExporter] to send logs via the OpenTelemetry Protocol (OTLP)

#[cfg(feature = "grpc-tonic")]
use {
    crate::exporter::tonic::{TonicConfig, TonicExporterBuilder},
    opentelemetry_proto::tonic::collector::logs::v1::{
        logs_service_client::LogsServiceClient as TonicLogsServiceClient,
        ExportLogsServiceRequest as TonicRequest,
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
        logs_service::ExportLogsServiceRequest as GrpcRequest,
        logs_service_grpc::LogsServiceClient as GrpcioLogServiceClient,
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
    opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest as ProstRequest,
    prost::Message,
    std::convert::TryFrom,
};

#[cfg(any(feature = "grpc-sys", feature = "http-proto"))]
use std::{collections::HashMap, sync::Arc};

use crate::exporter::ExportConfig;
use crate::OtlpPipeline;
use async_trait::async_trait;
use std::{
    borrow::Cow,
    fmt::{self, Debug},
    time::Duration,
};

use opentelemetry_api::logs::{LogError, LoggerProvider};
use opentelemetry_sdk::{self, export::logs::LogData, logs::LogRuntime};

impl OtlpPipeline {
    /// Create a OTLP logging pipeline.
    pub fn logging(self) -> OtlpLogPipeline {
        OtlpLogPipeline::default()
    }
}

/// OTLP log exporter builder
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
#[non_exhaustive]
pub enum LogExporterBuilder {
    /// Tonic log exporter builder
    #[cfg(feature = "grpc-tonic")]
    Tonic(TonicExporterBuilder),
    /// Grpc log exporter builder
    #[cfg(feature = "grpc-sys")]
    Grpcio(GrpcioExporterBuilder),
    /// Http log exporter builder
    #[cfg(feature = "http-proto")]
    Http(HttpExporterBuilder),
}

impl LogExporterBuilder {
    /// Build a OTLP log exporter using the given configuration.
    pub fn build_log_exporter(self) -> Result<LogExporter, LogError> {
        match self {
            #[cfg(feature = "grpc-tonic")]
            LogExporterBuilder::Tonic(builder) => Ok(match builder.channel {
                Some(channel) => LogExporter::from_tonic_channel(
                    builder.exporter_config,
                    builder.tonic_config,
                    channel,
                ),
                None => LogExporter::new_tonic(builder.exporter_config, builder.tonic_config),
            }?),
            #[cfg(feature = "grpc-sys")]
            LogExporterBuilder::Grpcio(builder) => Ok(LogExporter::new_grpcio(
                builder.exporter_config,
                builder.grpcio_config,
            )),
            #[cfg(feature = "http-proto")]
            LogExporterBuilder::Http(builder) => Ok(LogExporter::new_http(
                builder.exporter_config,
                builder.http_config,
            )?),
        }
    }
}

#[cfg(feature = "grpc-tonic")]
impl From<TonicExporterBuilder> for LogExporterBuilder {
    fn from(exporter: TonicExporterBuilder) -> Self {
        LogExporterBuilder::Tonic(exporter)
    }
}

#[cfg(feature = "grpc-sys")]
impl From<GrpcioExporterBuilder> for LogExporterBuilder {
    fn from(exporter: GrpcioExporterBuilder) -> Self {
        LogExporterBuilder::Grpcio(exporter)
    }
}

#[cfg(feature = "http-proto")]
impl From<HttpExporterBuilder> for LogExporterBuilder {
    fn from(exporter: HttpExporterBuilder) -> Self {
        LogExporterBuilder::Http(exporter)
    }
}

/// OTLP exporter that sends log data
pub enum LogExporter {
    #[cfg(feature = "grpc-tonic")]
    /// Log Exporter using tonic as grpc layer.
    Tonic {
        /// Duration of timeout when sending logs to backend.
        timeout: Duration,
        /// Additional headers of the outbound requests.
        metadata: Option<MetadataMap>,
        /// The Grpc log exporter
        log_exporter: TonicLogsServiceClient<TonicChannel>,
    },
    #[cfg(feature = "grpc-sys")]
    /// Log Exporter using grpcio as grpc layer
    Grpcio {
        /// Duration of timeout when sending logs to backend.
        timeout: Duration,
        /// Additional headers of the outbound requests.
        headers: Option<HashMap<String, String>>,
        /// The Grpc log exporter
        log_exporter: GrpcioLogServiceClient,
    },
    #[cfg(feature = "http-proto")]
    /// Log Exporter using HTTP transport
    Http {
        /// Duration of timeout when sending logs to backend.
        timeout: Duration,
        /// Additional headers of the outbound requests.
        headers: Option<HashMap<String, String>>,
        /// The Collector URL
        collector_endpoint: Uri,
        /// The HTTP log exporter
        log_exporter: Option<Arc<dyn HttpClient>>,
    },
}

impl Debug for LogExporter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "grpc-tonic")]
            LogExporter::Tonic {
                metadata, timeout, ..
            } => f
                .debug_struct("Exporter")
                .field("metadata", &metadata)
                .field("timeout", &timeout)
                .field("log_exporter", &"LogServiceClient")
                .finish(),
            #[cfg(feature = "grpc-sys")]
            LogExporter::Grpcio {
                headers, timeout, ..
            } => f
                .debug_struct("Exporter")
                .field("headers", &headers)
                .field("timeout", &timeout)
                .field("log_exporter", &"LogServiceClient")
                .finish(),
            #[cfg(feature = "http-proto")]
            LogExporter::Http {
                headers, timeout, ..
            } => f
                .debug_struct("Exporter")
                .field("headers", &headers)
                .field("timeout", &timeout)
                .field("log_exporter", &"LogServiceClient")
                .finish(),
        }
    }
}

impl LogExporter {
    /// Builds a new log exporter with the given configuration.
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

        LogExporter::from_tonic_channel(config, tonic_config, channel)
    }

    /// Builds a new log exporter with given tonic channel.
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
        Ok(LogExporter::Tonic {
            timeout: config.timeout,
            metadata: tonic_config.metadata,
            log_exporter: TonicLogsServiceClient::new(channel),
        })
    }

    /// Builds a new log exporter with the given configuration
    #[cfg(feature = "grpc-sys")]
    pub fn new_grpcio(config: ExportConfig, grpcio_config: GrpcioConfig) -> Self {
        let mut builder: ChannelBuilder = ChannelBuilder::new(Arc::new(Environment::new(
            grpcio_config.completion_queue_count,
        )));

        if let Some(compression) = grpcio_config.compression {
            builder = builder.default_compression_algorithm(compression.into());
        }

        let channel: GrpcChannel = match (grpcio_config.credentials, grpcio_config.use_tls) {
            (None, Some(true)) => builder
                .set_credentials(ChannelCredentialsBuilder::new().build())
                .connect(config.endpoint.as_str()),
            (None, _) => builder.connect(config.endpoint.as_str()),
            (Some(credentials), _) => builder
                .set_credentials(
                    ChannelCredentialsBuilder::new()
                        .cert(credentials.cert.into(), credentials.key.into())
                        .build(),
                )
                .connect(config.endpoint.as_str()),
        };

        LogExporter::Grpcio {
            log_exporter: GrpcioLogServiceClient::new(channel),
            timeout: config.timeout,
            headers: grpcio_config.headers,
        }
    }

    /// Builds a new log exporter with the given configuration
    #[cfg(feature = "http-proto")]
    pub fn new_http(config: ExportConfig, http_config: HttpConfig) -> Result<Self, crate::Error> {
        let url: Uri = config
            .endpoint
            .parse()
            .map_err::<crate::Error, _>(Into::into)?;

        Ok(LogExporter::Http {
            log_exporter: http_config.client,
            timeout: config.timeout,
            collector_endpoint: url,
            headers: http_config.headers,
        })
    }
}

#[async_trait]
impl opentelemetry_sdk::export::logs::LogExporter for LogExporter {
    async fn export(&mut self, batch: Vec<LogData>) -> opentelemetry_api::logs::LogResult<()> {
        match self {
            #[cfg(feature = "grpc-sys")]
            LogExporter::Grpcio {
                timeout,
                headers,
                log_exporter,
            } => {
                let request = GrpcRequest {
                    resource_logs: protobuf::RepeatedField::from_vec(
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

                let receiver = log_exporter
                    .export_async_opt(&request, call_options)
                    .map_err::<crate::Error, _>(Into::into)?;
                receiver.await.map_err::<crate::Error, _>(Into::into)?;
                Ok(())
            }
            #[cfg(feature = "grpc-tonic")]
            LogExporter::Tonic {
                log_exporter,
                metadata,
                ..
            } => {
                let mut request = Request::new(TonicRequest {
                    resource_logs: batch.into_iter().map(Into::into).collect(),
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

                log_exporter
                    .to_owned()
                    .export(request)
                    .await
                    .map_err::<crate::Error, _>(Into::into)?;

                Ok(())
            }

            #[cfg(feature = "http-proto")]
            LogExporter::Http {
                log_exporter,
                collector_endpoint,
                headers,
                ..
            } => {
                let req = ProstRequest {
                    resource_logs: batch.into_iter().map(Into::into).collect(),
                };

                let mut buf = vec![];
                req.encode(&mut buf)
                    .map_err::<crate::Error, _>(Into::into)?;

                let mut request = http::Request::builder()
                    .method(Method::POST)
                    .uri(collector_endpoint.clone())
                    .header(CONTENT_TYPE, "application/x-protobuf")
                    .body(buf)
                    .map_err::<crate::Error, _>(Into::into)?;

                if let Some(headers) = headers.clone() {
                    for (k, val) in headers {
                        let value = HeaderValue::from_str(val.as_ref())
                            .map_err::<crate::Error, _>(Into::into)?;
                        let key =
                            HeaderName::try_from(&k).map_err::<crate::Error, _>(Into::into)?;
                        request.headers_mut().insert(key, value);
                    }
                }

                if let Some(client) = log_exporter {
                    client.send(request).await?;
                    Ok(())
                } else {
                    Err(crate::Error::NoHttpClient.into())
                }
            }
        }
    }
}

/// Recommended configuration for an OTLP exporter pipeline.
#[derive(Default, Debug)]
pub struct OtlpLogPipeline {
    exporter_builder: Option<LogExporterBuilder>,
    log_config: Option<opentelemetry_sdk::logs::Config>,
}

impl OtlpLogPipeline {
    /// Set the OTLP log exporter builder.
    pub fn with_exporter<B: Into<LogExporterBuilder>>(mut self, pipeline: B) -> Self {
        self.exporter_builder = Some(pipeline.into());
        self
    }

    /// Returns a [`Logger`] with the name `opentelemetry-otlp` and the
    /// current crate version, using the configured log exporter.
    ///
    /// [`Logger`]: opentelemetry_sdk::logs::Logger
    pub fn simple(
        self,
        include_trace_context: bool,
    ) -> Result<opentelemetry_sdk::logs::Logger, LogError> {
        Ok(build_simple_with_exporter(
            self.exporter_builder
                .ok_or(crate::Error::NoExporterBuilder)?
                .build_log_exporter()?,
            self.log_config,
            include_trace_context,
        ))
    }

    /// Returns a [`Logger`] with the name `opentelemetry-otlp` and the
    /// current crate version, using the configured log exporter and a
    /// batch log processor.
    ///
    /// [`Logger`]: opentelemetry_sdk::logs::Logger
    pub fn batch<R: LogRuntime>(
        self,
        runtime: R,
        include_trace_context: bool,
    ) -> Result<opentelemetry_sdk::logs::Logger, LogError> {
        Ok(build_batch_with_exporter(
            self.exporter_builder
                .ok_or(crate::Error::NoExporterBuilder)?
                .build_log_exporter()?,
            self.log_config,
            runtime,
            include_trace_context,
        ))
    }
}

fn build_simple_with_exporter(
    exporter: LogExporter,
    log_config: Option<opentelemetry_sdk::logs::Config>,
    include_trace_context: bool,
) -> opentelemetry_sdk::logs::Logger {
    let mut provider_builder =
        opentelemetry_sdk::logs::LoggerProvider::builder().with_simple_exporter(exporter);
    if let Some(config) = log_config {
        provider_builder = provider_builder.with_config(config);
    }
    let provider = provider_builder.build();
    provider.versioned_logger(
        Cow::Borrowed("opentelemetry-otlp"),
        Some(Cow::Borrowed(env!("CARGO_PKG_VERSION"))),
        None,
        None,
        include_trace_context,
    )
}

fn build_batch_with_exporter<R: LogRuntime>(
    exporter: LogExporter,
    log_config: Option<opentelemetry_sdk::logs::Config>,
    runtime: R,
    include_trace_context: bool,
) -> opentelemetry_sdk::logs::Logger {
    let mut provider_builder =
        opentelemetry_sdk::logs::LoggerProvider::builder().with_batch_exporter(exporter, runtime);
    if let Some(config) = log_config {
        provider_builder = provider_builder.with_config(config);
    }
    let provider = provider_builder.build();
    provider.versioned_logger(
        Cow::Borrowed("opentelemetry-otlp"),
        Some(Cow::Borrowed("CARGO_PKG_VERSION")),
        None,
        None,
        include_trace_context,
    )
}
