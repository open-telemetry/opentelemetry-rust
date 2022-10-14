use crate::{ExportConfig, Protocol};
use opentelemetry_http::HttpClient;
use std::collections::HashMap;
use std::sync::Arc;

/// Configuration of the http transport
#[cfg(feature = "http-proto")]
#[derive(Debug)]
#[cfg_attr(
    all(
        not(feature = "reqwest-client"),
        not(feature = "surf-client"),
        not(feature = "reqwest-blocking-client")
    ),
    derive(Default)
)]
pub struct HttpConfig {
    /// Select the HTTP client
    pub client: Option<Arc<dyn HttpClient>>,

    /// Additional headers to send to the collector.
    pub headers: Option<HashMap<String, String>>,
}

#[cfg(any(
    feature = "reqwest-blocking-client",
    feature = "reqwest-client",
    feature = "surf-client"
))]
impl Default for HttpConfig {
    fn default() -> Self {
        HttpConfig {
            #[cfg(feature = "reqwest-blocking-client")]
            client: Some(Arc::new(reqwest::blocking::Client::new())),
            #[cfg(all(
                not(feature = "reqwest-blocking-client"),
                not(feature = "surf-client"),
                feature = "reqwest-client"
            ))]
            client: Some(Arc::new(reqwest::Client::new())),
            #[cfg(all(
                not(feature = "reqwest-client"),
                not(feature = "reqwest-blocking-client"),
                feature = "surf-client"
            ))]
            client: Some(Arc::new(surf::Client::new())),
            #[cfg(all(
                not(feature = "reqwest-client"),
                not(feature = "surf-client"),
                not(feature = "reqwest-blocking-client")
            ))]
            client: None,
            headers: None,
        }
    }
}

/// Build a trace exporter that uses HTTP transport and opentelemetry protocol.
///
/// It provides methods to config http client.
#[derive(Debug)]
pub struct HttpExporterBuilder {
    pub(crate) exporter_config: ExportConfig,
    pub(crate) http_config: HttpConfig,
}

impl Default for HttpExporterBuilder {
    fn default() -> Self {
        let mut headers: HashMap<String, String> = HashMap::new();
        headers.insert("User-Agent".to_string(), format!("OTel OTLP Exporter Rust/{}", env!("CARGO_PKG_VERSION")));
        let mut default_config = HttpConfig::default();
        default_config.headers = Some(headers);
        HttpExporterBuilder {
            exporter_config: ExportConfig {
                protocol: Protocol::HttpBinary,
                ..ExportConfig::default()
            },
            http_config: default_config,
        }
    }
}

impl HttpExporterBuilder {
    /// Assign client implementation
    pub fn with_http_client<T: HttpClient + 'static>(mut self, client: T) -> Self {
        self.http_config.client = Some(Arc::new(client));
        self
    }

    /// Set additional headers to send to the collector.
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        // headers will be wrapped, so we must do some logic to unwrap first.
        let mut inst_headers = self.http_config.headers.unwrap_or_default();
        inst_headers.extend(headers.into_iter());
        self.http_config.headers = Some(inst_headers);
        self
    }
}
