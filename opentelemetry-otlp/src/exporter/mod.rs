//! OTLP exporter builder and configurations.
//!
//! OTLP supports sending data via different protocols and formats.

#[cfg(feature = "grpc-sys")]
use crate::exporter::grpcio::GrpcioExporterBuilder;
#[cfg(feature = "http-proto")]
use crate::exporter::http::HttpExporterBuilder;
#[cfg(feature = "grpc-tonic")]
use crate::exporter::tonic::TonicExporterBuilder;
use crate::{Error, Protocol};
#[cfg(any(feature = "grpc-tonic", feature = "http-proto"))]
use ::http::header::{HeaderName, HeaderValue};
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::time::Duration;

/// Target to which the exporter is going to send signals, defaults to https://localhost:4317.
/// Learn about the relationship between this constant and metrics/spans/logs at
/// <https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/protocol/exporter.md#endpoint-urls-for-otlphttp>
pub const OTEL_EXPORTER_OTLP_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_ENDPOINT";
/// Default target to which the exporter is going to send signals.
pub const OTEL_EXPORTER_OTLP_ENDPOINT_DEFAULT: &str = OTEL_EXPORTER_OTLP_HTTP_ENDPOINT_DEFAULT;
/// Key-value pairs to be used as headers associated with gRPC or HTTP requests
/// Example: `k1=v1,k2=v2`
/// Note: as of now, this is only supported for HTTP requests.
pub const OTEL_EXPORTER_OTLP_HEADERS: &str = "OTEL_EXPORTER_OTLP_HEADERS";
/// Protocol the exporter will use. Either `http/protobuf` or `grpc`.
pub const OTEL_EXPORTER_OTLP_PROTOCOL: &str = "OTEL_EXPORTER_OTLP_PROTOCOL";
/// Compression algorithm to use, defaults to none.
pub const OTEL_EXPORTER_OTLP_COMPRESSION: &str = "OTEL_EXPORTER_OTLP_COMPRESSION";

#[cfg(feature = "http-proto")]
/// Default protocol, using http-proto.
pub const OTEL_EXPORTER_OTLP_PROTOCOL_DEFAULT: &str = OTEL_EXPORTER_OTLP_PROTOCOL_HTTP_PROTOBUF;
#[cfg(all(
    any(feature = "grpc-tonic", feature = "grpcio"),
    not(feature = "http-proto")
))]
/// Default protocol, using grpc as http-proto feature is not enabled.
pub const OTEL_EXPORTER_OTLP_PROTOCOL_DEFAULT: &str = OTEL_EXPORTER_OTLP_PROTOCOL_GRPC;
#[cfg(not(any(any(feature = "grpc-tonic", feature = "grpcio", feature = "http-proto"))))]
/// Default protocol if no features are enabled.
pub const OTEL_EXPORTER_OTLP_PROTOCOL_DEFAULT: &str = "";

const OTEL_EXPORTER_OTLP_PROTOCOL_HTTP_PROTOBUF: &str = "http/protobuf";
const OTEL_EXPORTER_OTLP_PROTOCOL_GRPC: &str = "grpc";

/// Max waiting time for the backend to process each signal batch, defaults to 10 seconds.
pub const OTEL_EXPORTER_OTLP_TIMEOUT: &str = "OTEL_EXPORTER_OTLP_TIMEOUT";
/// Default max waiting time for the backend to process each signal batch.
pub const OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT: u64 = 10;

// Endpoints per protocol https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/protocol/exporter.md
const OTEL_EXPORTER_OTLP_GRPC_ENDPOINT_DEFAULT: &str = "http://localhost:4317";
const OTEL_EXPORTER_OTLP_HTTP_ENDPOINT_DEFAULT: &str = "http://localhost:4318";

#[cfg(feature = "grpc-sys")]
pub(crate) mod grpcio;
#[cfg(feature = "http-proto")]
pub(crate) mod http;
#[cfg(feature = "grpc-tonic")]
pub(crate) mod tonic;

/// Configuration for the OTLP exporter.
#[derive(Debug)]
pub struct ExportConfig {
    /// The base address of the OTLP collector. If not set, the default address is used.
    pub endpoint: String,

    /// The protocol to use when communicating with the collector.
    pub protocol: Protocol,

    /// The timeout to the collector.
    pub timeout: Duration,
}

impl Default for ExportConfig {
    fn default() -> Self {
        let protocol = default_protocol();

        ExportConfig {
            endpoint: default_endpoint(protocol),
            protocol,
            timeout: Duration::from_secs(OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT),
        }
    }
}

/// The compression algorithm to use when sending data.
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Compression {
    /// Compresses data using gzip.
    Gzip,
}

impl Display for Compression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Compression::Gzip => write!(f, "gzip"),
        }
    }
}

impl FromStr for Compression {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "gzip" => Ok(Compression::Gzip),
            _ => Err(Error::UnsupportedCompressionAlgorithm(s.to_string())),
        }
    }
}

/// default protocol based on enabled features
fn default_protocol() -> Protocol {
    match OTEL_EXPORTER_OTLP_PROTOCOL_DEFAULT {
        OTEL_EXPORTER_OTLP_PROTOCOL_HTTP_PROTOBUF => Protocol::HttpBinary,
        OTEL_EXPORTER_OTLP_PROTOCOL_GRPC => Protocol::Grpc,
        _ => Protocol::HttpBinary,
    }
}

/// default endpoint for protocol
fn default_endpoint(protocol: Protocol) -> String {
    match protocol {
        Protocol::Grpc => OTEL_EXPORTER_OTLP_GRPC_ENDPOINT_DEFAULT.to_string(),
        Protocol::HttpBinary => OTEL_EXPORTER_OTLP_HTTP_ENDPOINT_DEFAULT.to_string(),
    }
}

/// default user-agent headers
#[cfg(any(feature = "grpc-tonic", feature = "grpc-sys", feature = "http-proto"))]
fn default_headers() -> std::collections::HashMap<String, String> {
    let mut headers = std::collections::HashMap::new();
    headers.insert(
        "User-Agent".to_string(),
        format!("OTel OTLP Exporter Rust/{}", env!("CARGO_PKG_VERSION")),
    );
    headers
}

/// Provide access to the export config field within the exporter builders.
pub trait HasExportConfig {
    /// Return a mutable reference to the export config within the exporter builders.
    fn export_config(&mut self) -> &mut ExportConfig;
}

#[cfg(feature = "grpc-tonic")]
impl HasExportConfig for TonicExporterBuilder {
    fn export_config(&mut self) -> &mut ExportConfig {
        &mut self.exporter_config
    }
}

#[cfg(feature = "grpc-sys")]
impl HasExportConfig for GrpcioExporterBuilder {
    fn export_config(&mut self) -> &mut ExportConfig {
        &mut self.exporter_config
    }
}

#[cfg(feature = "http-proto")]
impl HasExportConfig for HttpExporterBuilder {
    fn export_config(&mut self) -> &mut ExportConfig {
        &mut self.exporter_config
    }
}

/// Expose methods to override export configuration.
///
/// This trait will be implemented for every struct that implemented [`HasExportConfig`] trait.
///
/// ## Examples
/// ```
/// # #[cfg(all(feature = "trace", feature = "grpc-tonic"))]
/// # {
/// use crate::opentelemetry_otlp::WithExportConfig;
/// let exporter_builder = opentelemetry_otlp::new_exporter()
///     .tonic()
///     .with_endpoint("http://localhost:7201");
/// # }
/// ```
pub trait WithExportConfig {
    /// Set the address of the OTLP collector. If not set, the default address is used.
    fn with_endpoint<T: Into<String>>(self, endpoint: T) -> Self;
    /// Set the protocol to use when communicating with the collector.
    ///
    /// Note that protocols that are not supported by exporters will be ignore. The exporter
    /// will use default protocol in this case.
    ///
    /// ## Note
    /// All exporters in this crate are only support one protocol thus choosing the protocol is an no-op at the moment
    fn with_protocol(self, protocol: Protocol) -> Self;
    /// Set the timeout to the collector.
    fn with_timeout(self, timeout: Duration) -> Self;
    /// Set export config. This will override all previous configuration.
    fn with_export_config(self, export_config: ExportConfig) -> Self;
}

impl<B: HasExportConfig> WithExportConfig for B {
    fn with_endpoint<T: Into<String>>(mut self, endpoint: T) -> Self {
        self.export_config().endpoint = endpoint.into();
        self
    }

    fn with_protocol(mut self, protocol: Protocol) -> Self {
        self.export_config().protocol = protocol;
        self
    }

    fn with_timeout(mut self, timeout: Duration) -> Self {
        self.export_config().timeout = timeout;
        self
    }

    fn with_export_config(mut self, exporter_config: ExportConfig) -> Self {
        self.export_config().endpoint = exporter_config.endpoint;
        self.export_config().protocol = exporter_config.protocol;
        self.export_config().timeout = exporter_config.timeout;
        self
    }
}

#[cfg(any(feature = "grpc-tonic", feature = "http-proto"))]
fn parse_header_string(value: &str) -> impl Iterator<Item = (HeaderName, HeaderValue)> + '_ {
    value
        .split_terminator(',')
        .map(str::trim)
        .filter_map(|key_value_string| parse_header_key_value_string(key_value_string).ok()?)
}

#[cfg(any(feature = "grpc-tonic", feature = "http-proto"))]
fn parse_header_key_value_string(
    key_value_string: &str,
) -> Result<Option<(HeaderName, HeaderValue)>, Error> {
    if let Some((key, value)) = key_value_string
        .split_once('=')
        .map(|(key, value)| (key.trim(), value.trim()))
        .filter(|(key, value)| !key.is_empty() && !value.is_empty())
    {
        Ok(Some((
            HeaderName::from_str(key)?,
            HeaderValue::from_str(value)?,
        )))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
#[cfg(any(feature = "grpc-tonic", feature = "http-proto"))]
mod tests {
    use ::http::{HeaderName, HeaderValue};
    // Make sure env tests are not running concurrently
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    pub(crate) fn run_env_test<T, F>(env_vars: T, f: F)
    where
        F: FnOnce(),
        T: Into<Vec<(&'static str, &'static str)>>,
    {
        let _env_lock = ENV_LOCK.lock().expect("env test lock poisoned");
        let env_vars = env_vars.into();
        for (k, v) in env_vars.iter() {
            std::env::set_var(k, v);
        }
        f();
        for (k, _) in env_vars {
            std::env::remove_var(k);
        }
    }

    #[test]
    fn test_parse_header_string() {
        let test_cases = vec![
            // Format: (input_str, expected_headers)
            (
                "k1=v1",
                vec![(
                    HeaderName::from_static("k1"),
                    HeaderValue::from_static("v1"),
                )],
            ),
            (
                "k1=v1,k2=v2",
                vec![
                    (
                        HeaderName::from_static("k1"),
                        HeaderValue::from_static("v1"),
                    ),
                    (
                        HeaderName::from_static("k2"),
                        HeaderValue::from_static("v2"),
                    ),
                ],
            ),
            (
                "k1=v1=10,k2,k3",
                vec![(
                    HeaderName::from_static("k1"),
                    HeaderValue::from_static("v1=10"),
                )],
            ),
            (
                "k1=v1,,,k2,k3=10,k4=\x7F",
                vec![
                    (
                        HeaderName::from_static("k1"),
                        HeaderValue::from_static("v1"),
                    ),
                    (
                        HeaderName::from_static("k3"),
                        HeaderValue::from_static("10"),
                    ),
                ],
            ),
        ];

        for (input_str, expected_headers) in test_cases {
            assert_eq!(
                super::parse_header_string(input_str).collect::<Vec<_>>(),
                expected_headers,
            )
        }
    }

    #[test]
    fn test_parse_header_key_value_string() {
        assert_eq!(
            super::parse_header_key_value_string("k1=v1").unwrap(),
            Some((
                HeaderName::from_static("k1"),
                HeaderValue::from_static("v1")
            ))
        );
        assert_eq!(super::parse_header_key_value_string("").unwrap(), None);
        assert_eq!(super::parse_header_key_value_string("=v1").unwrap(), None);
        assert_eq!(super::parse_header_key_value_string("k1=").unwrap(), None);
        assert!(super::parse_header_key_value_string("k1=\x7F").is_err(),);
        assert!(super::parse_header_key_value_string("\x7F=1").is_err(),);
    }
}
