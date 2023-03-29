//! OTLP exporter builder and configurations.
//!
//! OTLP supports sending data via different protocols and formats.

#[cfg(feature = "grpc-sys")]
use crate::exporter::grpcio::GrpcioExporterBuilder;
#[cfg(any(feature = "http-proto", feature = "my-http"))]
use crate::exporter::http::HttpExporterBuilder;
#[cfg(feature = "grpc-tonic")]
use crate::exporter::tonic::TonicExporterBuilder;
use crate::Protocol;
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

/// Target to which the exporter is going to send signals, defaults to https://localhost:4317.
/// Learn about the relationship between this constant and metrics/spans/logs at
/// <https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/protocol/exporter.md#endpoint-urls-for-otlphttp>
pub const OTEL_EXPORTER_OTLP_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_ENDPOINT";
/// Default target to which the exporter is going to send signals.
pub const OTEL_EXPORTER_OTLP_ENDPOINT_DEFAULT: &str = OTEL_EXPORTER_OTLP_HTTP_ENDPOINT_DEFAULT;
/// Protocol the exporter will use. Either `http/protobuf` or `grpc`.
pub const OTEL_EXPORTER_OTLP_PROTOCOL: &str = "OTEL_EXPORTER_OTLP_PROTOCOL";

#[cfg(
    not(any(feature = "grpc-tonic", feature = "grpcio"))
)]
/// Default protocol, using http-proto.
pub const OTEL_EXPORTER_OTLP_PROTOCOL_DEFAULT: &str = OTEL_EXPORTER_OTLP_PROTOCOL_HTTP_PROTOBUF;
#[cfg(all(
    any(feature = "grpc-tonic", feature = "grpcio"),
    not(feature = "http-proto")
))]
/// Default protocol, using grpc as http-proto feature is not enabled.
pub const OTEL_EXPORTER_OTLP_PROTOCOL_DEFAULT: &str = OTEL_EXPORTER_OTLP_PROTOCOL_GRPC;

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
#[cfg(any(feature = "http-proto", feature = "my-http"))]
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
fn default_headers() -> HashMap<String, String> {
    let mut headers = HashMap::new();
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

#[cfg(any(feature = "http-proto", feature = "my-http"))]
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
/// ```no_run
/// use crate::opentelemetry_otlp::WithExportConfig;
/// let exporter_builder = opentelemetry_otlp::new_exporter()
///                         .tonic()
///                         .with_endpoint("http://localhost:7201");
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
        default_endpoint, default_protocol, WithExportConfig, OTEL_EXPORTER_OTLP_ENDPOINT,
        OTEL_EXPORTER_OTLP_GRPC_ENDPOINT_DEFAULT, OTEL_EXPORTER_OTLP_HTTP_ENDPOINT_DEFAULT,
        OTEL_EXPORTER_OTLP_PROTOCOL_GRPC, OTEL_EXPORTER_OTLP_PROTOCOL_HTTP_PROTOBUF,
        OTEL_EXPORTER_OTLP_TIMEOUT, OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT,
    };
    use crate::{new_exporter, Protocol, OTEL_EXPORTER_OTLP_PROTOCOL};
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
}
