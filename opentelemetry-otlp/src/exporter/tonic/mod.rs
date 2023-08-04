use std::env;
use std::fmt::{Debug, Formatter};
use std::time::Duration;

use tonic::codec::CompressionEncoding;
use tonic::metadata::{KeyAndValueRef, MetadataMap};
use tonic::service::Interceptor;
use tonic::transport::Channel;
#[cfg(feature = "tls")]
use tonic::transport::ClientTlsConfig;

use super::default_headers;
use crate::exporter::Compression;
use crate::{
    ExportConfig, OTEL_EXPORTER_OTLP_COMPRESSION, OTEL_EXPORTER_OTLP_ENDPOINT,
    OTEL_EXPORTER_OTLP_TIMEOUT,
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
        I: tonic::service::Interceptor + Clone + Send + Sync + 'static,
    {
        self.interceptor = Some(BoxInterceptor(Box::new(interceptor)));
        self
    }

    fn build_channel(
        &mut self,
        signal_endpoint_var: &str,
        signal_endpoint_path: &str,
        signal_timeout_var: &str,
        signal_compression_var: &str,
    ) -> Result<(Channel, BoxInterceptor, Option<CompressionEncoding>), crate::Error> {
        let config = &mut self.exporter_config;
        let tonic_config = &mut self.tonic_config;

        let endpoint = match env::var(signal_endpoint_var)
            .ok()
            .or(env::var(OTEL_EXPORTER_OTLP_ENDPOINT).ok())
        {
            Some(val) => val,
            None => format!("{}{signal_endpoint_path}", config.endpoint),
        };

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
        let compression = resolve_compression(tonic_config, signal_compression_var)?;

        let endpoint = Channel::from_shared(endpoint).map_err(crate::Error::from)?;

        #[cfg(feature = "tls")]
        let channel = match tonic_config.tls_config.take() {
            Some(tls_config) => endpoint
                .tls_config(tls_config)
                .map_err(crate::Error::from)?,
            None => endpoint,
        }
        .timeout(timeout)
        .connect_lazy();

        #[cfg(not(feature = "tls"))]
        let channel = endpoint.timeout(timeout).connect_lazy();

        let metadata = tonic_config.metadata.take().unwrap_or_default();
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

        let interceptor = match self.interceptor.take() {
            Some(mut interceptor) => {
                BoxInterceptor(Box::new(move |req| interceptor.call(add_metadata(req)?)))
            }
            None => BoxInterceptor(Box::new(add_metadata)),
        };

        Ok((channel, interceptor, compression))
    }

    /// Build a new tonic log exporter
    #[cfg(feature = "logs")]
    pub fn build_log_exporter(
        mut self,
    ) -> Result<crate::logs::LogExporter, opentelemetry_api::logs::LogError> {
        use crate::exporter::tonic::logs::TonicLogsClient;

        let (channel, interceptor, compression) = self.build_channel(
            crate::logs::OTEL_EXPORTER_OTLP_LOGS_ENDPOINT,
            "/v1/logs",
            crate::logs::OTEL_EXPORTER_OTLP_LOGS_TIMEOUT,
            crate::logs::OTEL_EXPORTER_OTLP_LOGS_COMPRESSION,
        )?;

        let client = TonicLogsClient::new(channel, interceptor, compression);

        Ok(crate::logs::LogExporter::new(client))
    }

    /// Build a new tonic metrics exporter
    #[cfg(feature = "metrics")]
    pub fn build_metrics_exporter(
        mut self,
        aggregation_selector: Box<dyn opentelemetry_sdk::metrics::reader::AggregationSelector>,
        temporality_selector: Box<dyn opentelemetry_sdk::metrics::reader::TemporalitySelector>,
    ) -> opentelemetry_api::metrics::Result<crate::MetricsExporter> {
        use crate::MetricsExporter;
        use metrics::TonicMetricsClient;

        let (channel, interceptor, compression) = self.build_channel(
            crate::metric::OTEL_EXPORTER_OTLP_METRICS_ENDPOINT,
            "/v1/metrics",
            crate::metric::OTEL_EXPORTER_OTLP_METRICS_TIMEOUT,
            crate::metric::OTEL_EXPORTER_OTLP_METRICS_COMPRESSION,
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
        mut self,
    ) -> Result<crate::SpanExporter, opentelemetry_api::trace::TraceError> {
        use crate::exporter::tonic::trace::TonicTracesClient;

        let (channel, interceptor, compression) = self.build_channel(
            crate::span::OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
            "/v1/traces",
            crate::span::OTEL_EXPORTER_OTLP_TRACES_TIMEOUT,
            crate::span::OTEL_EXPORTER_OTLP_TRACES_COMPRESSION,
        )?;

        let client = TonicTracesClient::new(channel, interceptor, compression);

        Ok(crate::SpanExporter::new(client))
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
