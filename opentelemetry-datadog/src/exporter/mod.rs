mod intern;
mod model;

pub use model::ApiVersion;
pub use model::Error;
pub use model::FieldMappingFn;

use std::borrow::Cow;
use std::fmt::{Debug, Formatter};

use crate::exporter::model::FieldMapping;
use futures::future::BoxFuture;
use http::{Method, Request, Uri};
use itertools::Itertools;
use opentelemetry::sdk::export::trace;
use opentelemetry::sdk::export::trace::SpanData;
use opentelemetry::sdk::resource::ResourceDetector;
use opentelemetry::sdk::resource::SdkProvidedResourceDetector;
use opentelemetry::sdk::trace::{Config, TraceRuntime};
use opentelemetry::sdk::Resource;
use opentelemetry::trace::TraceError;
use opentelemetry::{global, sdk, trace::TracerProvider, KeyValue};
use opentelemetry_http::{HttpClient, ResponseExt};
use opentelemetry_semantic_conventions as semcov;
use std::sync::Arc;
use std::time::Duration;
use url::Url;

/// Default Datadog collector endpoint
const DEFAULT_AGENT_ENDPOINT: &str = "http://127.0.0.1:8126";

/// Header name used to inform the Datadog agent of the number of traces in the payload
const DATADOG_TRACE_COUNT_HEADER: &str = "X-Datadog-Trace-Count";

/// Datadog span exporter
pub struct DatadogExporter {
    client: Arc<dyn HttpClient>,
    request_url: Uri,
    model_config: ModelConfig,
    version: ApiVersion,

    resource_mapping: Option<FieldMapping>,
    name_mapping: Option<FieldMapping>,
    service_name_mapping: Option<FieldMapping>,
}

impl DatadogExporter {
    fn new(
        model_config: ModelConfig,
        request_url: Uri,
        version: ApiVersion,
        client: Arc<dyn HttpClient>,
        resource_mapping: Option<FieldMapping>,
        name_mapping: Option<FieldMapping>,
        service_name_mapping: Option<FieldMapping>,
    ) -> Self {
        DatadogExporter {
            client,
            request_url,
            model_config,
            version,
            resource_mapping,
            name_mapping,
            service_name_mapping,
        }
    }

    fn build_request(&self, batch: Vec<SpanData>) -> Result<http::Request<Vec<u8>>, TraceError> {
        let traces: Vec<Vec<SpanData>> = group_into_traces(batch);
        let trace_count = traces.len();
        let data = self.version.encode(
            &self.model_config,
            traces,
            self.service_name_mapping.clone(),
            self.name_mapping.clone(),
            self.resource_mapping.clone(),
        )?;
        let req = Request::builder()
            .method(Method::POST)
            .uri(self.request_url.clone())
            .header(http::header::CONTENT_TYPE, self.version.content_type())
            .header(DATADOG_TRACE_COUNT_HEADER, trace_count)
            .body(data)
            .map_err::<Error, _>(Into::into)?;

        Ok(req)
    }
}

impl Debug for DatadogExporter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DatadogExporter")
            .field("model_config", &self.model_config)
            .field("request_url", &self.request_url)
            .field("version", &self.version)
            .field("client", &self.client)
            .field("resource_mapping", &mapping_debug(&self.resource_mapping))
            .field("name_mapping", &mapping_debug(&self.name_mapping))
            .field(
                "service_name_mapping",
                &mapping_debug(&self.service_name_mapping),
            )
            .finish()
    }
}

/// Create a new Datadog exporter pipeline builder.
pub fn new_pipeline() -> DatadogPipelineBuilder {
    DatadogPipelineBuilder::default()
}

/// Builder for `ExporterConfig` struct.
pub struct DatadogPipelineBuilder {
    service_name: Option<String>,
    agent_endpoint: String,
    trace_config: Option<sdk::trace::Config>,
    version: ApiVersion,
    client: Option<Arc<dyn HttpClient>>,
    resource_mapping: Option<FieldMapping>,
    name_mapping: Option<FieldMapping>,
    service_name_mapping: Option<FieldMapping>,
}

impl Default for DatadogPipelineBuilder {
    fn default() -> Self {
        DatadogPipelineBuilder {
            service_name: None,
            agent_endpoint: DEFAULT_AGENT_ENDPOINT.to_string(),
            trace_config: None,
            resource_mapping: None,
            name_mapping: None,
            service_name_mapping: None,
            version: ApiVersion::Version05,
            #[cfg(all(
                not(feature = "reqwest-client"),
                not(feature = "reqwest-blocking-client"),
                not(feature = "surf-client"),
            ))]
            client: None,
            #[cfg(all(
                not(feature = "reqwest-client"),
                not(feature = "reqwest-blocking-client"),
                feature = "surf-client"
            ))]
            client: Some(Arc::new(surf::Client::new())),
            #[cfg(all(
                not(feature = "surf-client"),
                not(feature = "reqwest-blocking-client"),
                feature = "reqwest-client"
            ))]
            client: Some(Arc::new(reqwest::Client::new())),
            #[cfg(feature = "reqwest-blocking-client")]
            client: Some(Arc::new(reqwest::blocking::Client::new())),
        }
    }
}

impl Debug for DatadogPipelineBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DatadogExporter")
            .field("service_name", &self.service_name)
            .field("agent_endpoint", &self.agent_endpoint)
            .field("version", &self.version)
            .field("trace_config", &self.trace_config)
            .field("client", &self.client)
            .field("resource_mapping", &mapping_debug(&self.resource_mapping))
            .field("name_mapping", &mapping_debug(&self.name_mapping))
            .field(
                "service_name_mapping",
                &mapping_debug(&self.service_name_mapping),
            )
            .finish()
    }
}

impl DatadogPipelineBuilder {
    /// Building a new exporter.
    ///
    /// This is useful if you are manually constructing a pipeline.
    pub fn build_exporter(mut self) -> Result<DatadogExporter, TraceError> {
        let (_, service_name) = self.build_config_and_service_name();
        self.build_exporter_with_service_name(service_name)
    }

    fn build_config_and_service_name(&mut self) -> (Config, String) {
        let service_name = self.service_name.take();
        if let Some(service_name) = service_name {
            let config = if let Some(mut cfg) = self.trace_config.take() {
                cfg.resource = Cow::Owned(Resource::new(
                    cfg.resource
                        .iter()
                        .filter(|(k, _v)| **k != semcov::resource::SERVICE_NAME)
                        .map(|(k, v)| KeyValue::new(k.clone(), v.clone())),
                ));
                cfg
            } else {
                Config {
                    resource: Cow::Owned(Resource::empty()),
                    ..Default::default()
                }
            };
            (config, service_name)
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
                service_name,
            )
        }
    }

    // parse the endpoint and append the path based on versions.
    // keep the query and host the same.
    fn build_endpoint(agent_endpoint: &str, version: &str) -> Result<Uri, TraceError> {
        // build agent endpoint based on version
        let mut endpoint = agent_endpoint
            .parse::<Url>()
            .map_err::<Error, _>(Into::into)?;
        let mut paths = endpoint
            .path_segments()
            .map(|c| c.filter(|s| !s.is_empty()).collect::<Vec<_>>())
            .unwrap_or_default();
        paths.push(version);

        let path_str = paths.join("/");
        endpoint.set_path(path_str.as_str());

        Ok(endpoint.as_str().parse().map_err::<Error, _>(Into::into)?)
    }

    fn build_exporter_with_service_name(
        self,
        service_name: String,
    ) -> Result<DatadogExporter, TraceError> {
        if let Some(client) = self.client {
            let model_config = ModelConfig {
                service_name,
                ..Default::default()
            };

            let exporter = DatadogExporter::new(
                model_config,
                Self::build_endpoint(&self.agent_endpoint, self.version.path())?,
                self.version,
                client,
                self.resource_mapping,
                self.name_mapping,
                self.service_name_mapping,
            );
            Ok(exporter)
        } else {
            Err(Error::NoHttpClient.into())
        }
    }

    /// Install the Datadog trace exporter pipeline using a simple span processor.
    pub fn install_simple(mut self) -> Result<sdk::trace::Tracer, TraceError> {
        let (config, service_name) = self.build_config_and_service_name();
        let exporter = self.build_exporter_with_service_name(service_name)?;
        let mut provider_builder =
            sdk::trace::TracerProvider::builder().with_simple_exporter(exporter);
        provider_builder = provider_builder.with_config(config);
        let provider = provider_builder.build();
        let tracer = provider.versioned_tracer(
            "opentelemetry-datadog",
            Some(env!("CARGO_PKG_VERSION")),
            None,
        );
        let _ = global::set_tracer_provider(provider);
        Ok(tracer)
    }

    /// Install the Datadog trace exporter pipeline using a batch span processor with the specified
    /// runtime.
    pub fn install_batch<R: TraceRuntime>(
        mut self,
        runtime: R,
    ) -> Result<sdk::trace::Tracer, TraceError> {
        let (config, service_name) = self.build_config_and_service_name();
        let exporter = self.build_exporter_with_service_name(service_name)?;
        let mut provider_builder =
            sdk::trace::TracerProvider::builder().with_batch_exporter(exporter, runtime);
        provider_builder = provider_builder.with_config(config);
        let provider = provider_builder.build();
        let tracer = provider.versioned_tracer(
            "opentelemetry-datadog",
            Some(env!("CARGO_PKG_VERSION")),
            None,
        );
        let _ = global::set_tracer_provider(provider);
        Ok(tracer)
    }

    /// Assign the service name under which to group traces
    pub fn with_service_name<T: Into<String>>(mut self, name: T) -> Self {
        self.service_name = Some(name.into());
        self
    }

    /// Assign the Datadog collector endpoint.
    ///
    /// The endpoint of the datadog agent, by default it is `http://127.0.0.1:8126`.
    pub fn with_agent_endpoint<T: Into<String>>(mut self, endpoint: T) -> Self {
        self.agent_endpoint = endpoint.into();
        self
    }

    /// Choose the http client used by uploader
    pub fn with_http_client<T: HttpClient + 'static>(
        mut self,
        client: Arc<dyn HttpClient>,
    ) -> Self {
        self.client = Some(client);
        self
    }

    /// Assign the SDK trace configuration
    pub fn with_trace_config(mut self, config: sdk::trace::Config) -> Self {
        self.trace_config = Some(config);
        self
    }

    /// Set version of Datadog trace ingestion API
    pub fn with_version(mut self, version: ApiVersion) -> Self {
        self.version = version;
        self
    }

    /// Custom the value used for `resource` field in datadog spans.
    /// See [`FieldMappingFn`] for details.
    pub fn with_resource_mapping<F>(mut self, f: F) -> Self
    where
        F: for<'a> Fn(&'a SpanData, &'a ModelConfig) -> &'a str + Send + Sync + 'static,
    {
        self.resource_mapping = Some(Arc::new(f));
        self
    }

    /// Custom the value used for `name` field in datadog spans.
    /// See [`FieldMappingFn`] for details.
    pub fn with_name_mapping<F>(mut self, f: F) -> Self
    where
        F: for<'a> Fn(&'a SpanData, &'a ModelConfig) -> &'a str + Send + Sync + 'static,
    {
        self.name_mapping = Some(Arc::new(f));
        self
    }

    /// Custom the value used for `service_name` field in datadog spans.
    /// See [`FieldMappingFn`] for details.
    pub fn with_service_name_mapping<F>(mut self, f: F) -> Self
    where
        F: for<'a> Fn(&'a SpanData, &'a ModelConfig) -> &'a str + Send + Sync + 'static,
    {
        self.service_name_mapping = Some(Arc::new(f));
        self
    }
}

fn group_into_traces(spans: Vec<SpanData>) -> Vec<Vec<SpanData>> {
    spans
        .into_iter()
        .into_group_map_by(|span_data| span_data.span_context.trace_id())
        .into_iter()
        .map(|(_, trace)| trace)
        .collect()
}

async fn send_request(
    client: Arc<dyn HttpClient>,
    request: http::Request<Vec<u8>>,
) -> trace::ExportResult {
    let _ = client.send(request).await?.error_for_status()?;
    Ok(())
}

impl trace::SpanExporter for DatadogExporter {
    /// Export spans to datadog-agent
    fn export(&mut self, batch: Vec<SpanData>) -> BoxFuture<'static, trace::ExportResult> {
        let request = match self.build_request(batch) {
            Ok(req) => req,
            Err(err) => return Box::pin(std::future::ready(Err(err))),
        };

        let client = self.client.clone();
        Box::pin(send_request(client, request))
    }
}

/// Helper struct to custom the mapping between Opentelemetry spans and datadog spans.
///
/// This struct will be passed to [`FieldMappingFn`]
#[derive(Default, Debug)]
#[non_exhaustive]
pub struct ModelConfig {
    pub service_name: String,
}

fn mapping_debug(f: &Option<FieldMapping>) -> String {
    if f.is_some() {
        "custom mapping"
    } else {
        "default mapping"
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ApiVersion::Version05;

    use crate::exporter::model::tests::get_span;

    #[test]
    fn test_out_of_order_group() {
        let batch = vec![get_span(1, 1, 1), get_span(2, 2, 2), get_span(1, 1, 3)];
        let expected = vec![
            vec![get_span(1, 1, 1), get_span(1, 1, 3)],
            vec![get_span(2, 2, 2)],
        ];

        let mut traces = group_into_traces(batch);
        // We need to sort the output in order to compare, but this is not required by the Datadog agent
        traces.sort_by_key(|t| u128::from_be_bytes(t[0].span_context.trace_id().to_bytes()));

        assert_eq!(traces, expected);
    }

    #[test]
    fn test_agent_endpoint_with_version() {
        let with_tail_slash =
            DatadogPipelineBuilder::build_endpoint("http://localhost:8126/", Version05.path());
        let without_tail_slash =
            DatadogPipelineBuilder::build_endpoint("http://localhost:8126", Version05.path());
        let with_query = DatadogPipelineBuilder::build_endpoint(
            "http://localhost:8126?api_key=123",
            Version05.path(),
        );
        let invalid = DatadogPipelineBuilder::build_endpoint(
            "http://localhost:klsajfjksfh",
            Version05.path(),
        );

        assert_eq!(
            with_tail_slash.unwrap().to_string(),
            "http://localhost:8126/v0.5/traces"
        );
        assert_eq!(
            without_tail_slash.unwrap().to_string(),
            "http://localhost:8126/v0.5/traces"
        );
        assert_eq!(
            with_query.unwrap().to_string(),
            "http://localhost:8126/v0.5/traces?api_key=123"
        );
        assert!(invalid.is_err())
    }
}
