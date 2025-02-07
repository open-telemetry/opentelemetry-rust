mod env;
mod model;
mod uploader;

use futures_core::future::BoxFuture;
use http::Uri;
use model::endpoint::Endpoint;
use opentelemetry::trace::TraceError;
use opentelemetry_http::HttpClient;
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::{trace, ExportError};
use std::net::{AddrParseError, SocketAddr};
use std::sync::Arc;

/// Zipkin span exporter
#[derive(Debug)]
pub struct ZipkinExporter {
    local_endpoint: Endpoint,
    uploader: uploader::Uploader,
}

impl ZipkinExporter {
    /// Get a builder to configure a [ZipkinExporter]
    pub fn builder() -> ZipkinExporterBuilder {
        ZipkinExporterBuilder::default()
    }

    fn new(local_endpoint: Endpoint, client: Arc<dyn HttpClient>, collector_endpoint: Uri) -> Self {
        ZipkinExporter {
            local_endpoint,
            uploader: uploader::Uploader::new(client, collector_endpoint),
        }
    }
}

/// Builder for `ZipkinExporter` struct.
#[derive(Debug)]
pub struct ZipkinExporterBuilder {
    service_addr: Option<SocketAddr>,
    collector_endpoint: String,
    client: Option<Arc<dyn HttpClient>>,
}

impl Default for ZipkinExporterBuilder {
    fn default() -> Self {
        let timeout = env::get_timeout();

        ZipkinExporterBuilder {
            #[cfg(feature = "reqwest-blocking-client")]
            client: Some(Arc::new(
                reqwest::blocking::Client::builder()
                    .timeout(timeout)
                    .build()
                    .unwrap_or_else(|_| reqwest::blocking::Client::new()),
            )),
            #[cfg(all(not(feature = "reqwest-blocking-client"), feature = "reqwest-client"))]
            client: Some(Arc::new(
                reqwest::Client::builder()
                    .timeout(timeout)
                    .build()
                    .unwrap_or_else(|_| reqwest::Client::new()),
            )),
            #[cfg(all(
                not(feature = "reqwest-client"),
                not(feature = "reqwest-blocking-client")
            ))]
            client: None,

            service_addr: None,
            collector_endpoint: env::get_endpoint(),
        }
    }
}

impl ZipkinExporterBuilder {
    /// Creates a new [ZipkinExporter] from this configuration.
    ///
    /// Returns error if the endpoint is not valid or if no http client is provided.
    pub fn build(self) -> Result<ZipkinExporter, TraceError> {
        let endpoint = Endpoint::new(self.service_addr);

        if let Some(client) = self.client {
            let exporter = ZipkinExporter::new(
                endpoint,
                client,
                self.collector_endpoint
                    .parse()
                    .map_err::<Error, _>(Into::into)?,
            );
            Ok(exporter)
        } else {
            Err(Error::NoHttpClient.into())
        }
    }

    /// Assign client implementation
    pub fn with_http_client<T: HttpClient + 'static>(mut self, client: T) -> Self {
        self.client = Some(Arc::new(client));
        self
    }

    /// Assign the service address.
    pub fn with_service_address(mut self, addr: SocketAddr) -> Self {
        self.service_addr = Some(addr);
        self
    }

    /// Assign the Zipkin collector endpoint
    pub fn with_collector_endpoint<T: Into<String>>(mut self, endpoint: T) -> Self {
        self.collector_endpoint = endpoint.into();
        self
    }
}

async fn zipkin_export(
    batch: Vec<trace::SpanData>,
    uploader: uploader::Uploader,
    local_endpoint: Endpoint,
) -> OTelSdkResult {
    let zipkin_spans = batch
        .into_iter()
        .map(|span| model::into_zipkin_span(local_endpoint.clone(), span))
        .collect();

    uploader.upload(zipkin_spans).await
}

impl trace::SpanExporter for ZipkinExporter {
    /// Export spans to Zipkin collector.
    fn export(&mut self, batch: Vec<trace::SpanData>) -> BoxFuture<'static, OTelSdkResult> {
        Box::pin(zipkin_export(
            batch,
            self.uploader.clone(),
            self.local_endpoint.clone(),
        ))
    }
}

/// Wrap type for errors from opentelemetry zipkin
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// No http client implementation found. User should provide one or enable features.
    #[error("http client must be set, users can enable reqwest feature to use http client implementation within create")]
    NoHttpClient,

    /// Http requests failed
    #[error("http request failed with {0}")]
    RequestFailed(#[from] http::Error),

    /// The uri provided is invalid
    #[error("invalid uri")]
    InvalidUri(#[from] http::uri::InvalidUri),

    /// The IP/socket address provided is invalid
    #[error("invalid address")]
    InvalidAddress(#[from] AddrParseError),

    /// Other errors
    #[error("export error: {0}")]
    Other(String),
}

impl ExportError for Error {
    fn exporter_name(&self) -> &'static str {
        "zipkin"
    }
}

impl opentelemetry::trace::ExportError for Error {
    fn exporter_name(&self) -> &'static str {
        "zipkin"
    }
}
