use crate::ExportConfig;
use http::HeaderMap;
use tonic::metadata::MetadataMap;
#[cfg(feature = "tls")]
use tonic::transport::ClientTlsConfig;

/// Configuration for [tonic]
///
/// [tonic]: https://github.com/hyperium/tonic
#[derive(Debug, Default)]
pub struct TonicConfig {
    /// Custom metadata entries to send to the collector.
    pub metadata: Option<MetadataMap>,

    /// TLS settings for the collector endpoint.
    #[cfg(feature = "tls")]
    pub tls_config: Option<ClientTlsConfig>,
}

/// Build a trace exporter that uses [tonic] as grpc layer and opentelemetry protocol.
///
/// It allows users to
/// - add additional metadata
/// - set tls config(with `tls` feature enabled)
/// - bring custom [channel]
///
/// [tonic]: <https://github.com/hyperium/tonic>
/// [channel]: tonic::transport::Channel
#[derive(Debug)]
pub struct TonicExporterBuilder {
    pub(crate) exporter_config: ExportConfig,
    pub(crate) tonic_config: TonicConfig,
    pub(crate) channel: Option<tonic::transport::Channel>,
}

impl Default for TonicExporterBuilder {
    fn default() -> Self {
        let mut tonic_config = TonicConfig::default();
        let mut headers: HeaderMap = HeaderMap::new();
        headers.insert(
            "User-Agent",
            format!("OTel OTLP Exporter Rust/{}", env!("CARGO_PKG_VERSION"))
                .parse()
                .unwrap(),
        );
        tonic_config.metadata = Some(MetadataMap::from_headers(headers));

        TonicExporterBuilder {
            exporter_config: ExportConfig::default(),
            tonic_config,
            channel: Option::default(),
        }
    }
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
        // extending metadatamaps is harder than just casting back/forth
        let incoming_headers = metadata.into_headers();
        let mut existing_headers = self
            .tonic_config
            .metadata
            .unwrap_or_default()
            .into_headers();
        existing_headers.extend(incoming_headers.into_iter());

        self.tonic_config.metadata = Some(MetadataMap::from_headers(existing_headers));
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
