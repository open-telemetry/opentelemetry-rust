use std::env;
use std::fmt::{Debug, Formatter};
use std::str::FromStr;
use std::time::Duration;

use http::{HeaderMap, HeaderName, HeaderValue};
use opentelemetry::otel_debug;
use tonic::codec::CompressionEncoding;
use tonic::metadata::{KeyAndValueRef, MetadataMap};
use tonic::service::Interceptor;
use tonic::transport::Channel;
#[cfg(feature = "tls")]
use tonic::transport::ClientTlsConfig;

use super::{default_headers, parse_header_string, OTEL_EXPORTER_OTLP_GRPC_ENDPOINT_DEFAULT};
use crate::exporter::Compression;
use crate::{
    ExportConfig, OTEL_EXPORTER_OTLP_COMPRESSION, OTEL_EXPORTER_OTLP_ENDPOINT,
    OTEL_EXPORTER_OTLP_HEADERS, OTEL_EXPORTER_OTLP_TIMEOUT,
};

#[cfg(feature = "logs")]
pub(crate) mod logs;

#[cfg(feature = "metrics")]
mod metrics;

#[cfg(feature = "trace")]
pub(crate) mod trace;

/// Configuration for [tonic]
///
/// [tonic]: https://github.com/hyperium/tonic
#[derive(Debug, Default)]
#[non_exhaustive]
pub struct TonicConfig {
    /// Custom metadata entries to send to the collector.
    pub(crate) metadata: Option<MetadataMap>,
    /// TLS settings for the collector endpoint.
    #[cfg(feature = "tls")]
    pub(crate) tls_config: Option<ClientTlsConfig>,
    /// The compression algorithm to use when communicating with the collector.
    pub(crate) compression: Option<Compression>,
    pub(crate) channel: Option<tonic::transport::Channel>,
    pub(crate) interceptor: Option<BoxInterceptor>,
}

impl TryFrom<Compression> for tonic::codec::CompressionEncoding {
    type Error = crate::Error;

    fn try_from(value: Compression) -> Result<Self, Self::Error> {
        match value {
            #[cfg(feature = "gzip-tonic")]
            Compression::Gzip => Ok(tonic::codec::CompressionEncoding::Gzip),
            #[cfg(not(feature = "gzip-tonic"))]
            Compression::Gzip => Err(crate::Error::FeatureRequiredForCompressionAlgorithm(
                "gzip-tonic",
                Compression::Gzip,
            )),
            #[cfg(feature = "zstd-tonic")]
            Compression::Zstd => Ok(tonic::codec::CompressionEncoding::Zstd),
            #[cfg(not(feature = "zstd-tonic"))]
            Compression::Zstd => Err(crate::Error::FeatureRequiredForCompressionAlgorithm(
                "zstd-tonic",
                Compression::Zstd,
            )),
        }
    }
}

/// Configuration for the [tonic] OTLP GRPC exporter.
///
/// It allows you to
/// - add additional metadata
/// - set tls config (via the  `tls` feature)
/// - specify custom [channel]s
///
/// [tonic]: <https://github.com/hyperium/tonic>
/// [channel]: tonic::transport::Channel
///
/// ## Examples
///
/// ```no_run
/// # #[cfg(feature="metrics")]
/// use opentelemetry_sdk::metrics::Temporality;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a span exporter you can use to when configuring tracer providers
/// # #[cfg(feature="trace")]
/// let span_exporter = opentelemetry_otlp::SpanExporter::builder().with_tonic().build()?;
///
/// // Create a metric exporter you can use when configuring meter providers
/// # #[cfg(feature="metrics")]
/// let metric_exporter = opentelemetry_otlp::MetricExporter::builder()
///     .with_tonic()
///     .with_temporality(Temporality::default())
///     .build()?;
///
/// // Create a log exporter you can use when configuring logger providers
/// # #[cfg(feature="logs")]
/// let log_exporter = opentelemetry_otlp::LogExporter::builder().with_tonic().build()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct TonicExporterBuilder {
    pub(crate) tonic_config: TonicConfig,
    pub(crate) exporter_config: ExportConfig,
}

pub(crate) struct BoxInterceptor(Box<dyn Interceptor + Send + Sync>);
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
        TonicExporterBuilder {
            tonic_config: TonicConfig {
                metadata: Some(MetadataMap::from_headers(
                    (&default_headers())
                        .try_into()
                        .expect("Invalid tonic headers"),
                )),
                #[cfg(feature = "tls")]
                tls_config: None,
                compression: None,
                channel: Option::default(),
                interceptor: Option::default(),
            },
            exporter_config: ExportConfig {
                protocol: crate::Protocol::Grpc,
                ..Default::default()
            },
        }
    }
}

impl TonicExporterBuilder {
    fn build_channel(
        self,
        signal_endpoint_var: &str,
        signal_timeout_var: &str,
        signal_compression_var: &str,
        signal_headers_var: &str,
    ) -> Result<(Channel, BoxInterceptor, Option<CompressionEncoding>), crate::Error> {
        let compression = self.resolve_compression(signal_compression_var)?;

        let (headers_from_env, headers_for_logging) = parse_headers_from_env(signal_headers_var);
        let metadata = merge_metadata_with_headers_from_env(
            self.tonic_config.metadata.unwrap_or_default(),
            headers_from_env,
        );

        let add_metadata = move |mut req: tonic::Request<()>| {
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
        };

        let interceptor = match self.tonic_config.interceptor {
            Some(mut interceptor) => {
                BoxInterceptor(Box::new(move |req| interceptor.call(add_metadata(req)?)))
            }
            None => BoxInterceptor(Box::new(add_metadata)),
        };

        // If a custom channel was provided, use that channel instead of creating one
        if let Some(channel) = self.tonic_config.channel {
            return Ok((channel, interceptor, compression));
        }

        let config = self.exporter_config;

        let endpoint = Self::resolve_endpoint(signal_endpoint_var, config.endpoint);

        // Used for logging the endpoint
        let endpoint_clone = endpoint.clone();

        let endpoint = Channel::from_shared(endpoint).map_err(crate::Error::from)?;
        let timeout = match env::var(signal_timeout_var)
            .ok()
            .or(env::var(OTEL_EXPORTER_OTLP_TIMEOUT).ok())
        {
            Some(val) => match val.parse() {
                Ok(seconds) => Duration::from_secs(seconds),
                Err(_) => config.timeout,
            },
            None => config.timeout,
        };

        #[cfg(feature = "tls")]
        let channel = match self.tonic_config.tls_config {
            Some(tls_config) => endpoint
                .tls_config(tls_config)
                .map_err(crate::Error::from)?,
            None => endpoint,
        }
        .timeout(timeout)
        .connect_lazy();

        #[cfg(not(feature = "tls"))]
        let channel = endpoint.timeout(timeout).connect_lazy();

        otel_debug!(name: "TonicChannelBuilt", endpoint = endpoint_clone, timeout_in_millisecs = timeout.as_millis(), compression = format!("{:?}", compression), headers = format!("{:?}", headers_for_logging));
        Ok((channel, interceptor, compression))
    }

    fn resolve_endpoint(default_endpoint_var: &str, provided_endpoint: Option<String>) -> String {
        // resolving endpoint string
        // grpc doesn't have a "path" like http(See https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-HTTP2.md)
        // the path of grpc calls are based on the protobuf service definition
        // so we won't append one for default grpc endpoints
        // If users for some reason want to use a custom path, they can use env var or builder to pass it
        match env::var(default_endpoint_var)
            .ok()
            .or(env::var(OTEL_EXPORTER_OTLP_ENDPOINT).ok())
        {
            Some(val) => val,
            None => {
                provided_endpoint.unwrap_or(OTEL_EXPORTER_OTLP_GRPC_ENDPOINT_DEFAULT.to_string())
            }
        }
    }

    fn resolve_compression(
        &self,
        env_override: &str,
    ) -> Result<Option<CompressionEncoding>, crate::Error> {
        if let Some(compression) = self.tonic_config.compression {
            Ok(Some(compression.try_into()?))
        } else if let Ok(compression) = env::var(env_override) {
            Ok(Some(compression.parse::<Compression>()?.try_into()?))
        } else if let Ok(compression) = env::var(OTEL_EXPORTER_OTLP_COMPRESSION) {
            Ok(Some(compression.parse::<Compression>()?.try_into()?))
        } else {
            Ok(None)
        }
    }

    /// Build a new tonic log exporter
    #[cfg(feature = "logs")]
    pub(crate) fn build_log_exporter(
        self,
    ) -> Result<crate::logs::LogExporter, opentelemetry_sdk::logs::LogError> {
        use crate::exporter::tonic::logs::TonicLogsClient;

        otel_debug!(name: "LogsTonicChannelBuilding");

        let (channel, interceptor, compression) = self.build_channel(
            crate::logs::OTEL_EXPORTER_OTLP_LOGS_ENDPOINT,
            crate::logs::OTEL_EXPORTER_OTLP_LOGS_TIMEOUT,
            crate::logs::OTEL_EXPORTER_OTLP_LOGS_COMPRESSION,
            crate::logs::OTEL_EXPORTER_OTLP_LOGS_HEADERS,
        )?;

        let client = TonicLogsClient::new(channel, interceptor, compression);

        Ok(crate::logs::LogExporter::from_tonic(client))
    }

    /// Build a new tonic metrics exporter
    #[cfg(feature = "metrics")]
    pub(crate) fn build_metrics_exporter(
        self,
        temporality: opentelemetry_sdk::metrics::Temporality,
    ) -> opentelemetry_sdk::metrics::MetricResult<crate::MetricExporter> {
        use crate::MetricExporter;
        use metrics::TonicMetricsClient;

        otel_debug!(name: "MetricsTonicChannelBuilding");

        let (channel, interceptor, compression) = self.build_channel(
            crate::metric::OTEL_EXPORTER_OTLP_METRICS_ENDPOINT,
            crate::metric::OTEL_EXPORTER_OTLP_METRICS_TIMEOUT,
            crate::metric::OTEL_EXPORTER_OTLP_METRICS_COMPRESSION,
            crate::metric::OTEL_EXPORTER_OTLP_METRICS_HEADERS,
        )?;

        let client = TonicMetricsClient::new(channel, interceptor, compression);

        Ok(MetricExporter::new(client, temporality))
    }

    /// Build a new tonic span exporter
    #[cfg(feature = "trace")]
    pub(crate) fn build_span_exporter(
        self,
    ) -> Result<crate::SpanExporter, opentelemetry::trace::TraceError> {
        use crate::exporter::tonic::trace::TonicTracesClient;

        otel_debug!(name: "TracesTonicChannelBuilding");

        let (channel, interceptor, compression) = self.build_channel(
            crate::span::OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
            crate::span::OTEL_EXPORTER_OTLP_TRACES_TIMEOUT,
            crate::span::OTEL_EXPORTER_OTLP_TRACES_COMPRESSION,
            crate::span::OTEL_EXPORTER_OTLP_TRACES_HEADERS,
        )?;

        let client = TonicTracesClient::new(channel, interceptor, compression);

        Ok(crate::SpanExporter::new(client))
    }
}

fn merge_metadata_with_headers_from_env(
    metadata: MetadataMap,
    headers_from_env: HeaderMap,
) -> MetadataMap {
    if headers_from_env.is_empty() {
        metadata
    } else {
        let mut existing_headers: HeaderMap = metadata.into_headers();
        existing_headers.extend(headers_from_env);

        MetadataMap::from_headers(existing_headers)
    }
}

fn parse_headers_from_env(signal_headers_var: &str) -> (HeaderMap, Vec<(String, String)>) {
    let mut headers = Vec::new();

    (
        env::var(signal_headers_var)
            .or_else(|_| env::var(OTEL_EXPORTER_OTLP_HEADERS))
            .map(|input| {
                parse_header_string(&input)
                    .filter_map(|(key, value)| {
                        headers.push((key.to_owned(), value.clone()));
                        Some((
                            HeaderName::from_str(key).ok()?,
                            HeaderValue::from_str(&value).ok()?,
                        ))
                    })
                    .collect::<HeaderMap>()
            })
            .unwrap_or_default(),
        headers,
    )
}

/// Expose interface for modifying [TonicConfig] fields within the exporter builders.
pub trait HasTonicConfig {
    /// Return a mutable reference to the export config within the exporter builders.
    fn tonic_config(&mut self) -> &mut TonicConfig;
}

/// Expose interface for modifying [TonicConfig] fields within the [TonicExporterBuilder].
impl HasTonicConfig for TonicExporterBuilder {
    fn tonic_config(&mut self) -> &mut TonicConfig {
        &mut self.tonic_config
    }
}

/// Expose methods to override [TonicConfig].
///
/// This trait will be implemented for every struct that implemented [`HasTonicConfig`] trait.
///
/// ## Examples
/// ```
/// # #[cfg(all(feature = "trace", feature = "grpc-tonic"))]
/// # {
/// use crate::opentelemetry_otlp::{WithExportConfig, WithTonicConfig};
/// let exporter_builder = opentelemetry_otlp::SpanExporter::builder()
///     .with_tonic()
///     .with_compression(opentelemetry_otlp::Compression::Gzip);
/// # }
/// ```
pub trait WithTonicConfig {
    /// Set the TLS settings for the collector endpoint.
    #[cfg(feature = "tls")]
    fn with_tls_config(self, tls_config: ClientTlsConfig) -> Self;

    /// Set custom metadata entries to send to the collector.
    fn with_metadata(self, metadata: MetadataMap) -> Self;

    /// Set the compression algorithm to use when communicating with the collector.
    fn with_compression(self, compression: Compression) -> Self;

    /// Use `channel` as tonic's transport channel.
    /// this will override tls config and should only be used
    /// when working with non-HTTP transports.
    ///
    /// Users MUST make sure the [`ExportConfig::timeout`] is
    /// the same as the channel's timeout.
    fn with_channel(self, channel: tonic::transport::Channel) -> Self;

    /// Use a custom `interceptor` to modify each outbound request.
    /// this can be used to modify the grpc metadata, for example
    /// to inject auth tokens.
    fn with_interceptor<I>(self, interceptor: I) -> Self
    where
        I: tonic::service::Interceptor + Clone + Send + Sync + 'static;
}

impl<B: HasTonicConfig> WithTonicConfig for B {
    #[cfg(feature = "tls")]
    fn with_tls_config(mut self, tls_config: ClientTlsConfig) -> Self {
        self.tonic_config().tls_config = Some(tls_config);
        self
    }

    /// Set custom metadata entries to send to the collector.
    fn with_metadata(mut self, metadata: MetadataMap) -> Self {
        // extending metadata maps is harder than just casting back/forth
        let mut existing_headers = self
            .tonic_config()
            .metadata
            .clone()
            .unwrap_or_default()
            .into_headers();
        existing_headers.extend(metadata.into_headers());

        self.tonic_config().metadata = Some(MetadataMap::from_headers(existing_headers));
        self
    }

    fn with_compression(mut self, compression: Compression) -> Self {
        self.tonic_config().compression = Some(compression);
        self
    }

    fn with_channel(mut self, channel: tonic::transport::Channel) -> Self {
        self.tonic_config().channel = Some(channel);
        self
    }

    fn with_interceptor<I>(mut self, interceptor: I) -> Self
    where
        I: tonic::service::Interceptor + Clone + Send + Sync + 'static,
    {
        self.tonic_config().interceptor = Some(BoxInterceptor(Box::new(interceptor)));
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::exporter::tests::run_env_test;
    use crate::exporter::tonic::WithTonicConfig;
    #[cfg(feature = "grpc-tonic")]
    use crate::exporter::Compression;
    use crate::{TonicExporterBuilder, WithExportConfig, OTEL_EXPORTER_OTLP_TRACES_ENDPOINT};
    use crate::{OTEL_EXPORTER_OTLP_HEADERS, OTEL_EXPORTER_OTLP_TRACES_HEADERS};
    use http::{HeaderMap, HeaderName, HeaderValue};
    use tonic::metadata::{MetadataMap, MetadataValue};

    #[test]
    fn test_with_metadata() {
        // metadata should merge with the current one with priority instead of just replacing it
        let mut metadata = MetadataMap::new();
        metadata.insert("foo", "bar".parse().unwrap());
        let builder = TonicExporterBuilder::default().with_metadata(metadata);
        let result = builder.tonic_config.metadata.unwrap();
        let foo = result
            .get("foo")
            .expect("there to always be an entry for foo");
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
    fn test_with_gzip_compression() {
        // metadata should merge with the current one with priority instead of just replacing it
        let mut metadata = MetadataMap::new();
        metadata.insert("foo", "bar".parse().unwrap());
        let builder = TonicExporterBuilder::default().with_compression(Compression::Gzip);
        assert_eq!(builder.tonic_config.compression.unwrap(), Compression::Gzip);
    }

    #[test]
    #[cfg(feature = "zstd-tonic")]
    fn test_with_zstd_compression() {
        let builder = TonicExporterBuilder::default().with_compression(Compression::Zstd);
        assert_eq!(builder.tonic_config.compression.unwrap(), Compression::Zstd);
    }

    #[test]
    fn test_convert_compression() {
        #[cfg(feature = "gzip-tonic")]
        assert!(tonic::codec::CompressionEncoding::try_from(Compression::Gzip).is_ok());
        #[cfg(not(feature = "gzip-tonic"))]
        assert!(tonic::codec::CompressionEncoding::try_from(Compression::Gzip).is_err());
        #[cfg(feature = "zstd-tonic")]
        assert!(tonic::codec::CompressionEncoding::try_from(Compression::Zstd).is_ok());
        #[cfg(not(feature = "zstd-tonic"))]
        assert!(tonic::codec::CompressionEncoding::try_from(Compression::Zstd).is_err());
    }

    #[test]
    fn test_parse_headers_from_env() {
        run_env_test(
            vec![
                (OTEL_EXPORTER_OTLP_TRACES_HEADERS, "k1=v1,k2=v2"),
                (OTEL_EXPORTER_OTLP_HEADERS, "k3=v3"),
            ],
            || {
                assert_eq!(
                    super::parse_headers_from_env(OTEL_EXPORTER_OTLP_TRACES_HEADERS).0,
                    HeaderMap::from_iter([
                        (
                            HeaderName::from_static("k1"),
                            HeaderValue::from_static("v1")
                        ),
                        (
                            HeaderName::from_static("k2"),
                            HeaderValue::from_static("v2")
                        ),
                    ])
                );

                assert_eq!(
                    super::parse_headers_from_env("EMPTY_ENV").0,
                    HeaderMap::from_iter([(
                        HeaderName::from_static("k3"),
                        HeaderValue::from_static("v3")
                    )])
                );
            },
        )
    }

    #[test]
    fn test_merge_metadata_with_headers_from_env() {
        run_env_test(
            vec![(OTEL_EXPORTER_OTLP_TRACES_HEADERS, "k1=v1,k2=v2")],
            || {
                let headers_from_env =
                    super::parse_headers_from_env(OTEL_EXPORTER_OTLP_TRACES_HEADERS);

                let mut metadata = MetadataMap::new();
                metadata.insert("foo", "bar".parse().unwrap());
                metadata.insert("k1", "v0".parse().unwrap());

                let result =
                    super::merge_metadata_with_headers_from_env(metadata, headers_from_env.0);

                assert_eq!(
                    result.get("foo").unwrap(),
                    MetadataValue::from_static("bar")
                );
                assert_eq!(result.get("k1").unwrap(), MetadataValue::from_static("v1"));
                assert_eq!(result.get("k2").unwrap(), MetadataValue::from_static("v2"));
            },
        );
    }

    #[test]
    fn test_tonic_exporter_endpoint() {
        // default endpoint for grpc should not add signal path.
        run_env_test(vec![], || {
            let exporter = TonicExporterBuilder::default();

            let url = TonicExporterBuilder::resolve_endpoint(
                OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
                exporter.exporter_config.endpoint,
            );

            assert_eq!(url, "http://localhost:4317");
        });

        // if builder endpoint is set, it should not use default.
        run_env_test(vec![], || {
            let exporter = TonicExporterBuilder::default().with_endpoint("http://localhost:1234");

            let url = TonicExporterBuilder::resolve_endpoint(
                OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
                exporter.exporter_config.endpoint,
            );

            assert_eq!(url, "http://localhost:1234");
        });
    }
}
