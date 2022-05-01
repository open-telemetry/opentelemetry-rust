mod env;
mod model;
mod uploader;

use async_trait::async_trait;
use http::Uri;
use model::endpoint::Endpoint;
use opentelemetry::sdk::resource::ResourceDetector;
use opentelemetry::sdk::resource::SdkProvidedResourceDetector;
use opentelemetry::sdk::trace::Config;
use opentelemetry::sdk::Resource;
use opentelemetry::{
    global, sdk,
    sdk::export::{trace, ExportError},
    sdk::trace::TraceRuntime,
    trace::{TraceError, TracerProvider},
    KeyValue,
};
use opentelemetry_http::HttpClient;
use opentelemetry_semantic_conventions as semcov;
use std::borrow::Cow;
#[cfg(all(
    not(feature = "reqwest-client"),
    not(feature = "reqwest-blocking-client"),
    feature = "surf-client"
))]
use std::convert::TryFrom;
use std::net::SocketAddr;
use std::time::Duration;

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
pub fn new_pipeline() -> ZipkinPipelineBuilder {
    ZipkinPipelineBuilder::default()
}

/// Builder for `ExporterConfig` struct.
#[derive(Debug)]
pub struct ZipkinPipelineBuilder {
    service_name: Option<String>,
    service_addr: Option<SocketAddr>,
    collector_endpoint: String,
    trace_config: Option<sdk::trace::Config>,
    client: Option<Box<dyn HttpClient>>,
}

impl Default for ZipkinPipelineBuilder {
    fn default() -> Self {
        let timeout = env::get_timeout();
        ZipkinPipelineBuilder {
            #[cfg(feature = "reqwest-blocking-client")]
            client: Some(Box::new(
                reqwest::blocking::Client::builder()
                    .timeout(timeout)
                    .build()
                    .unwrap_or_else(|_| reqwest::blocking::Client::new()),
            )),
            #[cfg(all(
                not(feature = "reqwest-blocking-client"),
                not(feature = "surf-client"),
                feature = "reqwest-client"
            ))]
            client: Some(Box::new(
                reqwest::Client::builder()
                    .timeout(timeout)
                    .build()
                    .unwrap_or_else(|_| reqwest::Client::new()),
            )),
            #[cfg(all(
                not(feature = "reqwest-client"),
                not(feature = "reqwest-blocking-client"),
                feature = "surf-client"
            ))]
            client: Some(Box::new(
                surf::Client::try_from(surf::Config::new().set_timeout(Some(timeout)))
                    .unwrap_or_else(|_| surf::Client::new()),
            )),
            #[cfg(all(
                not(feature = "reqwest-client"),
                not(feature = "surf-client"),
                not(feature = "reqwest-blocking-client")
            ))]
            client: None,

            service_name: None,
            service_addr: None,
            collector_endpoint: env::get_endpoint(),
            trace_config: None,
        }
    }
}

impl ZipkinPipelineBuilder {
    /// Initial a Zipkin span exporter.
    ///
    /// Returns error if the endpoint is not valid or if no http client is provided.
    pub fn init_exporter(mut self) -> Result<Exporter, TraceError> {
        let (_, endpoint) = self.init_config_and_endpoint();
        self.init_exporter_with_endpoint(endpoint)
    }

    fn init_config_and_endpoint(&mut self) -> (Config, Endpoint) {
        let service_name = self.service_name.take();
        if let Some(service_name) = service_name {
            let config = if let Some(mut cfg) = self.trace_config.take() {
                cfg.resource = Cow::Owned(Resource::new(
                    cfg.resource
                        .iter()
                        .filter(|(k, _v)| **k != semcov::resource::SERVICE_NAME)
                        .map(|(k, v)| KeyValue::new(k.clone(), v.clone()))
                        .collect::<Vec<KeyValue>>(),
                ));
                cfg
            } else {
                Config {
                    resource: Cow::Owned(Resource::empty()),
                    ..Default::default()
                }
            };
            (config, Endpoint::new(service_name, self.service_addr))
        } else {
            let service_name = SdkProvidedResourceDetector
                .detect(Duration::from_secs(0))
                .get(semcov::resource::SERVICE_NAME)
                .unwrap()
                .to_string();
            (
                Config {
                    // use a empty resource to prevent TracerProvider to assign a service name.
                    resource: Cow::Owned(Resource::empty()),
                    ..Default::default()
                },
                Endpoint::new(service_name, self.service_addr),
            )
        }
    }

    fn init_exporter_with_endpoint(self, endpoint: Endpoint) -> Result<Exporter, TraceError> {
        if let Some(client) = self.client {
            let exporter = Exporter::new(
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

    /// Install the Zipkin trace exporter pipeline with a simple span processor.
    pub fn install_simple(mut self) -> Result<sdk::trace::Tracer, TraceError> {
        let (config, endpoint) = self.init_config_and_endpoint();
        let exporter = self.init_exporter_with_endpoint(endpoint)?;
        let mut provider_builder =
            sdk::trace::TracerProvider::builder().with_simple_exporter(exporter);
        provider_builder = provider_builder.with_config(config);
        let provider = provider_builder.build();
        let tracer = provider.versioned_tracer(
            "opentelemetry-zipkin",
            Some(env!("CARGO_PKG_VERSION")),
            None,
        );
        let _ = global::set_tracer_provider(provider);
        Ok(tracer)
    }

    /// Install the Zipkin trace exporter pipeline with a batch span processor using the specified
    /// runtime.
    pub fn install_batch<R: TraceRuntime>(
        mut self,
        runtime: R,
    ) -> Result<sdk::trace::Tracer, TraceError> {
        let (config, endpoint) = self.init_config_and_endpoint();
        let exporter = self.init_exporter_with_endpoint(endpoint)?;
        let mut provider_builder =
            sdk::trace::TracerProvider::builder().with_batch_exporter(exporter, runtime);
        provider_builder = provider_builder.with_config(config);
        let provider = provider_builder.build();
        let tracer = provider.versioned_tracer(
            "opentelemetry-zipkin",
            Some(env!("CARGO_PKG_VERSION")),
            None,
        );
        let _ = global::set_tracer_provider(provider);
        Ok(tracer)
    }

    /// Assign the service name under which to group traces.
    pub fn with_service_name<T: Into<String>>(mut self, name: T) -> Self {
        self.service_name = Some(name.into());
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
