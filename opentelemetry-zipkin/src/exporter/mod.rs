mod model;
mod uploader;

use async_trait::async_trait;
use http::Uri;
use model::endpoint::Endpoint;
use opentelemetry::{
    global, sdk,
    sdk::export::{trace, ExportError},
    trace::{TraceError, TracerProvider},
};
use opentelemetry_http::HttpClient;
use std::net::SocketAddr;

/// Default Zipkin collector endpoint
const DEFAULT_COLLECTOR_ENDPOINT: &str = "http://127.0.0.1:9411/api/v2/spans";

/// Default service name if no service is configured.
const DEFAULT_SERVICE_NAME: &str = "OpenTelemetry";

/// Zipkin span exporter
#[derive(Debug)]
pub struct Exporter {
    local_endpoint: Endpoint,
    uploader: uploader::Uploader,
}

impl Exporter {
    fn new(local_endpoint: Endpoint, client: Box<dyn HttpClient>, collector_endpoint: Uri) -> Self {
        Exporter {
            local_endpoint,
            uploader: uploader::Uploader::new(client, collector_endpoint),
        }
    }
}

/// Create a new Zipkin exporter pipeline builder.
pub fn new_pipeline() -> ZipkinPipelineBuilder<()> {
    ZipkinPipelineBuilder::default()
}

/// Builder for `ExporterConfig` struct.
#[derive(Debug)]
pub struct ZipkinPipelineBuilder<R: opentelemetry::runtime::Runtime> {
    service_name: String,
    service_addr: Option<SocketAddr>,
    collector_endpoint: String,
    trace_config: Option<sdk::trace::Config>,
    client: Option<Box<dyn HttpClient>>,
    runtime: Option<R>,
}

impl Default for ZipkinPipelineBuilder<()> {
    fn default() -> Self {
        ZipkinPipelineBuilder {
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

            service_name: DEFAULT_SERVICE_NAME.to_string(),
            service_addr: None,
            collector_endpoint: DEFAULT_COLLECTOR_ENDPOINT.to_string(),
            trace_config: None,
            runtime: None,
        }
    }
}

impl<R: opentelemetry::runtime::Runtime> ZipkinPipelineBuilder<R> {
    /// Create `ExporterConfig` struct from current `ExporterConfigBuilder`
    pub fn install(mut self) -> Result<sdk::trace::Tracer, TraceError> {
        if let Some(client) = self.client {
            let endpoint = Endpoint::new(self.service_name, self.service_addr);
            let exporter = Exporter::new(
                endpoint,
                client,
                self.collector_endpoint
                    .parse()
                    .map_err::<Error, _>(Into::into)?,
            );

            let mut provider_builder = if let Some(runtime) = self.runtime {
                sdk::trace::TracerProvider::builder().with_default_batch_exporter(exporter, runtime)
            } else {
                sdk::trace::TracerProvider::builder().with_simple_exporter(exporter)
            };
            if let Some(config) = self.trace_config.take() {
                provider_builder = provider_builder.with_config(config);
            }
            let provider = provider_builder.build();
            let tracer =
                provider.get_tracer("opentelemetry-zipkin", Some(env!("CARGO_PKG_VERSION")));
            let _ = global::set_tracer_provider(provider);

            Ok(tracer)
        } else {
            Err(Error::NoHttpClient.into())
        }
    }

    /// Assign the service name under which to group traces.
    pub fn with_service_name<T: Into<String>>(mut self, name: T) -> Self {
        self.service_name = name.into();
        self
    }

    /// Assign client implementation
    pub fn with_http_client<T: HttpClient + 'static>(mut self, client: T) -> Self {
        self.client = Some(Box::new(client));
        self
    }

    /// Assign the service name under which to group traces.
    pub fn with_service_address(mut self, addr: SocketAddr) -> Self {
        self.service_addr = Some(addr);
        self
    }

    /// Assign the Zipkin collector endpoint
    pub fn with_collector_endpoint<T: Into<String>>(mut self, endpoint: T) -> Self {
        self.collector_endpoint = endpoint.into();
        self
    }

    /// Assign the SDK trace configuration.
    pub fn with_trace_config(mut self, config: sdk::trace::Config) -> Self {
        self.trace_config = Some(config);
        self
    }

    /// Assign the runtime to use.
    ///
    /// Please make sure the selected HTTP client works with the runtime.
    pub fn with_runtime<NewR: opentelemetry::runtime::Runtime>(
        self,
        runtime: NewR,
    ) -> ZipkinPipelineBuilder<NewR> {
        ZipkinPipelineBuilder {
            client: self.client,
            service_name: self.service_name,
            service_addr: self.service_addr,
            collector_endpoint: self.collector_endpoint,
            trace_config: self.trace_config,
            runtime: Some(runtime),
        }
    }
}

#[async_trait]
impl trace::SpanExporter for Exporter {
    /// Export spans to Zipkin collector.
    async fn export(&mut self, batch: Vec<trace::SpanData>) -> trace::ExportResult {
        let zipkin_spans = batch
            .into_iter()
            .map(|span| model::into_zipkin_span(self.local_endpoint.clone(), span))
            .collect();

        self.uploader.upload(zipkin_spans).await
    }
}

/// Wrap type for errors from opentelemetry zipkin
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// No http client implementation found. User should provide one or enable features.
    #[error("http client must be set, users can enable reqwest or surf feature to use http client implementation within create")]
    NoHttpClient,

    /// Http requests failed
    #[error("http request failed with {0}")]
    RequestFailed(#[from] http::Error),

    /// The uri provided is invalid
    #[error("invalid uri")]
    InvalidUri(#[from] http::uri::InvalidUri),

    /// Other errors
    #[error("export error: {0}")]
    Other(String),
}

impl ExportError for Error {
    fn exporter_name(&self) -> &'static str {
        "zipkin"
    }
}
