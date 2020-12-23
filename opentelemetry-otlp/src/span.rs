//! # OTLP - Span Exporter
//!
//! Defines a [SpanExporter] to send trace data via the OpenTelemetry Protocol (OTLP)

#[cfg(feature = "tonic")]
use crate::proto::collector::trace::v1::{
    trace_service_client::TraceServiceClient, ExportTraceServiceRequest,
};

#[cfg(feature = "tonic")]
use tonic::{
    metadata::{KeyAndValueRef, MetadataMap},
    transport::Channel,
    Request,
};

#[cfg(all(feature = "tonic", feature = "tls"))]
use tonic::transport::ClientTlsConfig;

#[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
use crate::proto::grpcio::trace_service::ExportTraceServiceRequest;

#[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
use crate::proto::grpcio::trace_service_grpc::TraceServiceClient;

#[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
use grpcio::{
    CallOption, Channel, ChannelBuilder, ChannelCredentialsBuilder, Environment, MetadataBuilder,
};

#[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
use protobuf::RepeatedField;

use async_trait::async_trait;

#[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
use std::collections::HashMap;

use std::fmt;
use std::fmt::Debug;

#[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
use std::sync::Arc;

use opentelemetry::sdk::export::trace::{ExportResult, SpanData, SpanExporter};
use std::time::Duration;

/// Exporter that sends data in OTLP format.
pub struct Exporter {
    #[cfg(feature = "tonic")]
    metadata: Option<MetadataMap>,

    #[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
    headers: Option<HashMap<String, String>>,

    timeout: Duration,

    #[cfg(feature = "tonic")]
    trace_exporter: TraceServiceClient<Channel>,

    #[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
    trace_exporter: TraceServiceClient,

    #[cfg(all(feature = "default", not(feature = "async")))]
    runtime: tokio::runtime::Runtime,
}

/// Configuration for the OTLP exporter.
#[derive(Debug)]
pub struct ExporterConfig {
    /// The address of the OTLP collector. If not set, the default address is used.
    pub endpoint: String,

    /// The protocol to use when communicating with the collector.
    pub protocol: Protocol,

    #[cfg(all(feature = "tonic", feature = "tls"))]
    /// TLS settings for the collector endpoint.
    pub tls_config: Option<ClientTlsConfig>,

    #[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
    /// The credentials to use when communicating with the collector.
    pub credentials: Option<Credentials>,

    /// Custom metadata entries to send to the collector.
    #[cfg(feature = "tonic")]
    pub metadata: Option<MetadataMap>,

    /// Additional headers to send to the collector.
    #[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
    pub headers: Option<HashMap<String, String>>,

    /// The compression algorithm to use when communicating with the collector.
    #[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
    pub compression: Option<Compression>,

    /// The timeout to the collector.
    pub timeout: Duration,

    /// The number of GRPC worker threads to poll queues.
    #[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
    pub completion_queue_count: usize,

    /// The Tokio runtime.
    #[cfg(all(feature = "tonic", not(feature = "async")))]
    pub runtime: Option<tokio::runtime::Runtime>,
}

/// Credential configuration for authenticated requests.
#[derive(Debug)]
#[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
pub struct Credentials {
    /// Credential cert
    pub cert: String,
    /// Credential key
    pub key: String,
}

/// The communication protocol to use when sending data.
#[derive(Clone, Copy, Debug)]
pub enum Protocol {
    /// GRPC protocol
    Grpc,
    // TODO add support for other protocols
    // HttpJson,
    // HttpProto,
}

/// The compression algorithm to use when sending data.
#[derive(Clone, Copy, Debug)]
#[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
pub enum Compression {
    /// Compresses data using gzip.
    Gzip,
}

#[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
impl Into<grpcio::CompressionAlgorithms> for Compression {
    fn into(self) -> grpcio::CompressionAlgorithms {
        match self {
            Compression::Gzip => grpcio::CompressionAlgorithms::GRPC_COMPRESS_GZIP,
        }
    }
}

const DEFAULT_OTLP_PORT: u16 = 4317;

impl Default for ExporterConfig {
    #[cfg(feature = "tonic")]
    fn default() -> Self {
        ExporterConfig {
            endpoint: format!("localhost:{}", DEFAULT_OTLP_PORT),
            protocol: Protocol::Grpc,
            #[cfg(all(feature = "tonic", feature = "tls"))]
            tls_config: None,
            metadata: None,
            timeout: Duration::from_secs(60),
            #[cfg(not(feature = "async"))]
            runtime: None,
        }
    }

    #[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
    fn default() -> Self {
        ExporterConfig {
            endpoint: format!("localhost:{}", DEFAULT_OTLP_PORT),
            protocol: Protocol::Grpc,
            credentials: None,
            headers: None,
            compression: None,
            timeout: Duration::from_secs(60),
            completion_queue_count: 2,
        }
    }
}

impl Default for Exporter {
    /// Return a Span Exporter with the default configuration
    #[cfg(feature = "tonic")]
    fn default() -> Self {
        let config: ExporterConfig = ExporterConfig::default();

        let endpoint = Channel::from_shared(config.endpoint).unwrap();

        let channel = endpoint.timeout(config.timeout).connect_lazy().unwrap();

        Exporter {
            trace_exporter: TraceServiceClient::new(channel),
            timeout: config.timeout,
            metadata: config.metadata,
            #[cfg(not(feature = "async"))]
            runtime: config.runtime.unwrap_or_else(|| {
                tokio::runtime::Builder::new()
                    .basic_scheduler()
                    .enable_all()
                    .build()
                    .unwrap()
            }),
        }
    }

    /// Return a Span Exporter with the default configuration
    #[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
    fn default() -> Self {
        let config: ExporterConfig = ExporterConfig::default();

        let channel: Channel =
            ChannelBuilder::new(Arc::new(Environment::new(config.completion_queue_count)))
                .connect(config.endpoint.as_str());

        Exporter {
            trace_exporter: TraceServiceClient::new(channel),
            timeout: config.timeout,
            headers: None,
        }
    }
}

impl Debug for Exporter {
    #[cfg(feature = "tonic")]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Exporter")
            .field("metadata", &self.metadata)
            .field("timeout", &self.timeout)
            .field("trace_exporter", &"TraceServiceClient")
            .finish()
    }

    #[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Exporter")
            .field("headers", &self.headers)
            .field("timeout", &self.timeout)
            .field("trace_exporter", &"TraceServiceClient")
            .finish()
    }
}

impl Exporter {
    /// Builds a new span exporter with the given configuration
    #[cfg(feature = "tonic")]
    pub fn new(config: ExporterConfig) -> Result<Self, crate::Error> {
        let endpoint = Channel::from_shared(config.endpoint)?;

        #[cfg(all(feature = "tonic", feature = "tls"))]
        let channel = match config.tls_config {
            Some(tls_config) => endpoint.tls_config(tls_config)?,
            None => endpoint,
        }
        .timeout(config.timeout)
        .connect_lazy()?;

        #[cfg(not(feature = "tls"))]
        let channel = endpoint.timeout(config.timeout).connect_lazy()?;

        let client = match config.metadata.to_owned() {
            None => TraceServiceClient::new(channel),
            Some(metadata) => {
                TraceServiceClient::with_interceptor(channel, move |mut req: Request<()>| {
                    for key_and_value in metadata.iter() {
                        match key_and_value {
                            KeyAndValueRef::Ascii(key, value) => {
                                req.metadata_mut().append(key, value.to_owned())
                            }
                            KeyAndValueRef::Binary(key, value) => {
                                req.metadata_mut().append_bin(key, value.to_owned())
                            }
                        };
                    }

                    Ok(req)
                })
            }
        };

        Ok(Exporter {
            trace_exporter: client,
            timeout: config.timeout,
            metadata: config.metadata,
            #[cfg(not(feature = "async"))]
            runtime: config.runtime.unwrap_or_else(|| {
                tokio::runtime::Builder::new()
                    .basic_scheduler()
                    .enable_all()
                    .build()
                    .unwrap()
            }),
        })
    }

    /// Builds a new span exporter with the given configuration
    #[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
    pub fn new(config: ExporterConfig) -> Self {
        let mut builder: ChannelBuilder =
            ChannelBuilder::new(Arc::new(Environment::new(config.completion_queue_count)));

        if let Some(compression) = config.compression {
            builder = builder.default_compression_algorithm(compression.into());
        }

        let channel: Channel = match config.credentials {
            None => builder.connect(config.endpoint.as_str()),
            Some(credentials) => builder.secure_connect(
                config.endpoint.as_str(),
                ChannelCredentialsBuilder::new()
                    .cert(credentials.cert.into(), credentials.key.into())
                    .build(),
            ),
        };

        Exporter {
            trace_exporter: TraceServiceClient::new(channel),
            timeout: config.timeout,
            headers: config.headers,
        }
    }
}

#[async_trait]
impl SpanExporter for Exporter {
    #[cfg(feature = "tonic")]
    async fn export(&mut self, batch: Vec<SpanData>) -> ExportResult {
        let request = Request::new(ExportTraceServiceRequest {
            resource_spans: batch.into_iter().map(Into::into).collect(),
        });

        #[cfg(feature = "async")]
        self.trace_exporter
            .to_owned()
            .export(request)
            .await
            .map_err::<crate::Error, _>(Into::into)?;

        #[cfg(not(feature = "async"))]
        self.runtime
            .block_on(self.trace_exporter.to_owned().export(request))
            .map_err::<crate::Error, _>(Into::into)?;

        Ok(())
    }

    #[cfg(all(feature = "grpc-sys", not(feature = "tonic")))]
    async fn export(&mut self, batch: Vec<SpanData>) -> ExportResult {
        let request = ExportTraceServiceRequest {
            resource_spans: RepeatedField::from_vec(batch.into_iter().map(Into::into).collect()),
            unknown_fields: Default::default(),
            cached_size: Default::default(),
        };

        let mut call_options = CallOption::default().timeout(self.timeout);

        if let Some(headers) = self.headers.clone() {
            let mut metadata_builder: MetadataBuilder = MetadataBuilder::new();

            for (key, value) in headers {
                let _ = metadata_builder.add_str(key.as_str(), value.as_str());
            }

            call_options = call_options.headers(metadata_builder.build());
        }

        let receiver = self
            .trace_exporter
            .export_async_opt(&request, call_options)
            .map_err::<crate::Error, _>(Into::into)?;
        receiver.await.map_err::<crate::Error, _>(Into::into)?;
        Ok(())
    }
}
