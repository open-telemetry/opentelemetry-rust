use crate::proto::trace_service::ExportTraceServiceRequest;
use crate::proto::trace_service_grpc::TraceServiceClient;
use grpcio::{Channel, ChannelBuilder, Environment};
use opentelemetry::exporter::trace::ExportResult::{FailedNotRetryable, Success};
use opentelemetry::exporter::trace::{ExportResult, SpanData, SpanExporter};
use protobuf::RepeatedField;
use std::fmt;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

pub struct Exporter {
    config: ExporterConfig,
    trace_exporter: TraceServiceClient,
}

#[derive(Debug)]
pub struct ExporterConfig {
    pub endpoint: String,
    pub protocol: Protocol,
    pub insecure: bool,
    pub certificate_file: Option<String>,
    pub headers: Option<String>,
    pub compression: Option<Compression>,
    pub timeout: Duration,
    pub completion_queue_count: usize,
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

impl Default for ExporterConfig {
    fn default() -> Self {
        ExporterConfig {
            endpoint: String::from("localhost:55680"),
            protocol: Protocol::Grpc,
            insecure: false,
            certificate_file: None,
            headers: None,
            compression: None,
            timeout: Duration::from_secs(60),
            completion_queue_count: 2,
        }
    }
}

impl Default for Exporter {
    fn default() -> Self {
        let config: ExporterConfig = ExporterConfig::default();

        let channel: Channel =
            ChannelBuilder::new(Arc::new(Environment::new(config.completion_queue_count)))
                .connect(config.endpoint.as_str());
        Exporter {
            trace_exporter: TraceServiceClient::new(channel),
            config,
        }
    }
}

impl Debug for Exporter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Exporter")
            .field("config", &self.config)
            .field("metrics_exporter", &String::from("MetricsServiceClient"))
            .field("trace_exporter", &String::from("TraceServiceClient"))
            .finish()
    }
}

impl Exporter {
    pub fn new(config: ExporterConfig) -> Self {
        let channel: Channel = ChannelBuilder::new(Arc::new(Environment::new(2_usize)))
            .connect(config.endpoint.as_str());
        Exporter {
            trace_exporter: TraceServiceClient::new(channel),
            config,
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

        match self.trace_exporter.export(&request) {
            Ok(_) => Success,
            Err(_) => FailedNotRetryable,
        }
    }

    /// Unimplemented for now. Channel will shutdown on drop
    fn shutdown(&self) {}
}
