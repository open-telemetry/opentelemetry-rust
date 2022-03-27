mod intern;
mod model;

use std::fmt::{Debug, Formatter};
pub use model::ApiVersion;
pub use model::Error;

use async_trait::async_trait;
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
use crate::exporter::model::ExtractStrTagsFn;

/// Default Datadog collector endpoint
const DEFAULT_AGENT_ENDPOINT: &str = "http://127.0.0.1:8126";

/// Header name used to inform the Datadog agent of the number of traces in the payload
const DATADOG_TRACE_COUNT_HEADER: &str = "X-Datadog-Trace-Count";

/// Datadog span exporter
#[derive(Debug)]
pub struct DatadogExporter {
    client: Box<dyn HttpClient>,
    request_url: Uri,
    service_name: String,
    version: ApiVersion,
}

impl DatadogExporter {
    fn new(
        service_name: String,
        request_url: Uri,
        version: ApiVersion,
        client: Box<dyn HttpClient>,
    ) -> Self {
        DatadogExporter {
            client,
            request_url,
            service_name,
            version,
        }
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
    client: Option<Box<dyn HttpClient>>,

    resource_mapping: Option<ExtractStrTagsFn>,
    name_mapping: Option<ExtractStrTagsFn>,
    service_name_mapping: Option<ExtractStrTagsFn>,
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
            client: Some(Box::new(surf::Client::new())),
            #[cfg(all(
            not(feature = "surf-client"),
            not(feature = "reqwest-blocking-client"),
            feature = "reqwest-client"
            ))]
            client: Some(Box::new(reqwest::Client::new())),
            #[cfg(feature = "reqwest-blocking-client")]
            client: Some(Box::new(reqwest::blocking::Client::new())),
        }
    }
}

impl Debug for DatadogPipelineBuilder{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
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
                cfg.resource = cfg.resource.map(|r| {
                    let without_service_name = r
                        .iter()
                        .filter(|(k, _v)| **k != semcov::resource::SERVICE_NAME)
                        .map(|(k, v)| KeyValue::new(k.clone(), v.clone()))
                        .collect::<Vec<KeyValue>>();
                    Arc::new(Resource::new(without_service_name))
                });
                cfg
            } else {
                Config {
                    resource: Some(Arc::new(Resource::empty())),
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
                    resource: Some(Arc::new(Resource::empty())),
                    ..Default::default()
                },
                service_name,
            )
        }
    }

    fn build_exporter_with_service_name(
        self,
        service_name: String,
    ) -> Result<DatadogExporter, TraceError> {
        if let Some(client) = self.client {
            let endpoint = self.agent_endpoint + self.version.path();
            let exporter = DatadogExporter::new(
                service_name,
                endpoint.parse().map_err::<Error, _>(Into::into)?,
                self.version,
                client,
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

    /// Assign the Datadog collector endpoint
    pub fn with_agent_endpoint<T: Into<String>>(mut self, endpoint: T) -> Self {
        self.agent_endpoint = endpoint.into();
        self
    }

    /// Choose the http client used by uploader
    pub fn with_http_client<T: HttpClient + 'static>(
        mut self,
        client: Box<dyn HttpClient>,
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

    pub fn with_resource_mapping<F>(mut self, f: F) -> Self
        where F: for<'a> Fn(&'a SpanData, &'a ModelConfig) -> &'a str + 'static {
        self.resource_mapping = Some(Arc::new(f));
        self
    }

    pub fn with_name_mapping<F>(mut self, f: F) -> Self
        where F: for<'a> Fn(&'a SpanData, &'a ModelConfig) -> &'a str + 'static {
        self.name_mapping = Some(Arc::new(f));
        self
    }

    pub fn with_service_name_mapping<F>(mut self, f: F) -> Self
        where F: for<'a> Fn(&'a SpanData, &'a ModelConfig) -> &'a str + 'static {
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

#[async_trait]
impl trace::SpanExporter for DatadogExporter {
    /// Export spans to datadog-agent
    async fn export(&mut self, batch: Vec<SpanData>) -> trace::ExportResult {
        let traces: Vec<Vec<SpanData>> = group_into_traces(batch);
        let trace_count = traces.len();
        let model_config = ModelConfig {
            service_name: self.service_name.clone(),
            _private: (),
        };
        let data = self.version.encode(&model_config, traces, None, None, None)?;
        let req = Request::builder()
            .method(Method::POST)
            .uri(self.request_url.clone())
            .header(http::header::CONTENT_TYPE, self.version.content_type())
            .header(DATADOG_TRACE_COUNT_HEADER, trace_count)
            .body(data)
            .map_err::<Error, _>(Into::into)?;
        let _ = self.client.send(req).await?.error_for_status()?;
        Ok(())
    }
}

/// Helper struct to custom the mapping between Opentelemetry spans and datadog spans
#[derive(Default)]
pub struct ModelConfig {
    pub service_name: String,
    _private: (),
}


#[cfg(test)]
mod tests {
    use super::*;

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
}
