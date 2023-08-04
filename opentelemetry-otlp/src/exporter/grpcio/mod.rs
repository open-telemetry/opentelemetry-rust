use std::collections::HashMap;
use std::sync::Arc;

use grpcio::{Channel, ChannelBuilder, ChannelCredentialsBuilder, Environment};
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

use crate::exporter::Compression;
use crate::ExportConfig;

use super::default_headers;

#[cfg(feature = "logs")]
mod logs;

#[cfg(feature = "trace")]
mod trace;

/// Configuration of grpcio
#[derive(Debug)]
#[non_exhaustive]
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

impl Default for GrpcioConfig {
    fn default() -> Self {
        GrpcioConfig {
            credentials: None,
            headers: Some(default_headers()),
            compression: None,
            use_tls: None,
            completion_queue_count: 2,
        }
    }
}

/// Credential configuration for authenticated requests.
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[derive(Clone, Debug)]
pub struct Credentials {
    /// Credential cert
    pub cert: String,
    /// Credential key
    pub key: String,
}

impl From<Compression> for grpcio::CompressionAlgorithms {
    fn from(compression: Compression) -> Self {
        match compression {
            Compression::Gzip => grpcio::CompressionAlgorithms::GRPC_COMPRESS_GZIP,
        }
    }
}

/// Configuration for the [grpcio] OTLP GRPC exporter.
///
/// It allows you to
/// - setup credentials
/// - add additional headers
/// - config compression
/// - select whether to use TLS
/// - set the number of GRPC worker threads to poll queues
///
/// [grpcio]: https://github.com/tikv/grpc-rs
///
/// ## Examples
///
/// ```no_run
/// # #[cfg(feature="metrics")]
/// use opentelemetry_sdk::metrics::reader::{
///     DefaultAggregationSelector, DefaultTemporalitySelector,
/// };
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a span exporter you can use to when configuring tracer providers
/// # #[cfg(feature="trace")]
/// let span_exporter = opentelemetry_otlp::new_exporter().grpcio().build_span_exporter()?;
///
/// // Create a log exporter you can use when configuring logger providers
/// # #[cfg(feature="logs")]
/// let log_exporter = opentelemetry_otlp::new_exporter().grpcio().build_log_exporter()?;
/// # Ok(())
/// # }
/// ```
#[derive(Default, Debug)]
pub struct GrpcioExporterBuilder {
    pub(crate) exporter_config: ExportConfig,
    pub(crate) grpcio_config: GrpcioConfig,
}

impl GrpcioExporterBuilder {
    /// Set the credentials to use when communicating with the collector.
    pub fn with_credentials(mut self, credentials: Credentials) -> Self {
        self.grpcio_config.credentials = Some(credentials);
        self
    }

    /// Set additional headers to send to the collector.
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        let mut inst_headers = self.grpcio_config.headers.unwrap_or_default();
        inst_headers.extend(headers.into_iter());
        self.grpcio_config.headers = Some(inst_headers);
        self
    }

    /// Set the compression algorithm to use when communicating with the collector.
    pub fn with_compression(mut self, compression: Compression) -> Self {
        self.grpcio_config.compression = Some(compression);
        self
    }

    /// Enable TLS without any certificate pinning.
    pub fn with_tls(mut self, use_tls: bool) -> Self {
        self.grpcio_config.use_tls = Some(use_tls);
        self
    }

    /// Set the number of GRPC worker threads to poll queues.
    pub fn with_completion_queue_count(mut self, count: usize) -> Self {
        self.grpcio_config.completion_queue_count = count;
        self
    }

    fn build_channel(&mut self) -> Result<Channel, crate::Error> {
        let mut builder: ChannelBuilder = ChannelBuilder::new(Arc::new(Environment::new(
            self.grpcio_config.completion_queue_count,
        )));

        if let Some(compression) = self.grpcio_config.compression {
            builder = builder.default_compression_algorithm(compression.into());
        }

        let channel = match (
            self.grpcio_config.credentials.take(),
            self.grpcio_config.use_tls.take(),
        ) {
            (None, Some(true)) => builder
                .set_credentials(ChannelCredentialsBuilder::new().build())
                .connect(self.exporter_config.endpoint.as_str()),
            (None, _) => builder.connect(self.exporter_config.endpoint.as_str()),
            (Some(credentials), _) => builder
                .set_credentials(
                    ChannelCredentialsBuilder::new()
                        .cert(credentials.cert.into(), credentials.key.into())
                        .build(),
                )
                .connect(self.exporter_config.endpoint.as_str()),
        };

        Ok(channel)
    }

    /// Create a new span exporter with the current configuration
    #[cfg(feature = "trace")]
    pub fn build_span_exporter(
        mut self,
    ) -> Result<crate::SpanExporter, opentelemetry_api::trace::TraceError> {
        use self::trace::GrpcioTraceClient;
        use opentelemetry_proto::grpcio::trace_service_grpc::TraceServiceClient;

        let channel = self.build_channel()?;

        let client = GrpcioTraceClient::new(
            TraceServiceClient::new(channel),
            self.exporter_config.timeout,
            self.grpcio_config.headers.unwrap_or_default(),
        );

        Ok(crate::SpanExporter::new(client))
    }

    #[cfg(feature = "logs")]
    /// Builds a new log exporter with the given configuration
    pub fn build_log_exporter(
        mut self,
    ) -> Result<crate::logs::LogExporter, opentelemetry_api::logs::LogError> {
        use self::logs::GrpcioLogsClient;
        use opentelemetry_proto::grpcio::logs_service_grpc::LogsServiceClient;

        let channel = self.build_channel()?;

        let client = GrpcioLogsClient::new(
            LogsServiceClient::new(channel),
            self.exporter_config.timeout,
            self.grpcio_config.headers.unwrap_or_default(),
        );

        Ok(crate::logs::LogExporter::new(client))
    }
}

#[cfg(test)]
mod tests {
    use crate::GrpcioExporterBuilder;
    use std::collections::HashMap;

    #[test]
    fn test_with_headers() {
        // metadata should merge with the current one with priority instead of just replacing it
        let mut headers = HashMap::new();
        headers.insert("key".to_string(), "value".to_string());
        let builder = GrpcioExporterBuilder::default().with_headers(headers);
        let result = builder.grpcio_config.headers.unwrap();
        assert_eq!(result.get("key").unwrap(), "value");
        assert!(result.get("User-Agent").is_some());

        // metadata should override entries with the same key in the default one
        let mut headers = HashMap::new();
        headers.insert("User-Agent".to_string(), "baz".to_string());
        let builder = GrpcioExporterBuilder::default().with_headers(headers);
        let result = builder.grpcio_config.headers.unwrap();
        assert_eq!(result.get("User-Agent").unwrap(), "baz");
        assert_eq!(
            result.len(),
            GrpcioExporterBuilder::default()
                .grpcio_config
                .headers
                .unwrap()
                .len()
        );
    }
}
