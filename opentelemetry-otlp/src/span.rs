//! # OTLP - Span Exporter
//!
//! Defines a [SpanExporter] to send trace data via the OpenTelemetry Protocol (OTLP)

#[cfg(feature = "tonic")]
use crate::proto::collector::trace::v1::{
    trace_service_client::TraceServiceClient as TonicTraceServiceClient,
    ExportTraceServiceRequest as TonicRequest,
};

#[cfg(feature = "http-proto")]
use crate::proto::prost::collector::trace::v1::ExportTraceServiceRequest as ProstRequest;

#[cfg(feature = "tonic")]
use tonic::{
    metadata::{KeyAndValueRef, MetadataMap},
    transport::Channel as TonicChannel,
    Request,
};

#[cfg(all(feature = "tonic", feature = "tls"))]
use tonic::transport::ClientTlsConfig;

#[cfg(feature = "grpc-sys")]
use crate::proto::grpcio::trace_service::ExportTraceServiceRequest as GrpcRequest;

#[cfg(feature = "grpc-sys")]
use crate::proto::grpcio::trace_service_grpc::TraceServiceClient as GrpcioTraceServiceClient;

#[cfg(feature = "grpc-sys")]
use grpcio::{
    CallOption, Channel as GrpcChannel, ChannelBuilder, ChannelCredentialsBuilder, Environment,
    MetadataBuilder,
};

#[cfg(feature = "grpc-sys")]
use protobuf::RepeatedField;

#[cfg(feature = "http-proto")]
use prost::Message;
#[cfg(feature = "http-proto")]
use std::convert::TryFrom;

use async_trait::async_trait;

#[cfg(any(feature = "grpc-sys", feature = "http-proto"))]
use std::collections::HashMap;

use std::fmt;
use std::fmt::Debug;

#[cfg(feature = "grpc-sys")]
use std::sync::Arc;

use crate::{Protocol, OTEL_EXPORTER_OTLP_ENDPOINT_DEFAULT, OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT};
use opentelemetry::sdk::export::trace::{ExportResult, SpanData};
use std::time::Duration;

#[cfg(feature = "http-proto")]
use http::{
    header::{HeaderName, HeaderValue, CONTENT_TYPE},
    Method, Uri,
};
#[cfg(feature = "http-proto")]
use opentelemetry_http::HttpClient;

/// Exporter that sends data in OTLP format.
pub enum SpanExporter {
    #[cfg(feature = "tonic")]
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
        trace_exporter: Option<Box<dyn HttpClient>>,
    },
}

/// Configuration for the OTLP exporter.
#[derive(Debug)]
pub struct ExportConfig {
    /// The address of the OTLP collector. If not set, the default address is used.
    pub endpoint: String,

    /// The protocol to use when communicating with the collector.
    pub protocol: Protocol,

    /// The timeout to the collector.
    pub timeout: Duration,
}

/// Configuration for [tonic]
///
/// [tonic]: https://github.com/hyperium/tonic
#[cfg(feature = "tonic")]
#[derive(Debug)]
pub struct TonicConfig {
    /// Custom metadata entries to send to the collector.
    pub metadata: Option<MetadataMap>,

    /// TLS settings for the collector endpoint.
    #[cfg(feature = "tls")]
    pub tls_config: Option<ClientTlsConfig>,
}

#[cfg(feature = "tonic")]
impl Default for TonicConfig {
    fn default() -> Self {
        TonicConfig {
            #[cfg(feature = "tls")]
            tls_config: None,
            metadata: None,
        }
    }
}

/// Configuration of grpcio
#[cfg(feature = "grpc-sys")]
#[derive(Debug)]
pub struct GrpcioConfig {
    /// The credentials to use when communicating with the collector.
    pub credentials: Option<Credentials>,

    /// Additional headers to send to the collector.
    pub headers: Option<HashMap<String, String>>,

    /// The compression algorithm to use when communicating with the collector.
    pub compression: Option<Compression>,

    /// Use TLS without any specific certificate pinning.
    pub use_tls: Option<bool>,

    /// The number of GRPC worker threads to poll queues.
    pub completion_queue_count: usize,
}

#[cfg(feature = "grpc-sys")]
impl Default for GrpcioConfig {
    fn default() -> Self {
        GrpcioConfig {
            credentials: None,
            headers: None,
            compression: None,
            use_tls: None,
            completion_queue_count: 2,
        }
    }
}

/// Credential configuration for authenticated requests.
#[derive(Debug)]
#[cfg(feature = "grpc-sys")]
pub struct Credentials {
    /// Credential cert
    pub cert: String,
    /// Credential key
    pub key: String,
}

/// The compression algorithm to use when sending data.
#[derive(Clone, Copy, Debug)]
#[cfg(feature = "grpc-sys")]
pub enum Compression {
    /// Compresses data using gzip.
    Gzip,
}

#[cfg(feature = "grpc-sys")]
impl From<Compression> for grpcio::CompressionAlgorithms {
    fn from(compression: Compression) -> Self {
        match compression {
            Compression::Gzip => grpcio::CompressionAlgorithms::GRPC_COMPRESS_GZIP,
        }
    }
}

/// Configuration of the http transport
#[cfg(feature = "http-proto")]
#[derive(Debug)]
pub struct HttpConfig {
    /// Select the HTTP client
    pub client: Option<Box<dyn HttpClient>>,

    /// Additional headers to send to the collector.
    pub headers: Option<HashMap<String, String>>,
}

#[cfg(feature = "http-proto")]
impl Default for HttpConfig {
    fn default() -> Self {
        HttpConfig {
            #[cfg(feature = "reqwest-blocking-client")]
            client: Some(Box::new(reqwest::blocking::Client::new())),
            #[cfg(all(
                not(feature = "reqwest-blocking-client"),
                not(feature = "surf-client"),
                feature = "reqwest-client"
            ))]
            client: Some(Box::new(reqwest::Client::new())),
            #[cfg(all(
                not(feature = "reqwest-client"),
                not(feature = "reqwest-blocking-client"),
                feature = "surf-client"
            ))]
            client: Some(Box::new(surf::Client::new())),
            #[cfg(all(
                not(feature = "reqwest-client"),
                not(feature = "surf-client"),
                not(feature = "reqwest-blocking-client")
            ))]
            client: None,
            headers: None,
        }
    }
}

impl Default for ExportConfig {
    fn default() -> Self {
        ExportConfig {
            endpoint: OTEL_EXPORTER_OTLP_ENDPOINT_DEFAULT.to_string(),
            protocol: Protocol::Grpc,
            timeout: Duration::from_secs(OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT),
        }
    }
}

impl Debug for SpanExporter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "tonic")]
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
    #[cfg(feature = "tonic")]
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
        .connect_lazy()?;

        #[cfg(not(feature = "tls"))]
        let channel = endpoint.timeout(config.timeout).connect_lazy()?;

        SpanExporter::from_tonic_channel(config, tonic_config, channel)
    }

    /// Builds a new span exporter with given tonic channel.
    ///
    /// This allows users to bring their own custom channel like UDS.
    /// However, users MUST make sure the [`ExporterConfig::timeout`] is
    /// the same as the channel's timeout.
    ///
    /// [`ExporterConfig::timeout`]: crate::span::ExporterConfig::timeout
    #[cfg(feature = "tonic")]
    pub fn from_tonic_channel(
        config: ExportConfig,
        tonic_config: TonicConfig,
        channel: tonic::transport::Channel,
    ) -> Result<Self, crate::Error> {
        let client = match tonic_config.metadata.to_owned() {
            None => TonicTraceServiceClient::new(channel),
            Some(metadata) => {
                TonicTraceServiceClient::with_interceptor(channel, move |mut req: Request<()>| {
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

        Ok(SpanExporter::Tonic {
            timeout: config.timeout,
            metadata: tonic_config.metadata,
            trace_exporter: client,
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

#[async_trait]
impl opentelemetry::sdk::export::trace::SpanExporter for SpanExporter {
    async fn export(&mut self, batch: Vec<SpanData>) -> ExportResult {
        match self {
            #[cfg(feature = "grpc-sys")]
            SpanExporter::Grpcio {
                timeout,
                headers,
                trace_exporter,
            } => {
                let request = GrpcRequest {
                    resource_spans: RepeatedField::from_vec(
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

                let receiver = trace_exporter
                    .export_async_opt(&request, call_options)
                    .map_err::<crate::Error, _>(Into::into)?;
                receiver.await.map_err::<crate::Error, _>(Into::into)?;
                Ok(())
            }

            #[cfg(feature = "tonic")]
            SpanExporter::Tonic { trace_exporter, .. } => {
                let request = Request::new(TonicRequest {
                    resource_spans: batch.into_iter().map(Into::into).collect(),
                });

                trace_exporter
                    .to_owned()
                    .export(request)
                    .await
                    .map_err::<crate::Error, _>(Into::into)?;

                Ok(())
            }

            #[cfg(feature = "http-proto")]
            SpanExporter::Http {
                trace_exporter,
                collector_endpoint,
                headers,
                ..
            } => {
                let req = ProstRequest {
                    resource_spans: batch.into_iter().map(Into::into).collect(),
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

                if let Some(client) = trace_exporter {
                    client.send(request).await?;
                    Ok(())
                } else {
                    Err(crate::Error::NoHttpClient.into())
                }
            }
        }
    }
}
