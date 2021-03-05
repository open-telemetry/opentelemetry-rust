mod intern;
mod model;

pub use model::ApiVersion;
pub use model::Error;

use async_trait::async_trait;
use http::{Method, Request, Uri};
use itertools::Itertools;
use opentelemetry::sdk::export::trace;
use opentelemetry::sdk::export::trace::SpanData;
use opentelemetry::trace::TraceError;
use opentelemetry::{global, sdk, trace::TracerProvider};
use opentelemetry_http::HttpClient;

/// Default Datadog collector endpoint
const DEFAULT_AGENT_ENDPOINT: &str = "http://127.0.0.1:8126";

/// Default service name if no service is configured.
const DEFAULT_SERVICE_NAME: &str = "OpenTelemetry";

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
pub fn new_pipeline() -> DatadogPipelineBuilder<()> {
    DatadogPipelineBuilder::default()
}

/// Builder for `ExporterConfig` struct.
#[derive(Debug)]
pub struct DatadogPipelineBuilder<R: opentelemetry::runtime::Runtime> {
    service_name: String,
    agent_endpoint: String,
    trace_config: Option<sdk::trace::Config>,
    version: ApiVersion,
    client: Option<Box<dyn HttpClient>>,
    runtime: R,
}

impl Default for DatadogPipelineBuilder<()> {
    fn default() -> Self {
        DatadogPipelineBuilder {
            service_name: DEFAULT_SERVICE_NAME.to_string(),
            agent_endpoint: DEFAULT_AGENT_ENDPOINT.to_string(),
            trace_config: None,
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
            runtime: (),
        }
    }
}

impl<R: opentelemetry::runtime::Runtime> DatadogPipelineBuilder<R> {
    /// Create `ExporterConfig` struct from current `ExporterConfigBuilder`
    pub fn install(mut self) -> Result<sdk::trace::Tracer, TraceError> {
        if let Some(client) = self.client {
            let endpoint = self.agent_endpoint + self.version.path();
            let exporter = DatadogExporter::new(
                self.service_name.clone(),
                endpoint.parse().map_err::<Error, _>(Into::into)?,
                self.version,
                client,
            );
            let mut provider_builder =
                sdk::trace::TracerProvider::builder().with_exporter(exporter, self.runtime);
            if let Some(config) = self.trace_config.take() {
                provider_builder = provider_builder.with_config(config);
            }
            let provider = provider_builder.build();
            let tracer =
                provider.get_tracer("opentelemetry-datadog", Some(env!("CARGO_PKG_VERSION")));
            let _ = global::set_tracer_provider(provider);
            Ok(tracer)
        } else {
            Err(Error::NoHttpClient.into())
        }
    }

    /// Assign the service name under which to group traces
    pub fn with_service_name<T: Into<String>>(mut self, name: T) -> Self {
        self.service_name = name.into();
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

    /// Assign the runtime to use.
    ///
    /// Please make sure the selected HTTP client works with the runtime.
    pub fn with_runtime<NewR: opentelemetry::runtime::Runtime>(
        self,
        runtime: NewR,
    ) -> DatadogPipelineBuilder<NewR> {
        DatadogPipelineBuilder {
            service_name: self.service_name,
            agent_endpoint: self.agent_endpoint,
            trace_config: self.trace_config,
            version: self.version,
            client: self.client,
            runtime,
        }
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
        let data = self.version.encode(&self.service_name, traces)?;
        let req = Request::builder()
            .method(Method::POST)
            .uri(self.request_url.clone())
            .header(http::header::CONTENT_TYPE, self.version.content_type())
            .header(DATADOG_TRACE_COUNT_HEADER, trace_count)
            .body(data)
            .map_err::<Error, _>(Into::into)?;
        self.client.send(req).await
    }
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
        traces.sort_by_key(|t| t[0].span_context.trace_id().to_u128());

        assert_eq!(traces, expected);
    }
}
