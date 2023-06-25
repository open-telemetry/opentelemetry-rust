//! OTLP exporter builder and configurations.
//!
//! OTLP supports sending data via different protocols and formats.

#[cfg(feature = "grpc-sys")]
use crate::exporter::grpcio::GrpcioExporterBuilder;
#[cfg(feature = "http-proto")]
use crate::exporter::http::HttpExporterBuilder;
#[cfg(feature = "grpc-tonic")]
use crate::exporter::tonic::TonicExporterBuilder;
use crate::Protocol;
use std::collections::HashMap;
use std::ffi::OsString;
use std::time::Duration;

/// Target to which the exporter is going to send signals, defaults to https://localhost:4317.
/// Learn about the relationship between this constant and metrics/spans/logs at
/// <https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/protocol/exporter.md#endpoint-urls-for-otlphttp>
pub const OTEL_EXPORTER_OTLP_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_ENDPOINT";
/// Default target to which the exporter is going to send signals.
pub const OTEL_EXPORTER_OTLP_ENDPOINT_DEFAULT: &str = OTEL_EXPORTER_OTLP_HTTP_ENDPOINT_DEFAULT;
/// Protocol the exporter will use. Either `http/protobuf` or `grpc`.
pub const OTEL_EXPORTER_OTLP_PROTOCOL: &str = "OTEL_EXPORTER_OTLP_PROTOCOL";

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

/// Env key: OTEL_EXPORTER_OTLP_INSECURE
pub const OTEL_EXPORTER_OTLP_INSECURE: &str = "OTEL_EXPORTER_OTLP_INSECURE";
/// Env key: OTEL_EXPORTER_OTLP_CERTIFICATE
pub const OTEL_EXPORTER_OTLP_CERTIFICATE: &str = "OTEL_EXPORTER_OTLP_CERTIFICATE";
/// Env key: OTEL_EXPORTER_OTLP_CLIENT_KEY
pub const OTEL_EXPORTER_OTLP_CLIENT_KEY: &str = "OTEL_EXPORTER_OTLP_CLIENT_KEY";
/// Env key: OTEL_EXPORTER_OTLP_CLIENT_CERTIFICATE
pub const OTEL_EXPORTER_OTLP_CLIENT_CERTIFICATE: &str = "OTEL_EXPORTER_OTLP_CLIENT_CERTIFICATE";
/// Env key: OTEL_EXPORTER_OTLP_HEADERS
pub const OTEL_EXPORTER_OTLP_HEADERS: &str = "OTEL_EXPORTER_OTLP_HEADERS";

#[cfg(feature = "trace")]
mod traces_headers {
    /// Env key: OTEL_EXPORTER_OTLP_TRACES_ENDPOINT
    pub const OTEL_EXPORTER_OTLP_TRACES_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT";
    /// Env key: OTEL_EXPORTER_OTLP_TRACES_TIMEOUT
    pub const OTEL_EXPORTER_OTLP_TRACES_TIMEOUT: &str = "OTEL_EXPORTER_OTLP_TRACES_TIMEOUT";
    /// Env key: OTEL_EXPORTER_OTLP_TRACES_INSECURE
    pub const OTEL_EXPORTER_OTLP_TRACES_INSECURE: &str = "OTEL_EXPORTER_OTLP_TRACES_INSECURE";
    /// Env key: OTEL_EXPORTER_OTLP_TRACES_CERTIFICATE
    pub const OTEL_EXPORTER_OTLP_TRACES_CERTIFICATE: &str = "OTEL_EXPORTER_OTLP_TRACES_CERTIFICATE";
    /// Env key: OTEL_EXPORTER_OTLP_TRACES_CLIENT_KEY
    pub const OTEL_EXPORTER_OTLP_TRACES_CLIENT_KEY: &str = "OTEL_EXPORTER_OTLP_TRACES_CLIENT_KEY";
    /// Env key: OTEL_EXPORTER_OTLP_TRACES_CLIENT_CERTIFICATE
    pub const OTEL_EXPORTER_OTLP_TRACES_CLIENT_CERTIFICATE: &str =
        "OTEL_EXPORTER_OTLP_TRACES_CLIENT_CERTIFICATE";
    /// Env key: OTEL_EXPORTER_OTLP_TRACES_HEADERS
    pub const OTEL_EXPORTER_OTLP_TRACES_HEADERS: &str = "OTEL_EXPORTER_OTLP_TRACES_HEADERS";
    /// Env key: OTEL_EXPORTER_OTLP_TRACES_PROTOCOL
    pub const OTEL_EXPORTER_OTLP_TRACES_PROTOCOL: &str = "OTEL_EXPORTER_OTLP_TRACES_PROTOCOL";
}
#[cfg(feature = "trace")]
pub use traces_headers::*;

#[cfg(feature = "metrics")]
mod metrics_headers {
    /// Env key: OTEL_EXPORTER_OTLP_METRICS_ENDPOINT
    pub const OTEL_EXPORTER_OTLP_METRICS_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_METRICS_ENDPOINT";
    /// Env key: OTEL_EXPORTER_OTLP_METRICS_TIMEOUT
    pub const OTEL_EXPORTER_OTLP_METRICS_TIMEOUT: &str = "OTEL_EXPORTER_OTLP_METRICS_TIMEOUT";
    /// Env key: OTEL_EXPORTER_OTLP_METRICS_INSECURE
    pub const OTEL_EXPORTER_OTLP_METRICS_INSECURE: &str = "OTEL_EXPORTER_OTLP_METRICS_INSECURE";
    /// Env key: OTEL_EXPORTER_OTLP_METRICS_CERTIFICATE
    pub const OTEL_EXPORTER_OTLP_METRICS_CERTIFICATE: &str =
        "OTEL_EXPORTER_OTLP_METRICS_CERTIFICATE";
    /// Env key: OTEL_EXPORTER_OTLP_METRICS_CLIENT_KEY
    pub const OTEL_EXPORTER_OTLP_METRICS_CLIENT_KEY: &str = "OTEL_EXPORTER_OTLP_METRICS_CLIENT_KEY";
    /// Env key: OTEL_EXPORTER_OTLP_METRICS_CLIENT_CERTIFICATE
    pub const OTEL_EXPORTER_OTLP_METRICS_CLIENT_CERTIFICATE: &str =
        "OTEL_EXPORTER_OTLP_METRICS_CLIENT_CERTIFICATE";
    /// Env key: OTEL_EXPORTER_OTLP_METRICS_HEADERS
    pub const OTEL_EXPORTER_OTLP_METRICS_HEADERS: &str = "OTEL_EXPORTER_OTLP_METRICS_HEADERS";
    /// Env key: OTEL_EXPORTER_OTLP_METRICS_PROTOCOL
    pub const OTEL_EXPORTER_OTLP_METRICS_PROTOCOL: &str = "OTEL_EXPORTER_OTLP_METRICS_PROTOCOL";
}
#[cfg(feature = "metrics")]
pub use metrics_headers::*;

#[cfg(feature = "logs")]
mod logs_headers {
    /// Env key: OTEL_EXPORTER_OTLP_LOGS_ENDPOINT
    pub const OTEL_EXPORTER_OTLP_LOGS_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_LOGS_ENDPOINT";
    /// Env key: OTEL_EXPORTER_OTLP_LOGS_TIMEOUT
    pub const OTEL_EXPORTER_OTLP_LOGS_TIMEOUT: &str = "OTEL_EXPORTER_OTLP_LOGS_TIMEOUT";
    /// Env key: OTEL_EXPORTER_OTLP_LOGS_INSECURE
    pub const OTEL_EXPORTER_OTLP_LOGS_INSECURE: &str = "OTEL_EXPORTER_OTLP_LOGS_INSECURE";
    /// Env key: OTEL_EXPORTER_OTLP_LOGS_CERTIFICATE
    pub const OTEL_EXPORTER_OTLP_LOGS_CERTIFICATE: &str = "OTEL_EXPORTER_OTLP_LOGS_CERTIFICATE";
    /// Env key: OTEL_EXPORTER_OTLP_LOGS_CLIENT_KEY
    pub const OTEL_EXPORTER_OTLP_LOGS_CLIENT_KEY: &str = "OTEL_EXPORTER_OTLP_LOGS_CLIENT_KEY";
    /// Env key: OTEL_EXPORTER_OTLP_LOGS_CLIENT_CERTIFICATE
    pub const OTEL_EXPORTER_OTLP_LOGS_CLIENT_CERTIFICATE: &str =
        "OTEL_EXPORTER_OTLP_LOGS_CLIENT_CERTIFICATE";
    /// Env key: OTEL_EXPORTER_OTLP_LOGS_HEADERS
    pub const OTEL_EXPORTER_OTLP_LOGS_HEADERS: &str = "OTEL_EXPORTER_OTLP_LOGS_HEADERS";
    /// Env key: OTEL_EXPORTER_OTLP_LOGS_PROTOCOL
    pub const OTEL_EXPORTER_OTLP_LOGS_PROTOCOL: &str = "OTEL_EXPORTER_OTLP_LOGS_PROTOCOL";
}
#[cfg(feature = "logs")]
pub use logs_headers::*;

#[cfg(feature = "grpc-sys")]
pub(crate) mod grpcio;
#[cfg(feature = "http-proto")]
pub(crate) mod http;
#[cfg(feature = "grpc-tonic")]
pub(crate) mod tonic;

macro_rules! set_from_env_with_default {
    ($ident:expr,$env_name:ident,$fn:expr,$default:expr) => {
        $ident = match std::env::var_os($env_name) {
            Some(v) => match $fn(v) {
                Some(v) => v,
                None => $default,
            },
            None => $default,
        }
    };
}

macro_rules! set_from_env {
    ($ident:expr,$env_name:ident,$fn:expr) => {
        if let Some(v) = std::env::var_os($env_name) {
            if let Some(v) = $fn(v) {
                $ident = v;
            }
        }
    };
}

/// The type of data for the OTLP exporter.
#[derive(Debug)]
pub enum DataType {
    /// Trace data.
    #[cfg(feature = "trace")]
    Trace,

    /// Metric data.
    #[cfg(feature = "metrics")]
    Metric,

    /// Log data.
    #[cfg(feature = "logs")]
    Log,
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

    /// It represents whether to enable client transport security for gRPC requests. A scheme of https
    /// takes precedence over this configuration setting. Default: false.
    pub insecure: bool,

    /// The path to the file containing trusted certificates to use when verifying an OTLP trace,
    /// metric, or log server's TLS credentials. The file should contain one or more X.509
    /// certificates in PEM format. By default the host platform's trusted root certificates are
    /// used.
    pub certificate: Option<OsString>,

    /// The path to the file containing private client key to use when verifying an OTLP trace,
    /// metric, or log client's TLS credentials. The file should contain one private key PKCS8 PEM
    /// format. By default no client key is used.
    pub client_key: Option<OsString>,

    /// The path to the file containing trusted certificates to use when verifying an OTLP trace,
    /// metric, or log client's TLS credentials. The file should contain one or more X.509
    /// certificates in PEM format. By default no chain file is used.
    pub client_certificate: Option<OsString>,

    /// Key-value pairs separated by commas to pass as request headers on OTLP trace, metric, and
    /// log requests.
    pub headers: Option<HashMap<String, String>>,
}

impl Default for ExportConfig {
    fn default() -> Self {
        let protocol = default_protocol();

        ExportConfig {
            endpoint: default_endpoint(protocol),
            protocol,
            timeout: Duration::from_secs(OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT),
            insecure: false,
            certificate: None,
            client_key: None,
            client_certificate: None,
            headers: Some(default_headers()),
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
    /// Specify the type of data. It is used for reading env. If not set, only common variable will be used.
    fn with_env_ext(self, data_type: Option<DataType>) -> Self;
    /// Set whether to enable client transport security for gRPC requests. A scheme of https
    /// takes precedence over this configuration setting.
    fn with_insecure(self, insecure: bool) -> Self;
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

    fn with_env(self) -> Self {
        self.with_env_ext(None)
    }

    fn with_env_ext(mut self, data_type: Option<DataType>) -> Self {
        macro_rules! set_all_from_env {
            ($c:ident,$timeout_key:ident,$insecure_key:ident,$certificate_key:ident,$client_key_key:ident,$client_certificate_key:ident,$headers_key:ident) => {
                set_from_env!($c.timeout, $timeout_key, parse_duration);
                set_from_env!($c.insecure, $insecure_key, parse_bool);
                set_from_env!($c.certificate, $certificate_key, |v| Some(Some(v)));
                set_from_env!($c.client_key, $client_key_key, |v| Some(Some(v)));
                set_from_env!($c.client_certificate, $client_certificate_key, |v| Some(
                    Some(v)
                ));
                set_from_env!($c.headers, $headers_key, parse_headers);
            };
        }

        fn parse_protocol(p: OsString) -> Option<Protocol> {
            match p.to_str()? {
                OTEL_EXPORTER_OTLP_PROTOCOL_HTTP_PROTOBUF => Some(Protocol::HttpBinary),
                OTEL_EXPORTER_OTLP_PROTOCOL_GRPC => Some(Protocol::Grpc),
                _ => None,
            }
        }

        fn parse_duration(t: OsString) -> Option<Duration> {
            t.to_str()?.parse().ok().map(Duration::from_secs)
        }

        fn parse_bool(b: OsString) -> Option<bool> {
            b.to_str()?.parse().ok()
        }

        fn parse_headers(h: OsString) -> Option<Option<HashMap<String, String>>> {
            // Parse headers according to:
            // https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/protocol/exporter.md#specifying-headers-via-environment-variables
            let items = h.to_str()?.split(',');
            let mut res = default_headers();
            for item in items {
                if let Some((key, value)) = item.split_once('=') {
                    let key = key.trim();
                    if key.is_empty() {
                        continue;
                    }
                    res.insert(key.to_string(), value.trim().to_string());
                }
            }
            Some(Some(res))
        }

        let export_config = self.export_config();
        // Set default
        export_config.timeout = Duration::from_secs(OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT);
        export_config.insecure = false;

        let protocol;
        set_from_env_with_default!(
            protocol,
            OTEL_EXPORTER_OTLP_PROTOCOL,
            parse_protocol,
            default_protocol()
        );
        export_config.protocol = protocol;

        set_from_env_with_default!(
            export_config.endpoint,
            OTEL_EXPORTER_OTLP_ENDPOINT,
            |e: OsString| e.to_str().map(ToString::to_string),
            default_endpoint(export_config.protocol)
        );

        set_all_from_env!(
            export_config,
            OTEL_EXPORTER_OTLP_TIMEOUT,
            OTEL_EXPORTER_OTLP_INSECURE,
            OTEL_EXPORTER_OTLP_CERTIFICATE,
            OTEL_EXPORTER_OTLP_CLIENT_KEY,
            OTEL_EXPORTER_OTLP_CLIENT_CERTIFICATE,
            OTEL_EXPORTER_OTLP_HEADERS
        );

        match data_type {
            #[cfg(feature = "trace")]
            Some(DataType::Trace) => {
                let trace_protocol;
                set_from_env_with_default!(
                    trace_protocol,
                    OTEL_EXPORTER_OTLP_TRACES_PROTOCOL,
                    parse_protocol,
                    protocol
                );
                export_config.protocol = trace_protocol;

                set_from_env_with_default!(
                    export_config.endpoint,
                    OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
                    |e: OsString| e.to_str().map(ToString::to_string),
                    if trace_protocol == protocol {
                        format!("{}/v1/traces", export_config.endpoint.trim_end_matches('/'))
                    } else {
                        format!("{}/v1/traces", default_endpoint(trace_protocol).trim_end_matches('/'))
                    }
                );

                set_all_from_env!(
                    export_config,
                    OTEL_EXPORTER_OTLP_TRACES_TIMEOUT,
                    OTEL_EXPORTER_OTLP_TRACES_INSECURE,
                    OTEL_EXPORTER_OTLP_TRACES_CERTIFICATE,
                    OTEL_EXPORTER_OTLP_TRACES_CLIENT_KEY,
                    OTEL_EXPORTER_OTLP_TRACES_CLIENT_CERTIFICATE,
                    OTEL_EXPORTER_OTLP_TRACES_HEADERS
                );
            },
            #[cfg(feature = "metrics")]
            Some(DataType::Metric) => {
                let metrics_protocol;
                set_from_env_with_default!(
                    metrics_protocol,
                    OTEL_EXPORTER_OTLP_METRICS_PROTOCOL,
                    parse_protocol,
                    protocol
                );
                export_config.protocol = metrics_protocol;

                set_from_env_with_default!(
                    export_config.endpoint,
                    OTEL_EXPORTER_OTLP_METRICS_ENDPOINT,
                    |e: OsString| e.to_str().map(ToString::to_string),
                    if metrics_protocol == protocol {
                        format!("{}/v1/metrics", export_config.endpoint.trim_end_matches('/'))
                    } else {
                        format!("{}/v1/metrics", default_endpoint(metrics_protocol).trim_end_matches('/'))
                    }
                );

                set_all_from_env!(
                    export_config,
                    OTEL_EXPORTER_OTLP_METRICS_TIMEOUT,
                    OTEL_EXPORTER_OTLP_METRICS_INSECURE,
                    OTEL_EXPORTER_OTLP_METRICS_CERTIFICATE,
                    OTEL_EXPORTER_OTLP_METRICS_CLIENT_KEY,
                    OTEL_EXPORTER_OTLP_METRICS_CLIENT_CERTIFICATE,
                    OTEL_EXPORTER_OTLP_METRICS_HEADERS
                );
            },
            #[cfg(feature = "logs")]
            Some(DataType::Log) => {
                let logs_protocol;
                set_from_env_with_default!(
                    logs_protocol,
                    OTEL_EXPORTER_OTLP_LOGS_PROTOCOL,
                    parse_protocol,
                    protocol
                );
                export_config.protocol = logs_protocol;

                set_from_env_with_default!(
                    export_config.endpoint,
                    OTEL_EXPORTER_OTLP_LOGS_ENDPOINT,
                    |e: OsString| e.to_str().map(ToString::to_string),
                    if logs_protocol == protocol {
                        format!("{}/v1/logs", export_config.endpoint.trim_end_matches('/'))
                    } else {
                        format!("{}/v1/logs", default_endpoint(logs_protocol).trim_end_matches('/'))
                    }
                );

                set_all_from_env!(
                    export_config,
                    OTEL_EXPORTER_OTLP_LOGS_TIMEOUT,
                    OTEL_EXPORTER_OTLP_LOGS_INSECURE,
                    OTEL_EXPORTER_OTLP_LOGS_CERTIFICATE,
                    OTEL_EXPORTER_OTLP_LOGS_CLIENT_KEY,
                    OTEL_EXPORTER_OTLP_LOGS_CLIENT_CERTIFICATE,
                    OTEL_EXPORTER_OTLP_LOGS_HEADERS
                );
            },
            #[cfg(not(any(feature = "trace", feature = "metrics", feature = "logs")))]
            Some(_) => unreachable!("DataType should be an empty enum when all of `trace`, `metrics`, `logs` are disabled"),
            None => {}
        }
        self
    }

    fn with_insecure(mut self, insecure: bool) -> Self {
        self.export_config().insecure = insecure;
        self
    }

    fn with_export_config(mut self, other: ExportConfig) -> Self {
        let export_config = self.export_config();
        export_config.endpoint = other.endpoint;
        export_config.protocol = other.protocol;
        export_config.timeout = other.timeout;
        export_config.insecure = other.insecure;
        export_config.certificate = other.certificate;
        export_config.client_key = other.client_key;
        export_config.client_certificate = other.client_certificate;
        export_config.headers = other.headers;
        self
    }
}

#[cfg(test)]
#[cfg(feature = "grpc-tonic")]
mod tests {
    use crate::exporter::{
        default_endpoint, default_headers, default_protocol, WithExportConfig,
        OTEL_EXPORTER_OTLP_GRPC_ENDPOINT_DEFAULT, OTEL_EXPORTER_OTLP_HTTP_ENDPOINT_DEFAULT,
        OTEL_EXPORTER_OTLP_PROTOCOL_GRPC, OTEL_EXPORTER_OTLP_PROTOCOL_HTTP_PROTOBUF,
    };
    #[cfg(any(feature = "trace", feature = "metrics", feature = "logs"))]
    use crate::DataType;
    use crate::{
        new_exporter, Protocol, OTEL_EXPORTER_OTLP_ENDPOINT, OTEL_EXPORTER_OTLP_HEADERS,
        OTEL_EXPORTER_OTLP_PROTOCOL, OTEL_EXPORTER_OTLP_TIMEOUT,
        OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT,
    };
    #[cfg(feature = "logs")]
    use crate::{OTEL_EXPORTER_OTLP_LOGS_ENDPOINT, OTEL_EXPORTER_OTLP_LOGS_HEADERS};
    #[cfg(feature = "metrics")]
    use crate::{OTEL_EXPORTER_OTLP_METRICS_ENDPOINT, OTEL_EXPORTER_OTLP_METRICS_HEADERS};
    #[cfg(feature = "trace")]
    use crate::{OTEL_EXPORTER_OTLP_TRACES_ENDPOINT, OTEL_EXPORTER_OTLP_TRACES_HEADERS};

    #[test]
    fn test_pipeline_builder_from_env_default_vars() {
        let kvs: Vec<(&str, Option<&str>)> = vec![];
        temp_env::with_vars(kvs, || {
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
        });
    }

    #[test]
    fn test_pipeline_builder_from_env_endpoint() {
        let expected_endpoint = "https://test_endpoint_from_env:4317".to_string();
        temp_env::with_vars(
            vec![
                (OTEL_EXPORTER_OTLP_ENDPOINT, Some(&expected_endpoint)),
                #[cfg(feature = "trace")]
                (OTEL_EXPORTER_OTLP_TRACES_ENDPOINT, None),
                #[cfg(feature = "metrics")]
                (OTEL_EXPORTER_OTLP_METRICS_ENDPOINT, None),
                #[cfg(feature = "logs")]
                (OTEL_EXPORTER_OTLP_LOGS_ENDPOINT, None),
            ],
            || {
                // with_env == with_env_ext(None)
                assert_eq!(
                    new_exporter().tonic().with_env().exporter_config.endpoint,
                    new_exporter()
                        .tonic()
                        .with_env_ext(None)
                        .exporter_config
                        .endpoint,
                );
                // endpoint without specifying the data type
                assert_eq!(
                    new_exporter().tonic().with_env().exporter_config.endpoint,
                    expected_endpoint,
                );
                // endpoint for traces
                #[cfg(feature = "trace")]
                {
                    assert_eq!(
                        new_exporter()
                            .tonic()
                            .with_env_ext(Some(DataType::Trace))
                            .exporter_config
                            .endpoint,
                        format!("{}/v1/traces", expected_endpoint),
                    );
                    std::env::set_var(OTEL_EXPORTER_OTLP_TRACES_ENDPOINT, &expected_endpoint);
                    assert_eq!(
                        new_exporter()
                            .tonic()
                            .with_env_ext(Some(DataType::Trace))
                            .exporter_config
                            .endpoint,
                        expected_endpoint,
                    );
                }

                // endpoint for metrics
                #[cfg(feature = "metrics")]
                {
                    assert_eq!(
                        new_exporter()
                            .tonic()
                            .with_env_ext(Some(super::DataType::Metric))
                            .exporter_config
                            .endpoint,
                        format!("{}/v1/metrics", expected_endpoint),
                    );
                    std::env::set_var(OTEL_EXPORTER_OTLP_METRICS_ENDPOINT, &expected_endpoint);
                    assert_eq!(
                        new_exporter()
                            .tonic()
                            .with_env_ext(Some(super::DataType::Metric))
                            .exporter_config
                            .endpoint,
                        expected_endpoint,
                    );
                }

                // endpoint for logs
                #[cfg(feature = "logs")]
                {
                    assert_eq!(
                        new_exporter()
                            .tonic()
                            .with_env_ext(Some(super::DataType::Log))
                            .exporter_config
                            .endpoint,
                        format!("{}/v1/logs", expected_endpoint),
                    );
                    std::env::set_var(OTEL_EXPORTER_OTLP_LOGS_ENDPOINT, &expected_endpoint);
                    assert_eq!(
                        new_exporter()
                            .tonic()
                            .with_env_ext(Some(super::DataType::Log))
                            .exporter_config
                            .endpoint,
                        expected_endpoint,
                    );
                }

                // test if trailing '/' will be trimmed.
                std::env::set_var(
                    OTEL_EXPORTER_OTLP_ENDPOINT,
                    format!("{}/", expected_endpoint),
                );
                #[cfg(feature = "trace")]
                {
                    std::env::remove_var(OTEL_EXPORTER_OTLP_TRACES_ENDPOINT);
                    assert_eq!(
                        new_exporter()
                            .tonic()
                            .with_env_ext(Some(DataType::Trace))
                            .exporter_config
                            .endpoint,
                        format!("{}/v1/traces", expected_endpoint),
                    );
                }

                #[cfg(feature = "metrics")]
                {
                    std::env::remove_var(OTEL_EXPORTER_OTLP_METRICS_ENDPOINT);
                    assert_eq!(
                        new_exporter()
                            .tonic()
                            .with_env_ext(Some(DataType::Metric))
                            .exporter_config
                            .endpoint,
                        format!("{}/v1/metrics", expected_endpoint),
                    );
                }

                #[cfg(feature = "logs")]
                {
                    std::env::remove_var(OTEL_EXPORTER_OTLP_LOGS_ENDPOINT);
                    assert_eq!(
                        new_exporter()
                            .tonic()
                            .with_env_ext(Some(DataType::Log))
                            .exporter_config
                            .endpoint,
                        format!("{}/v1/logs", expected_endpoint),
                    );
                }
            },
        );
    }

    #[test]
    fn test_pipeline_builder_from_env_protocol_http_protobuf() {
        temp_env::with_vars(
            vec![(
                OTEL_EXPORTER_OTLP_PROTOCOL,
                Some(OTEL_EXPORTER_OTLP_PROTOCOL_HTTP_PROTOBUF),
            )],
            || {
                let exporter_builder = new_exporter().tonic().with_env();
                assert_eq!(
                    exporter_builder.exporter_config.protocol,
                    Protocol::HttpBinary
                );
                assert_eq!(
                    exporter_builder.exporter_config.endpoint,
                    OTEL_EXPORTER_OTLP_HTTP_ENDPOINT_DEFAULT
                );
            },
        );
    }

    #[test]
    fn test_pipeline_builder_from_env_protocol_grpc() {
        temp_env::with_vars(
            vec![(
                OTEL_EXPORTER_OTLP_PROTOCOL,
                Some(OTEL_EXPORTER_OTLP_PROTOCOL_GRPC),
            )],
            || {
                let exporter_builder = new_exporter().tonic().with_env();
                assert_eq!(exporter_builder.exporter_config.protocol, Protocol::Grpc);
                assert_eq!(
                    exporter_builder.exporter_config.endpoint,
                    OTEL_EXPORTER_OTLP_GRPC_ENDPOINT_DEFAULT
                );
            },
        );
    }

    #[test]
    fn test_pipeline_builder_from_env_bad_protocol() {
        temp_env::with_vars(
            vec![(OTEL_EXPORTER_OTLP_PROTOCOL, Some("bad_protocol"))],
            || {
                let exporter_builder = new_exporter().tonic().with_env();
                assert_eq!(
                    exporter_builder.exporter_config.protocol,
                    default_protocol()
                );
                assert_eq!(
                    exporter_builder.exporter_config.endpoint,
                    default_endpoint(default_protocol())
                );
            },
        );
    }

    #[test]
    fn test_pipeline_builder_from_env_timeout() {
        temp_env::with_vars(vec![(OTEL_EXPORTER_OTLP_TIMEOUT, Some("60"))], || {
            let exporter_builder = new_exporter().tonic().with_env();
            assert_eq!(
                exporter_builder.exporter_config.timeout,
                std::time::Duration::from_secs(60)
            );
        });
    }

    #[test]
    fn test_pipeline_builder_from_env_bad_timeout() {
        temp_env::with_vars(
            vec![(OTEL_EXPORTER_OTLP_TIMEOUT, Some("bad_timeout"))],
            || {
                let exporter_builder = new_exporter().tonic().with_env();
                assert_eq!(
                    exporter_builder.exporter_config.timeout,
                    std::time::Duration::from_secs(OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT)
                );
            },
        );
    }

    #[test]
    fn test_headers_from_env() {
        let mut otlp_headers = default_headers();
        otlp_headers.insert("Accept".to_owned(), "text/plain".to_owned());
        let mut otlp_traces_headers = default_headers();
        otlp_traces_headers.insert("Accept".to_owned(), "application/json".to_owned());
        let mut otlp_metrics_headers = default_headers();
        otlp_metrics_headers.insert("Accept".to_owned(), "application/xml".to_owned());
        let mut otlp_logs_headers = default_headers();
        otlp_logs_headers.insert("Accept".to_owned(), "application/html".to_owned());
        temp_env::with_vars(
            vec![
                (OTEL_EXPORTER_OTLP_HEADERS, None::<String>),
                #[cfg(feature = "trace")]
                (OTEL_EXPORTER_OTLP_TRACES_HEADERS, None),
                #[cfg(feature = "metrics")]
                (OTEL_EXPORTER_OTLP_METRICS_HEADERS, None),
                #[cfg(feature = "logs")]
                (OTEL_EXPORTER_OTLP_LOGS_HEADERS, None),
            ],
            || {
                assert_eq!(
                    new_exporter().tonic().with_env().exporter_config.headers,
                    Some(default_headers())
                );

                #[cfg(feature = "trace")]
                assert_eq!(
                    new_exporter()
                        .tonic()
                        .with_env_ext(Some(DataType::Trace))
                        .exporter_config
                        .headers,
                    Some(default_headers())
                );
                #[cfg(feature = "metrics")]
                assert_eq!(
                    new_exporter()
                        .tonic()
                        .with_env_ext(Some(DataType::Metric))
                        .exporter_config
                        .headers,
                    Some(default_headers())
                );
                #[cfg(feature = "logs")]
                assert_eq!(
                    new_exporter()
                        .tonic()
                        .with_env_ext(Some(DataType::Log))
                        .exporter_config
                        .headers,
                    Some(default_headers())
                );

                std::env::set_var(OTEL_EXPORTER_OTLP_HEADERS, "Accept=text/plain");
                assert_eq!(
                    new_exporter().tonic().with_env().exporter_config.headers,
                    Some(otlp_headers.clone())
                );

                #[cfg(feature = "trace")]
                assert_eq!(
                    new_exporter()
                        .tonic()
                        .with_env_ext(Some(DataType::Trace))
                        .exporter_config
                        .headers,
                    Some(otlp_headers.clone())
                );
                #[cfg(feature = "metrics")]
                assert_eq!(
                    new_exporter()
                        .tonic()
                        .with_env_ext(Some(DataType::Metric))
                        .exporter_config
                        .headers,
                    Some(otlp_headers.clone())
                );
                #[cfg(feature = "logs")]
                assert_eq!(
                    new_exporter()
                        .tonic()
                        .with_env_ext(Some(DataType::Log))
                        .exporter_config
                        .headers,
                    Some(otlp_headers.clone())
                );

                #[cfg(feature = "trace")]
                std::env::set_var(OTEL_EXPORTER_OTLP_TRACES_HEADERS, "Accept=application/json");
                #[cfg(feature = "metrics")]
                std::env::set_var(OTEL_EXPORTER_OTLP_METRICS_HEADERS, "Accept=application/xml");
                #[cfg(feature = "logs")]
                std::env::set_var(OTEL_EXPORTER_OTLP_LOGS_HEADERS, "Accept=application/html");
                assert_eq!(
                    new_exporter().tonic().with_env().exporter_config.headers,
                    Some(otlp_headers.clone())
                );

                #[cfg(feature = "trace")]
                assert_eq!(
                    new_exporter()
                        .tonic()
                        .with_env_ext(Some(DataType::Trace))
                        .exporter_config
                        .headers,
                    Some(otlp_traces_headers.clone())
                );
                #[cfg(feature = "metrics")]
                assert_eq!(
                    new_exporter()
                        .tonic()
                        .with_env_ext(Some(DataType::Metric))
                        .exporter_config
                        .headers,
                    Some(otlp_metrics_headers.clone())
                );
                #[cfg(feature = "logs")]
                assert_eq!(
                    new_exporter()
                        .tonic()
                        .with_env_ext(Some(DataType::Log))
                        .exporter_config
                        .headers,
                    Some(otlp_logs_headers.clone())
                );
            },
        );
    }
}
