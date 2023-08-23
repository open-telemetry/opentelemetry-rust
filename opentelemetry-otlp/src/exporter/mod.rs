//! OTLP exporter builder and configurations.
//!
//! OTLP supports sending data via different protocols and formats.
//!
//! Learn about the relationship between the OTEL_EXPORTER_OTLP_* environment variables
//! and metrics/spans/logs at
//! <https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/protocol/exporter.md#endpoint-urls-for-otlphttp>

#[cfg(feature = "grpc-sys")]
use crate::exporter::grpcio::GrpcioExporterBuilder;
#[cfg(feature = "http-proto")]
use crate::exporter::http::HttpExporterBuilder;
#[cfg(feature = "grpc-tonic")]
use crate::exporter::tonic::TonicExporterBuilder;
use crate::{Error, Protocol};
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::time::Duration;

// Environment variables
//
// These are used if no corresponding 'per-signal' environment variable is set.

/// Protocol the exporter will use. Either `http/protobuf` or `grpc`.
pub const OTEL_EXPORTER_OTLP_PROTOCOL: &str = "OTEL_EXPORTER_OTLP_PROTOCOL";
/// Target to which the exporter is going to send signals, defaults to https://localhost:4317.
pub const OTEL_EXPORTER_OTLP_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_ENDPOINT";
/// Max waiting time for the backend to process each signal batch, defaults to 10 seconds.
pub const OTEL_EXPORTER_OTLP_TIMEOUT: &str = "OTEL_EXPORTER_OTLP_TIMEOUT";
/// Compression algorithm to use, defaults to none.
pub const OTEL_EXPORTER_OTLP_COMPRESSION: &str = "OTEL_EXPORTER_OTLP_COMPRESSION";

// Defaults

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

/// Default target to which the exporter is going to send signals.
pub const OTEL_EXPORTER_OTLP_ENDPOINT_DEFAULT: &str = OTEL_EXPORTER_OTLP_HTTP_ENDPOINT_DEFAULT;

// Endpoints per protocol https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/protocol/exporter.md
const OTEL_EXPORTER_OTLP_GRPC_ENDPOINT_DEFAULT: &str = "http://localhost:4317";
const OTEL_EXPORTER_OTLP_HTTP_ENDPOINT_DEFAULT: &str = "http://localhost:4318";

/// Default max waiting time for the backend to process each signal batch.
pub const OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT: u64 = 10;

#[cfg(feature = "grpc-sys")]
pub(crate) mod grpcio;
#[cfg(feature = "http-proto")]
pub(crate) mod http;
#[cfg(feature = "grpc-tonic")]
pub(crate) mod tonic;

pub(crate) fn resolve_endpoint(
    config_value: &str,
    signal_endpoint_var: &str,
    signal_endpoint_path: &str,
) -> String {
    if let Ok(endpoint) = std::env::var(signal_endpoint_var) {
        endpoint
    } else if let Ok(endpoint) = std::env::var(OTEL_EXPORTER_OTLP_ENDPOINT) {
        with_path(&endpoint, signal_endpoint_path)
    } else {
        with_path(config_value, signal_endpoint_path)
    }
}

/// a helper function to add path to a base URL
fn with_path(base: &str, path: &str) -> String {
    let b = match base.strip_suffix('/') {
        Some(s) => s,
        None => base,
    };
    format!("{}{}", b, path)
}

pub(crate) fn resolve_timeout(config_value: Duration, signal_timeout_var: &str) -> Duration {
    if let Ok(val) = std::env::var(signal_timeout_var) {
        match val.parse() {
            Ok(seconds) => Duration::from_secs(seconds),
            Err(_) => config_value,
        }
    } else if let Ok(val) = std::env::var(OTEL_EXPORTER_OTLP_TIMEOUT) {
        match val.parse() {
            Ok(seconds) => Duration::from_secs(seconds),
            Err(_) => config_value,
        }
    } else {
        config_value
    }
}

pub(crate) fn resolve_compression(
    config_value: Option<Compression>,
    signal_compression_var: &str,
) -> Result<Option<Compression>, crate::Error> {
    if let Some(compression) = config_value {
        Ok(Some(compression))
    } else if let Ok(compression) = std::env::var(signal_compression_var) {
        Ok(Some(compression.parse::<Compression>()?))
    } else if let Ok(compression) = std::env::var(OTEL_EXPORTER_OTLP_COMPRESSION) {
        Ok(Some(compression.parse::<Compression>()?))
    } else {
        Ok(None)
    }
}

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
    fn with_protocol(self, protocol: Protocol) -> Self;
    /// Set the timeout to the collector.
    fn with_timeout(self, timeout: Duration) -> Self;
    /// Set the trace provider configuration from the given environment variables.
    ///
    /// If the value in environment variables is illegal, will fall back to use default value.
    fn with_env(self) -> Self;
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

    fn with_env(mut self) -> Self {
        let protocol = match std::env::var(OTEL_EXPORTER_OTLP_PROTOCOL)
            .unwrap_or_else(|_| OTEL_EXPORTER_OTLP_PROTOCOL_DEFAULT.to_string())
            .as_str()
        {
            OTEL_EXPORTER_OTLP_PROTOCOL_HTTP_PROTOBUF => Protocol::HttpBinary,
            OTEL_EXPORTER_OTLP_PROTOCOL_GRPC => Protocol::Grpc,
            _ => default_protocol(),
        };

        self.export_config().protocol = protocol;

        let endpoint = match std::env::var(OTEL_EXPORTER_OTLP_ENDPOINT) {
            Ok(val) => val,
            Err(_) => default_endpoint(protocol),
        };
        self.export_config().endpoint = endpoint;

        let timeout = match std::env::var(OTEL_EXPORTER_OTLP_TIMEOUT) {
            Ok(val) => u64::from_str(&val).unwrap_or(OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT),
            Err(_) => OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT,
        };
        self.export_config().timeout = Duration::from_secs(timeout);
        self
    }

    fn with_export_config(mut self, exporter_config: ExportConfig) -> Self {
        self.export_config().endpoint = exporter_config.endpoint;
        self.export_config().protocol = exporter_config.protocol;
        self.export_config().timeout = exporter_config.timeout;
        self
    }
}

#[cfg(test)]
#[cfg(feature = "grpc-tonic")]
mod tests {
    // If an env test fails then the mutex will be poisoned and the following error will be displayed.
    const LOCK_POISONED_MESSAGE: &str = "one of the other pipeline builder from env tests failed";

    use crate::exporter::{
        default_endpoint, default_protocol, resolve_endpoint, ExportConfig, HasExportConfig,
        WithExportConfig, OTEL_EXPORTER_OTLP_ENDPOINT, OTEL_EXPORTER_OTLP_GRPC_ENDPOINT_DEFAULT,
        OTEL_EXPORTER_OTLP_HTTP_ENDPOINT_DEFAULT, OTEL_EXPORTER_OTLP_PROTOCOL_GRPC,
        OTEL_EXPORTER_OTLP_PROTOCOL_HTTP_PROTOBUF, OTEL_EXPORTER_OTLP_TIMEOUT,
        OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT,
    };
    use crate::{new_exporter, Compression, Protocol, OTEL_EXPORTER_OTLP_PROTOCOL};
    use std::str::FromStr;
    use std::sync::Mutex;

    // Make sure env tests are not running concurrently
    static ENV_LOCK: Mutex<usize> = Mutex::new(0);

    #[test]
    fn test_pipeline_builder_from_env_default_vars() {
        let _env_lock = ENV_LOCK.lock().expect(LOCK_POISONED_MESSAGE);
        let exporter_builder = new_exporter().tonic().with_env();
        assert_eq!(
            exporter_builder.exporter_config.protocol,
            default_protocol()
        );
        assert_eq!(
            exporter_builder.exporter_config.endpoint,
            default_endpoint(default_protocol())
        );
        assert_eq!(
            exporter_builder.exporter_config.timeout,
            std::time::Duration::from_secs(OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT)
        );
    }

    #[test]
    fn test_pipeline_builder_from_env_endpoint() {
        let _env_lock = ENV_LOCK.lock().expect(LOCK_POISONED_MESSAGE);
        std::env::set_var(OTEL_EXPORTER_OTLP_ENDPOINT, "http://example.com");
        let exporter_builder = new_exporter().tonic().with_env();
        assert_eq!(
            exporter_builder.exporter_config.endpoint,
            "http://example.com"
        );
        std::env::remove_var(OTEL_EXPORTER_OTLP_ENDPOINT);
        assert!(std::env::var(OTEL_EXPORTER_OTLP_ENDPOINT).is_err());
    }

    #[test]
    fn test_pipeline_builder_from_env_protocol_http_protobuf() {
        let _env_lock = ENV_LOCK.lock().expect(LOCK_POISONED_MESSAGE);
        std::env::set_var(
            OTEL_EXPORTER_OTLP_PROTOCOL,
            OTEL_EXPORTER_OTLP_PROTOCOL_HTTP_PROTOBUF,
        );
        let exporter_builder = new_exporter().tonic().with_env();
        assert_eq!(
            exporter_builder.exporter_config.protocol,
            Protocol::HttpBinary
        );
        assert_eq!(
            exporter_builder.exporter_config.endpoint,
            OTEL_EXPORTER_OTLP_HTTP_ENDPOINT_DEFAULT
        );

        std::env::remove_var(OTEL_EXPORTER_OTLP_PROTOCOL);
        assert!(std::env::var(OTEL_EXPORTER_OTLP_PROTOCOL).is_err());
    }

    #[test]
    fn test_pipeline_builder_from_env_protocol_grpc() {
        let _env_lock = ENV_LOCK.lock().expect(LOCK_POISONED_MESSAGE);
        std::env::set_var(
            OTEL_EXPORTER_OTLP_PROTOCOL,
            OTEL_EXPORTER_OTLP_PROTOCOL_GRPC,
        );
        let exporter_builder = new_exporter().tonic().with_env();
        assert_eq!(exporter_builder.exporter_config.protocol, Protocol::Grpc);
        assert_eq!(
            exporter_builder.exporter_config.endpoint,
            OTEL_EXPORTER_OTLP_GRPC_ENDPOINT_DEFAULT
        );

        std::env::remove_var(OTEL_EXPORTER_OTLP_PROTOCOL);
        assert!(std::env::var(OTEL_EXPORTER_OTLP_PROTOCOL).is_err());
    }

    #[test]
    fn test_pipeline_builder_from_env_bad_protocol() {
        let _env_lock = ENV_LOCK.lock().expect(LOCK_POISONED_MESSAGE);
        std::env::set_var(OTEL_EXPORTER_OTLP_PROTOCOL, "bad_protocol");
        let exporter_builder = new_exporter().tonic().with_env();
        assert_eq!(
            exporter_builder.exporter_config.protocol,
            default_protocol()
        );
        assert_eq!(
            exporter_builder.exporter_config.endpoint,
            default_endpoint(default_protocol())
        );

        std::env::remove_var(OTEL_EXPORTER_OTLP_PROTOCOL);
        assert!(std::env::var(OTEL_EXPORTER_OTLP_PROTOCOL).is_err());
    }

    #[test]
    fn test_pipeline_builder_from_env_timeout() {
        let _env_lock = ENV_LOCK.lock().expect(LOCK_POISONED_MESSAGE);
        std::env::set_var(OTEL_EXPORTER_OTLP_TIMEOUT, "60");
        let exporter_builder = new_exporter().tonic().with_env();
        assert_eq!(
            exporter_builder.exporter_config.timeout,
            std::time::Duration::from_secs(60)
        );

        std::env::remove_var(OTEL_EXPORTER_OTLP_TIMEOUT);
        assert!(std::env::var(OTEL_EXPORTER_OTLP_TIMEOUT).is_err());
    }

    #[test]
    fn test_pipeline_builder_from_env_bad_timeout() {
        let _env_lock = ENV_LOCK.lock().expect(LOCK_POISONED_MESSAGE);
        std::env::set_var(OTEL_EXPORTER_OTLP_TIMEOUT, "bad_timeout");

        let exporter_builder = new_exporter().tonic().with_env();
        assert_eq!(
            exporter_builder.exporter_config.timeout,
            std::time::Duration::from_secs(OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT)
        );

        std::env::remove_var(OTEL_EXPORTER_OTLP_TIMEOUT);
        assert!(std::env::var(OTEL_EXPORTER_OTLP_TIMEOUT).is_err());
    }

    #[test]
    fn test_compression_parse() {
        assert_eq!(Compression::from_str("gzip").unwrap(), Compression::Gzip);
        Compression::from_str("bad_compression").expect_err("bad compression");
    }

    #[test]
    fn test_compression_to_str() {
        assert_eq!(Compression::Gzip.to_string(), "gzip");
    }

    #[derive(Default)]
    struct DummyExportBuilder {
        exporter_config: ExportConfig,
    }
    impl HasExportConfig for DummyExportBuilder {
        fn export_config(&mut self) -> &mut ExportConfig {
            &mut self.exporter_config
        }
    }

    // Test the examples from
    // <https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/protocol/exporter.md#endpoint-urls-for-otlphttp>
    #[test]
    fn test_resolve_endpoint() {
        let _env_lock = ENV_LOCK.lock().expect(LOCK_POISONED_MESSAGE);

        // These constants are defined in `span` and `metrics` crates, but they are only
        // built if the right features are selected. We only want to test the logic of
        // resolving env variables, so we don't need the actual functionality. So redefine
        // these here, so that the test works even without the 'logs' or 'metrics'
        // features.
        const OTEL_EXPORTER_OTLP_TRACES_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT";
        const OTEL_EXPORTER_OTLP_METRICS_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_METRICS_ENDPOINT";

        // Example 1:
        //
        // The following configuration sends all signals to the same collector:
        //
        // export OTEL_EXPORTER_OTLP_ENDPOINT=http://collector:4318
        //
        // Traces are sent to http://collector:4318/v1/traces,
        // metrics to http://collector:4318/v1/metrics and
        // logs to http://collector:4318/v1/logs.
        std::env::set_var(OTEL_EXPORTER_OTLP_ENDPOINT, "http://collector:4318");
        let builder = DummyExportBuilder::default().with_env();
        assert_eq!(
            resolve_endpoint(
                &builder.exporter_config.endpoint,
                OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
                "/v1/traces"
            ),
            "http://collector:4318/v1/traces"
        );
        assert_eq!(
            resolve_endpoint(
                &builder.exporter_config.endpoint,
                OTEL_EXPORTER_OTLP_METRICS_ENDPOINT,
                "/v1/metrics"
            ),
            "http://collector:4318/v1/metrics"
        );
        std::env::remove_var(OTEL_EXPORTER_OTLP_ENDPOINT);

        // Example 2
        //
        // Traces and metrics are sent to different collectors and paths:
        //
        // export OTEL_EXPORTER_OTLP_TRACES_ENDPOINT=http://collector:4318
        // export OTEL_EXPORTER_OTLP_METRICS_ENDPOINT=https://collector.example.com/v1/metrics
        //
        // This will send traces directly to the root path http://collector:4318/
        // (/v1/traces is only automatically added when using the non-signal-specific environment variable) and
        // metrics to https://collector.example.com/v1/metrics, using the default https port (443).
        std::env::set_var(OTEL_EXPORTER_OTLP_TRACES_ENDPOINT, "http://collector:4318");
        std::env::set_var(
            OTEL_EXPORTER_OTLP_METRICS_ENDPOINT,
            "https://collector.example.com/v1/metrics",
        );
        let builder = DummyExportBuilder::default().with_env();
        // The example says this should be "http://collector:4318/", with trailing slash. However, it earlier
        // also says that for the per-signal configuration options, "the URL is used as-is for them, without any modifications".
        assert_eq!(
            resolve_endpoint(
                &builder.exporter_config.endpoint,
                OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
                "/v1/traces"
            ),
            "http://collector:4318"
        );
        assert_eq!(
            resolve_endpoint(
                &builder.exporter_config.endpoint,
                OTEL_EXPORTER_OTLP_METRICS_ENDPOINT,
                "/v1/metrics"
            ),
            "https://collector.example.com/v1/metrics"
        );
        std::env::remove_var(OTEL_EXPORTER_OTLP_TRACES_ENDPOINT);
        std::env::remove_var(OTEL_EXPORTER_OTLP_METRICS_ENDPOINT);

        // Example 3
        // The following configuration sends all signals except for metrics to the same collector:
        //
        // export OTEL_EXPORTER_OTLP_ENDPOINT=http://collector:4318/mycollector/
        // export OTEL_EXPORTER_OTLP_METRICS_ENDPOINT=https://collector.example.com/v1/metrics/
        //
        // Traces are sent to http://collector:4318/mycollector/v1/traces,
        // logs to http://collector:4318/mycollector/v1/logs and
        // metrics to https://collector.example.com/v1/metrics/, using the default https port (443).
        // Other signals, (if there were any) would be sent to their specific paths relative to http://collector:4318/mycollector/.
        std::env::set_var(
            OTEL_EXPORTER_OTLP_ENDPOINT,
            "http://collector:4318/mycollector/",
        );
        std::env::set_var(
            OTEL_EXPORTER_OTLP_METRICS_ENDPOINT,
            "https://collector.example.com/v1/metrics/",
        );
        let builder = DummyExportBuilder::default().with_env();
        assert_eq!(
            resolve_endpoint(
                &builder.exporter_config.endpoint,
                OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
                "/v1/traces"
            ),
            "http://collector:4318/mycollector/v1/traces"
        );
        assert_eq!(
            resolve_endpoint(
                &builder.exporter_config.endpoint,
                OTEL_EXPORTER_OTLP_METRICS_ENDPOINT,
                "/v1/metrics"
            ),
            "https://collector.example.com/v1/metrics/"
        );
        std::env::remove_var(OTEL_EXPORTER_OTLP_ENDPOINT);
        std::env::remove_var(OTEL_EXPORTER_OTLP_METRICS_ENDPOINT);
    }
}
