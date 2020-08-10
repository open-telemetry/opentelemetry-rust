//! # OTLP - Span Exporter
//!
//! Defines a [SpanExporter] to send trace data via the OpenTelemetry Protocol (OTLP)
use crate::proto::trace_service::ExportTraceServiceRequest;
use crate::proto::trace_service_grpc::TraceServiceClient;
use grpcio::{
    CallOption, Channel, ChannelBuilder, ChannelCredentialsBuilder, Environment, MetadataBuilder,
};
use opentelemetry::exporter::trace::ExportResult::{FailedNotRetryable, Success};
use opentelemetry::exporter::trace::{ExportResult, SpanData, SpanExporter};
use protobuf::RepeatedField;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

pub struct Exporter {
    headers: Option<HashMap<String, String>>,
    timeout: Duration,
    trace_exporter: TraceServiceClient,
}

#[derive(Debug)]
pub struct ExporterConfig {
    pub endpoint: String,
    pub protocol: Protocol,
    pub credentials: Option<Credentials>,
    pub headers: Option<HashMap<String, String>>,
    pub compression: Option<Compression>,
    pub timeout: Duration,
    pub completion_queue_count: usize,
}

#[derive(Debug)]
pub struct Credentials {
    pub cert: String,
    pub key: String,
}

#[derive(Clone, Copy, Debug)]
pub enum Protocol {
    Grpc,
    // TODO add support for other protocols
    // HttpJson,
    // HttpProto,
}

#[derive(Clone, Copy, Debug)]
pub enum Compression {
    Gzip,
}

impl Into<grpcio::CompressionAlgorithms> for Compression {
    fn into(self) -> grpcio::CompressionAlgorithms {
        match self {
            Compression::Gzip => grpcio::CompressionAlgorithms::GRPC_COMPRESS_GZIP,
        }
    }
}

impl Default for ExporterConfig {
    fn default() -> Self {
        ExporterConfig {
            endpoint: String::from("localhost:55680"),
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Exporter")
            .field("metrics_exporter", &String::from("MetricsServiceClient"))
            .field("trace_exporter", &String::from("TraceServiceClient"))
            .finish()
    }
}

impl Exporter {
    /// Builds a new span exporter with the given configuration
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

impl SpanExporter for Exporter {
    fn export(&self, batch: Vec<Arc<SpanData>>) -> ExportResult {
        let request = ExportTraceServiceRequest {
            resource_spans: RepeatedField::from_vec(
                batch.into_iter().map(|span| span.into()).collect(),
            ),
            unknown_fields: Default::default(),
            cached_size: Default::default(),
        };

        let mut call_options: CallOption = CallOption::default().timeout(self.timeout);

        if let Some(headers) = self.headers.clone() {
            let mut metadata_builder: MetadataBuilder = MetadataBuilder::new();

            for (key, value) in headers {
                let _ = metadata_builder.add_str(key.as_str(), value.as_str());
            }

            call_options = call_options.headers(metadata_builder.build());
        }

        match self.trace_exporter.export_opt(&request, call_options) {
            Ok(_) => Success,
            Err(_) => FailedNotRetryable,
        }
    }

    /// Unimplemented for now. Channel will shutdown on drop
    fn shutdown(&self) {}
}
