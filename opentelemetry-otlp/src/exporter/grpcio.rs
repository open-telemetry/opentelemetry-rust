use crate::ExportConfig;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
        let mut headers: HashMap<String, String> = HashMap::new();
        headers.insert("User-Agent".to_string(), format!("OTel OLTP Exporter Rust/{}", env!("CARGO_PKG_VERSION")));
        GrpcioConfig {
            credentials: None,
            headers: Some(headers),
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

/// The compression algorithm to use when sending data.
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
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
/// It allows users to
/// - setup credentials
/// - add additional headers
/// - config compression
/// - select whether to use TLS
/// - set the number of GRPC worker threads to poll queues
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
}
