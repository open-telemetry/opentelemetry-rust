use tonic::metadata::MetadataMap;
use tonic::transport::ClientTlsConfig;
use crate::ExportConfig;

/// Configuration for [tonic]
///
/// [tonic]: https://github.com/hyperium/tonic
#[derive(Debug)]
pub struct TonicConfig {
    /// Custom metadata entries to send to the collector.
    pub metadata: Option<MetadataMap>,

    /// TLS settings for the collector endpoint.
    #[cfg(feature = "tls")]
    pub tls_config: Option<ClientTlsConfig>,
}

impl Default for TonicConfig {
    fn default() -> Self {
        TonicConfig {
            #[cfg(feature = "tls")]
            tls_config: None,
            metadata: None,
        }
    }
}

/// Build a trace exporter that uses [tonic] as grpc layer and opentelemetry protocol.
///
/// It provides methods to config tonic.
/// [tonic]: https://github.com/hyperium/tonic
#[derive(Default, Debug)]
pub struct TonicExporterBuilder {
    pub(crate) exporter_config: ExportConfig,
    pub(crate) tonic_config: TonicConfig,
    pub(crate) channel: Option<tonic::transport::Channel>,
}

impl TonicExporterBuilder {
    /// Set the TLS settings for the collector endpoint.
    #[cfg(feature = "tls")]
    pub fn with_tls_config(mut self, tls_config: ClientTlsConfig) -> Self {
        self.tonic_config.tls_config = Some(tls_config);
        self
    }

    /// Set custom metadata entries to send to the collector.
    pub fn with_metadata(mut self, metadata: MetadataMap) -> Self {
        self.tonic_config.metadata = Some(metadata);
        self
    }

    /// Use `channel` as tonic's transport channel.
    /// this will override tls config and should only be used
    /// when working with non-HTTP transports.
    ///
    /// Users MUST make sure the [`ExportConfig::timeout`] is
    /// the same as the channel's timeout.
    pub fn with_channel(mut self, channel: tonic::transport::Channel) -> Self {
        self.channel = Some(channel);
        self
    }
}