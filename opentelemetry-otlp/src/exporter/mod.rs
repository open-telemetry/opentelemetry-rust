//! OTLP exporter builder and configurations.
//!
//! OTLP supports sending data via different protocols and formats.

#[cfg(any(feature = "http-proto", feature = "http-json"))]
use crate::exporter::http::HttpExporterBuilder;
#[cfg(feature = "grpc-tonic")]
use crate::exporter::tonic::TonicExporterBuilder;
use crate::{Error, Protocol};
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

#[cfg(all(
    feature = "trace",
    not(feature = "http-proto"),
    not(feature = "grpc-tonic")
))]
/// Default protocol, using http-json.
pub const OTEL_EXPORTER_OTLP_PROTOCOL_DEFAULT: &str = OTEL_EXPORTER_OTLP_PROTOCOL_HTTP_JSON;
#[cfg(feature = "http-proto")]
/// Default protocol, using http-proto.
pub const OTEL_EXPORTER_OTLP_PROTOCOL_DEFAULT: &str = OTEL_EXPORTER_OTLP_PROTOCOL_HTTP_PROTOBUF;
#[cfg(all(
    feature = "grpc-tonic",
    not(all(feature = "http-proto", feature = "http-json"))
))]
/// Default protocol, using grpc as http-proto or http-json feature is not enabled.
pub const OTEL_EXPORTER_OTLP_PROTOCOL_DEFAULT: &str = OTEL_EXPORTER_OTLP_PROTOCOL_GRPC;
#[cfg(not(any(any(feature = "grpc-tonic", feature = "http-proto", feature = "http-json"))))]
/// Default protocol if no features are enabled.
pub const OTEL_EXPORTER_OTLP_PROTOCOL_DEFAULT: &str = "";

const OTEL_EXPORTER_OTLP_PROTOCOL_HTTP_PROTOBUF: &str = "http/protobuf";
const OTEL_EXPORTER_OTLP_PROTOCOL_GRPC: &str = "grpc";
const OTEL_EXPORTER_OTLP_PROTOCOL_HTTP_JSON: &str = "http/json";

/// Max waiting time for the backend to process each signal batch, defaults to 10 seconds.
pub const OTEL_EXPORTER_OTLP_TIMEOUT: &str = "OTEL_EXPORTER_OTLP_TIMEOUT";
/// Default max waiting time for the backend to process each signal batch.
pub const OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT: u64 = 10;

// Endpoints per protocol https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/protocol/exporter.md
const OTEL_EXPORTER_OTLP_GRPC_ENDPOINT_DEFAULT: &str = "http://localhost:4317";
const OTEL_EXPORTER_OTLP_HTTP_ENDPOINT_DEFAULT: &str = "http://localhost:4318";

#[cfg(any(feature = "http-proto", feature = "http-json"))]
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
        OTEL_EXPORTER_OTLP_PROTOCOL_HTTP_JSON => Protocol::HttpJson,
        _ => Protocol::HttpBinary,
    }
}

/// default endpoint for protocol
fn default_endpoint(protocol: Protocol) -> String {
    match protocol {
        Protocol::Grpc => OTEL_EXPORTER_OTLP_GRPC_ENDPOINT_DEFAULT.to_string(),
        Protocol::HttpBinary => OTEL_EXPORTER_OTLP_HTTP_ENDPOINT_DEFAULT.to_string(),
        Protocol::HttpJson => OTEL_EXPORTER_OTLP_HTTP_ENDPOINT_DEFAULT.to_string(),
    }
}

/// default user-agent headers
#[cfg(any(feature = "grpc-tonic", feature = "http-proto", feature = "http-json"))]
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

#[cfg(any(feature = "http-proto", feature = "http-json"))]
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

#[cfg(any(feature = "grpc-tonic", feature = "http-proto", feature = "http-json"))]
fn parse_header_string(value: &str) -> impl Iterator<Item = (&str, String)> {
    value
        .split_terminator(',')
        .map(str::trim)
        .filter_map(parse_header_key_value_string)
}

#[cfg(any(feature = "grpc-tonic", feature = "http-proto", feature = "http-json"))]
fn url_decode(value: &str) -> Option<String> {
    let mut result = String::with_capacity(value.len());
    let mut chars_to_decode = Vec::<u8>::new();
    let mut all_chars = value.chars();

    loop {
        let ch = all_chars.next();

        if ch.is_some() && ch.unwrap() == '%' {
            chars_to_decode.push(
                u8::from_str_radix(&format!("{}{}", all_chars.next()?, all_chars.next()?), 16)
                    .ok()?,
            );
            continue;
        }

        if !chars_to_decode.is_empty() {
            result.push_str(std::str::from_utf8(&chars_to_decode).ok()?);
            chars_to_decode.clear();
        }

        if let Some(c) = ch {
            result.push(c);
        } else {
            return Some(result);
        }
    }
}

#[cfg(any(feature = "grpc-tonic", feature = "http-proto", feature = "http-json"))]
fn parse_header_key_value_string(key_value_string: &str) -> Option<(&str, String)> {
    key_value_string
        .split_once('=')
        .map(|(key, value)| {
            (
                key.trim(),
                url_decode(value.trim()).unwrap_or(value.to_string()),
            )
        })
        .filter(|(key, value)| !key.is_empty() && !value.is_empty())
}

#[cfg(test)]
#[cfg(any(feature = "grpc-tonic", feature = "http-proto", feature = "http-json"))]
mod tests {

    pub(crate) fn run_env_test<T, F>(env_vars: T, f: F)
    where
        F: FnOnce(),
        T: Into<Vec<(&'static str, &'static str)>>,
    {
        temp_env::with_vars(
            env_vars
                .into()
                .iter()
                .map(|&(k, v)| (k, Some(v)))
                .collect::<Vec<(&'static str, Option<&'static str>)>>(),
            f,
        )
    }

    #[cfg(any(feature = "http-proto", feature = "http-json"))]
    #[test]
    fn test_default_http_endpoint() {
        let exporter_builder = crate::new_exporter().http();

        assert_eq!(
            exporter_builder.exporter_config.endpoint,
            "http://localhost:4318"
        );
    }

    #[cfg(feature = "grpc-tonic")]
    #[test]
    fn test_default_tonic_endpoint() {
        let exporter_builder = crate::new_exporter().tonic();

        assert_eq!(
            exporter_builder.exporter_config.endpoint,
            "http://localhost:4317"
        );
    }

    #[test]
    fn test_url_decode() {
        let test_cases = vec![
            // Format: (encoded, expected_decoded)
            ("v%201", Some("v 1")),
            ("v 1", Some("v 1")),
            ("%C3%B6%C3%A0%C2%A7%C3%96abcd%C3%84", Some("öà§ÖabcdÄ")),
            ("v%XX1", None),
        ];

        for (encoded, expected_decoded) in test_cases {
            assert_eq!(
                super::url_decode(encoded),
                expected_decoded.map(|v| v.to_string()),
            )
        }
    }

    #[test]
    fn test_parse_header_string() {
        let test_cases = vec![
            // Format: (input_str, expected_headers)
            ("k1=v1", vec![("k1", "v1")]),
            ("k1=v1,k2=v2", vec![("k1", "v1"), ("k2", "v2")]),
            ("k1=v1=10,k2,k3", vec![("k1", "v1=10")]),
            ("k1=v1,,,k2,k3=10", vec![("k1", "v1"), ("k3", "10")]),
        ];

        for (input_str, expected_headers) in test_cases {
            assert_eq!(
                super::parse_header_string(input_str).collect::<Vec<_>>(),
                expected_headers
                    .into_iter()
                    .map(|(k, v)| (k, v.to_string()))
                    .collect::<Vec<_>>(),
            )
        }
    }

    #[test]
    fn test_parse_header_key_value_string() {
        let test_cases = vec![
            // Format: (input_str, expected_header)
            ("k1=v1", Some(("k1", "v1"))),
            (
                "Authentication=Basic AAA",
                Some(("Authentication", "Basic AAA")),
            ),
            (
                "Authentication=Basic%20AAA",
                Some(("Authentication", "Basic AAA")),
            ),
            ("k1=%XX", Some(("k1", "%XX"))),
            ("", None),
            ("=v1", None),
            ("k1=", None),
        ];

        for (input_str, expected_headers) in test_cases {
            assert_eq!(
                super::parse_header_key_value_string(input_str),
                expected_headers.map(|(k, v)| (k, v.to_string())),
            )
        }
    }
}
