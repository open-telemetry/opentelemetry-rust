use opentelemetry_http::HttpClient;
use std::collections::HashMap;
use crate::{ExportConfig, Protocol};

/// Configuration of the http transport
#[cfg(feature = "http-proto")]
#[derive(Debug)]
pub struct HttpConfig {
    /// Select the HTTP client
    pub client: Option<Box<dyn HttpClient>>,

    /// Additional headers to send to the collector.
    pub headers: Option<HashMap<String, String>>,
}

impl Default for HttpConfig {
    fn default() -> Self {
        HttpConfig {
            #[cfg(feature = "reqwest-blocking-client")]
            client: Some(Box::new(reqwest::blocking::Client::new())),
            #[cfg(all(
            not(feature = "reqwest-blocking-client"),
            not(feature = "surf-client"),
            feature = "reqwest-client"
            ))]
            client: Some(Box::new(reqwest::Client::new())),
            #[cfg(all(
            not(feature = "reqwest-client"),
            not(feature = "reqwest-blocking-client"),
            feature = "surf-client"
            ))]
            client: Some(Box::new(surf::Client::new())),
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
        HttpExporterBuilder{
            exporter_config: ExportConfig {
                protocol: Protocol::HttpBinary,
                ..ExportConfig::default()
            },
            http_config: HttpConfig::default(),
        }
    }
}

impl HttpExporterBuilder {
    /// Assign client implementation
    pub fn with_http_client<T: HttpClient + 'static>(mut self, client: T) -> Self {
        self.http_config.client = Some(Box::new(client));
        self
    }

    /// Set additional headers to send to the collector.
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.http_config.headers = Some(headers);
        self
    }
}