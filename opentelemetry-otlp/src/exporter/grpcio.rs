use std::collections::HashMap;
use crate::ExportConfig;

/// Configuration of grpcio
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
pub struct Credentials {
    /// Credential cert
    pub cert: String,
    /// Credential key
    pub key: String,
}

/// The compression algorithm to use when sending data.
#[derive(Clone, Copy, Debug)]
pub enum Compression {
    /// Compresses data using gzip.
    Gzip,
}


impl From<Compression> for grpcio::CompressionAlgorithms {
    fn from(compression: Compression) -> Self {
        match compression {
            Compression::Gzip => grpcio::CompressionAlgorithms::GRPC_COMPRESS_GZIP,
        }
    }
}

/// Build a trace exporter that uses [grpcio] as grpc layer and opentelemetry protocol.
///
/// It provides methods to config grpcio.
///
///
/// [grpcio]: https://github.com/tikv/grpc-rs
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
        self.grpcio_config.headers = Some(headers);
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
}