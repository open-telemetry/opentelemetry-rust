//! OTLP exporter builder and configurations.
//!

#[cfg(feature = "grpc-sys")]
use crate::exporter::grpcio::GrpcioExporterBuilder;
#[cfg(feature = "http-proto")]
use crate::exporter::http::HttpExporterBuilder;
#[cfg(feature = "tonic")]
use crate::exporter::tonic::TonicExporterBuilder;
use crate::Protocol;
use std::str::FromStr;
use std::time::Duration;

/// Target to which the exporter is going to send spans or metrics, defaults to https://localhost:4317.
pub(crate) const OTEL_EXPORTER_OTLP_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_ENDPOINT";
/// Default target to which the exporter is going to send spans or metrics.
pub(crate) const OTEL_EXPORTER_OTLP_ENDPOINT_DEFAULT: &str = "https://localhost:4317";
/// Max waiting time for the backend to process each spans or metrics batch, defaults to 10 seconds.
pub(crate) const OTEL_EXPORTER_OTLP_TIMEOUT: &str = "OTEL_EXPORTER_OTLP_TIMEOUT";
/// Default max waiting time for the backend to process each spans or metrics batch.
pub(crate) const OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT: u64 = 10;

/// Target to which the exporter is going to send spans, defaults to https://localhost:4317.
pub(crate) const OTEL_EXPORTER_OTLP_TRACES_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT";
/// Max waiting time for the backend to process each spans batch, defaults to 10s.
pub(crate) const OTEL_EXPORTER_OTLP_TRACES_TIMEOUT: &str = "OTEL_EXPORTER_OTLP_TRACES_TIMEOUT";

#[cfg(feature = "grpc-sys")]
pub(crate) mod grpcio;
#[cfg(feature = "http-proto")]
pub(crate) mod http;
#[cfg(feature = "tonic")]
pub(crate) mod tonic;

/// Configuration for the OTLP exporter.
#[derive(Debug)]
pub struct ExportConfig {
    /// The address of the OTLP collector. If not set, the default address is used.
    pub endpoint: String,

    /// The protocol to use when communicating with the collector.
    pub protocol: Protocol,

    /// The timeout to the collector.
    pub timeout: Duration,
}

impl Default for ExportConfig {
    fn default() -> Self {
        ExportConfig {
            endpoint: OTEL_EXPORTER_OTLP_ENDPOINT_DEFAULT.to_string(),
            protocol: Protocol::Grpc,
            timeout: Duration::from_secs(OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT),
        }
    }
}

/// Provide access to the export config field within the exporter builders.
pub trait HasExportConfig {
    /// Return a mutable reference to the export config within the exporter builders.
    fn export_config(&mut self) -> &mut ExportConfig;
}

#[cfg(feature = "tonic")]
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
/// ```no_run
///
/// let exporter_builder = opentelemetry_otlp::new_exporter()
///                         .tonic()
///                         .with_endpoint("http://localhost:7201")
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
    /// Set export config.
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
        let endpoint = match std::env::var(OTEL_EXPORTER_OTLP_TRACES_ENDPOINT) {
            Ok(val) => val,
            Err(_) => std::env::var(OTEL_EXPORTER_OTLP_ENDPOINT)
                .unwrap_or_else(|_| OTEL_EXPORTER_OTLP_ENDPOINT_DEFAULT.to_string()),
        };
        self.export_config().endpoint = endpoint;

        let timeout = match std::env::var(OTEL_EXPORTER_OTLP_TRACES_TIMEOUT) {
            Ok(val) => u64::from_str(&val).unwrap_or(OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT),
            Err(_) => std::env::var(OTEL_EXPORTER_OTLP_TIMEOUT)
                .map(|val| u64::from_str(&val).unwrap_or(OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT))
                .unwrap_or(OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT),
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
#[cfg(feature = "tonic")]
mod tests {
    use crate::exporter::{
        WithExportConfig, OTEL_EXPORTER_OTLP_ENDPOINT, OTEL_EXPORTER_OTLP_TIMEOUT,
        OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT, OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
        OTEL_EXPORTER_OTLP_TRACES_TIMEOUT,
    };
    use crate::new_exporter;

    #[test]
    fn test_pipeline_builder_from_env() {
        std::env::set_var(OTEL_EXPORTER_OTLP_ENDPOINT, "https://otlp_endpoint:4317");
        std::env::set_var(OTEL_EXPORTER_OTLP_TIMEOUT, "bad_timeout");

        let mut exporter_builder = new_exporter().tonic().with_env();
        assert_eq!(
            exporter_builder.exporter_config.timeout,
            std::time::Duration::from_secs(OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT)
        );

        std::env::set_var(OTEL_EXPORTER_OTLP_TIMEOUT, "60");

        exporter_builder = new_exporter().tonic().with_env();
        assert_eq!(
            exporter_builder.exporter_config.timeout,
            std::time::Duration::from_secs(60)
        );

        std::env::remove_var(OTEL_EXPORTER_OTLP_ENDPOINT);
        std::env::remove_var(OTEL_EXPORTER_OTLP_TIMEOUT);
        assert!(std::env::var(OTEL_EXPORTER_OTLP_ENDPOINT).is_err());
        assert!(std::env::var(OTEL_EXPORTER_OTLP_TIMEOUT).is_err());

        // test from traces env var
        std::env::set_var(
            OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
            "https://otlp_traces_endpoint:4317",
        );
        std::env::set_var(OTEL_EXPORTER_OTLP_TRACES_TIMEOUT, "bad_timeout");

        let mut exporter_builder = new_exporter().tonic().with_env();
        assert_eq!(
            exporter_builder.exporter_config.timeout,
            std::time::Duration::from_secs(OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT)
        );

        std::env::set_var(OTEL_EXPORTER_OTLP_TRACES_TIMEOUT, "60");

        exporter_builder = new_exporter().tonic().with_env();
        assert_eq!(
            exporter_builder.exporter_config.timeout,
            std::time::Duration::from_secs(60)
        );

        std::env::remove_var(OTEL_EXPORTER_OTLP_TRACES_ENDPOINT);
        std::env::remove_var(OTEL_EXPORTER_OTLP_TRACES_TIMEOUT);
        assert!(std::env::var(OTEL_EXPORTER_OTLP_TRACES_ENDPOINT).is_err());
        assert!(std::env::var(OTEL_EXPORTER_OTLP_TRACES_TIMEOUT).is_err());
    }
}
