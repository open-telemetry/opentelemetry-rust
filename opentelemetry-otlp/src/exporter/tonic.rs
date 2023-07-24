use crate::exporter::Compression;
use crate::{ExportConfig, OTEL_EXPORTER_OTLP_COMPRESSION};
use std::fmt::{Debug, Formatter};
use tonic::codec::CompressionEncoding;
use tonic::metadata::MetadataMap;
#[cfg(feature = "tls")]
use tonic::transport::ClientTlsConfig;

use super::default_headers;

/// Configuration for [tonic]
///
/// [tonic]: https://github.com/hyperium/tonic
#[derive(Debug, Default)]
#[non_exhaustive]
pub struct TonicConfig {
    /// Custom metadata entries to send to the collector.
    pub metadata: Option<MetadataMap>,

    /// TLS settings for the collector endpoint.
    #[cfg(feature = "tls")]
    pub tls_config: Option<ClientTlsConfig>,

    /// The compression algorithm to use when communicating with the collector.
    pub compression: Option<Compression>,
}

impl TryFrom<Compression> for tonic::codec::CompressionEncoding {
    type Error = crate::Error;

    fn try_from(value: Compression) -> Result<Self, Self::Error> {
        match value {
            #[cfg(feature = "gzip-tonic")]
            Compression::Gzip => Ok(tonic::codec::CompressionEncoding::Gzip),
            #[cfg(not(feature = "gzip-tonic"))]
            Compression::Gzip => Err(crate::Error::UnsupportedCompressionAlgorithm(
                value.to_string(),
            )),
        }
    }
}

pub(crate) fn resolve_compression(
    tonic_config: &TonicConfig,
    env_override: &'static str,
) -> Result<Option<CompressionEncoding>, crate::Error> {
    if let Some(compression) = tonic_config.compression {
        Ok(Some(compression.try_into()?))
    } else if let Ok(compression) = std::env::var(env_override) {
        Ok(Some(compression.parse::<Compression>()?.try_into()?))
    } else if let Ok(compression) = std::env::var(OTEL_EXPORTER_OTLP_COMPRESSION) {
        Ok(Some(compression.parse::<Compression>()?.try_into()?))
    } else {
        Ok(None)
    }
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
    pub(crate) interceptor: Option<BoxInterceptor>,
}

pub(crate) struct BoxInterceptor(Box<dyn tonic::service::Interceptor + Send>);
impl tonic::service::Interceptor for BoxInterceptor {
    fn call(&mut self, request: tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> {
        self.0.call(request)
    }
}

impl Debug for BoxInterceptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "BoxInterceptor(..)")
    }
}

impl Default for TonicExporterBuilder {
    fn default() -> Self {
        let tonic_config = TonicConfig {
            metadata: Some(MetadataMap::from_headers(
                (&default_headers())
                    .try_into()
                    .expect("Invalid tonic headers"),
            )),
            #[cfg(feature = "tls")]
            tls_config: None,
            compression: None,
        };

        TonicExporterBuilder {
            exporter_config: ExportConfig::default(),
            tonic_config,
            channel: Option::default(),
            interceptor: Option::default(),
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
        // extending metadata maps is harder than just casting back/forth
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

    /// Set the compression algorithm to use when communicating with the collector.
    pub fn with_compression(mut self, compression: Compression) -> Self {
        self.tonic_config.compression = Some(compression);
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

    /// Use a custom `interceptor` to modify each outbound request.
    /// this can be used to modify the grpc metadata, for example
    /// to inject auth tokens.
    pub fn with_interceptor<I>(mut self, interceptor: I) -> Self
    where
        I: tonic::service::Interceptor + Send + 'static,
    {
        self.interceptor = Some(BoxInterceptor(Box::new(interceptor)));
        self
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "gzip-tonic")]
    use crate::exporter::Compression;
    use crate::TonicExporterBuilder;
    use tonic::metadata::{MetadataMap, MetadataValue};

    #[test]
    fn test_with_metadata() {
        // metadata should merge with the current one with priority instead of just replacing it
        let mut metadata = MetadataMap::new();
        metadata.insert("foo", "bar".parse().unwrap());
        let builder = TonicExporterBuilder::default().with_metadata(metadata);
        let result = builder.tonic_config.metadata.unwrap();
        let foo = result.get("foo").unwrap();
        assert_eq!(foo, &MetadataValue::try_from("bar").unwrap());
        assert!(result.get("User-Agent").is_some());

        // metadata should override entries with the same key in the default one
        let mut metadata = MetadataMap::new();
        metadata.insert("user-agent", "baz".parse().unwrap());
        let builder = TonicExporterBuilder::default().with_metadata(metadata);
        let result = builder.tonic_config.metadata.unwrap();
        assert_eq!(
            result.get("User-Agent").unwrap(),
            &MetadataValue::try_from("baz").unwrap()
        );
        assert_eq!(
            result.len(),
            TonicExporterBuilder::default()
                .tonic_config
                .metadata
                .unwrap()
                .len()
        );
    }

    #[test]
    #[cfg(feature = "gzip-tonic")]
    fn test_with_compression() {
        // metadata should merge with the current one with priority instead of just replacing it
        let mut metadata = MetadataMap::new();
        metadata.insert("foo", "bar".parse().unwrap());
        let builder = TonicExporterBuilder::default().with_compression(Compression::Gzip);
        assert_eq!(builder.tonic_config.compression.unwrap(), Compression::Gzip);
    }
}
