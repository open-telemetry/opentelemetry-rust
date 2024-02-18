use std::env;
use std::fmt::{Debug, Formatter};
use std::str::FromStr;
use std::time::Duration;

use http::{HeaderMap, HeaderName, HeaderValue};
use tonic::codec::CompressionEncoding;
use tonic::metadata::{KeyAndValueRef, MetadataMap};
use tonic::service::Interceptor;
use tonic::transport::Channel;
#[cfg(feature = "tls")]
use tonic::transport::ClientTlsConfig;

use super::{default_headers, parse_header_string};
use crate::exporter::Compression;
use crate::{
    ExportConfig, OTEL_EXPORTER_OTLP_COMPRESSION, OTEL_EXPORTER_OTLP_ENDPOINT,
    OTEL_EXPORTER_OTLP_HEADERS, OTEL_EXPORTER_OTLP_TIMEOUT,
};

#[cfg(feature = "logs")]
mod logs;

#[cfg(feature = "metrics")]
mod metrics;

#[cfg(feature = "trace")]
mod trace;

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

fn resolve_compression(
    tonic_config: &TonicConfig,
    env_override: &str,
) -> Result<Option<CompressionEncoding>, crate::Error> {
    if let Some(compression) = tonic_config.compression {
        Ok(Some(compression.try_into()?))
    } else if let Ok(compression) = env::var(env_override) {
        Ok(Some(compression.parse::<Compression>()?.try_into()?))
    } else if let Ok(compression) = env::var(OTEL_EXPORTER_OTLP_COMPRESSION) {
        Ok(Some(compression.parse::<Compression>()?.try_into()?))
    } else {
        Ok(None)
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
/// use opentelemetry_sdk::metrics::reader::{
///     DefaultAggregationSelector, DefaultTemporalitySelector,
/// };
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a span exporter you can use to when configuring tracer providers
/// # #[cfg(feature="trace")]
/// let span_exporter = opentelemetry_otlp::new_exporter().tonic().build_span_exporter()?;
///
/// // Create a metrics exporter you can use when configuring meter providers
/// # #[cfg(feature="metrics")]
/// let metrics_exporter = opentelemetry_otlp::new_exporter()
///     .tonic()
///     .build_metrics_exporter(
///         Box::new(DefaultAggregationSelector::new()),
///         Box::new(DefaultTemporalitySelector::new()),
///     )?;
///
/// // Create a log exporter you can use when configuring logger providers
/// # #[cfg(feature="logs")]
/// let log_exporter = opentelemetry_otlp::new_exporter().tonic().build_log_exporter()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct TonicExporterBuilder {
    pub(crate) exporter_config: ExportConfig,
    pub(crate) tonic_config: TonicConfig,
    pub(crate) channel: Option<tonic::transport::Channel>,
    pub(crate) interceptor: Option<BoxInterceptor>,
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
            exporter_config: ExportConfig {
                protocol: crate::Protocol::Grpc,
                endpoint: crate::exporter::default_endpoint(crate::Protocol::Grpc),
                ..Default::default()
            },
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
        existing_headers.extend(incoming_headers);

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
        I: tonic::service::Interceptor + Clone + Send + Sync + 'static,
    {
        self.interceptor = Some(BoxInterceptor(Box::new(interceptor)));
        self
    }

    fn build_channel(
        self,
        signal_endpoint_var: &str,
        signal_endpoint_path: &str,
        signal_timeout_var: &str,
        signal_compression_var: &str,
        signal_headers_var: &str,
    ) -> Result<(Channel, BoxInterceptor, Option<CompressionEncoding>), crate::Error> {
        let tonic_config = self.tonic_config;
        let compression = resolve_compression(&tonic_config, signal_compression_var)?;

        let headers_from_env = parse_headers_from_env(signal_headers_var);
        let metadata = merge_metadata_with_headers_from_env(
            tonic_config.metadata.unwrap_or_default(),
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

        let interceptor = match self.interceptor {
            Some(mut interceptor) => {
                BoxInterceptor(Box::new(move |req| interceptor.call(add_metadata(req)?)))
            }
            None => BoxInterceptor(Box::new(add_metadata)),
        };

        // If a custom channel was provided, use that channel instead of creating one
        if let Some(channel) = self.channel {
            return Ok((channel, interceptor, compression));
        }

        let config = self.exporter_config;
        let endpoint = match env::var(signal_endpoint_var)
            .ok()
            .or(env::var(OTEL_EXPORTER_OTLP_ENDPOINT).ok())
        {
            Some(val) => val,
            None => format!("{}{signal_endpoint_path}", config.endpoint),
        };

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
        let channel = match tonic_config.tls_config {
            Some(tls_config) => endpoint
                .tls_config(tls_config)
                .map_err(crate::Error::from)?,
            None => endpoint,
        }
        .timeout(timeout)
        .connect_lazy();

        #[cfg(not(feature = "tls"))]
        let channel = endpoint.timeout(timeout).connect_lazy();

        Ok((channel, interceptor, compression))
    }

    /// Build a new tonic log exporter
    #[cfg(feature = "logs")]
    pub fn build_log_exporter(
        self,
    ) -> Result<crate::logs::LogExporter, opentelemetry::logs::LogError> {
        use crate::exporter::tonic::logs::TonicLogsClient;

        let (channel, interceptor, compression) = self.build_channel(
            crate::logs::OTEL_EXPORTER_OTLP_LOGS_ENDPOINT,
            "/v1/logs",
            crate::logs::OTEL_EXPORTER_OTLP_LOGS_TIMEOUT,
            crate::logs::OTEL_EXPORTER_OTLP_LOGS_COMPRESSION,
            crate::logs::OTEL_EXPORTER_OTLP_LOGS_HEADERS,
        )?;

        let client = TonicLogsClient::new(channel, interceptor, compression);

        Ok(crate::logs::LogExporter::new(client))
    }

    /// Build a new tonic metrics exporter
    #[cfg(feature = "metrics")]
    pub fn build_metrics_exporter(
        self,
        aggregation_selector: Box<dyn opentelemetry_sdk::metrics::reader::AggregationSelector>,
        temporality_selector: Box<dyn opentelemetry_sdk::metrics::reader::TemporalitySelector>,
    ) -> opentelemetry::metrics::Result<crate::MetricsExporter> {
        use crate::MetricsExporter;
        use metrics::TonicMetricsClient;

        let (channel, interceptor, compression) = self.build_channel(
            crate::metric::OTEL_EXPORTER_OTLP_METRICS_ENDPOINT,
            "/v1/metrics",
            crate::metric::OTEL_EXPORTER_OTLP_METRICS_TIMEOUT,
            crate::metric::OTEL_EXPORTER_OTLP_METRICS_COMPRESSION,
            crate::metric::OTEL_EXPORTER_OTLP_METRICS_HEADERS,
        )?;

        let client = TonicMetricsClient::new(channel, interceptor, compression);

        Ok(MetricsExporter::new(
            client,
            temporality_selector,
            aggregation_selector,
        ))
    }

    /// Build a new tonic span exporter
    #[cfg(feature = "trace")]
    pub fn build_span_exporter(
        self,
    ) -> Result<crate::SpanExporter, opentelemetry::trace::TraceError> {
        use crate::exporter::tonic::trace::TonicTracesClient;

        let (channel, interceptor, compression) = self.build_channel(
            crate::span::OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
            "/v1/traces",
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

fn parse_headers_from_env(signal_headers_var: &str) -> HeaderMap {
    env::var(signal_headers_var)
        .or_else(|_| env::var(OTEL_EXPORTER_OTLP_HEADERS))
        .map(|input| {
            parse_header_string(&input)
                .filter_map(|(key, value)| {
                    Some((
                        HeaderName::from_str(key).ok()?,
                        HeaderValue::from_str(value).ok()?,
                    ))
                })
                .collect::<HeaderMap>()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use crate::exporter::tests::run_env_test;
    #[cfg(feature = "gzip-tonic")]
    use crate::exporter::Compression;
    use crate::TonicExporterBuilder;
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

    #[test]
    fn test_parse_headers_from_env() {
        run_env_test(
            vec![
                (OTEL_EXPORTER_OTLP_TRACES_HEADERS, "k1=v1,k2=v2"),
                (OTEL_EXPORTER_OTLP_HEADERS, "k3=v3"),
            ],
            || {
                assert_eq!(
                    super::parse_headers_from_env(OTEL_EXPORTER_OTLP_TRACES_HEADERS),
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
                    super::parse_headers_from_env("EMPTY_ENV"),
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
                    super::merge_metadata_with_headers_from_env(metadata, headers_from_env);

                assert_eq!(
                    result.get("foo").unwrap(),
                    MetadataValue::from_static("bar")
                );
                assert_eq!(result.get("k1").unwrap(), MetadataValue::from_static("v1"));
                assert_eq!(result.get("k2").unwrap(), MetadataValue::from_static("v2"));
            },
        );
    }
}
