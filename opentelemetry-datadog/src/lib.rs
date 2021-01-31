//! # OpenTelemetry Datadog Exporter
//!
//! An OpenTelemetry exporter implementation
//!
//! See the [Datadog Docs](https://docs.datadoghq.com/agent/) for information on how to run the datadog-agent
//!
//! ## Quirks
//!
//! There are currently some incompatibilities between Datadog and OpenTelemetry, and this manifests
//! as minor quirks to this exporter.
//!
//! Firstly Datadog uses operation_name to describe what OpenTracing would call a component.
//! Or to put it another way, in OpenTracing the operation / span name's are relatively
//! granular and might be used to identify a specific endpoint. In datadog, however, they
//! are less granular - it is expected in Datadog that a service will have single
//! primary span name that is the root of all traces within that service, with an additional piece of
//! metadata called resource_name providing granularity - https://docs.datadoghq.com/tracing/guide/configuring-primary-operation/
//!
//! The Datadog Golang API takes the approach of using a `resource.name` OpenTelemetry attribute to set the
//! resource_name - https://github.com/DataDog/dd-trace-go/blob/ecb0b805ef25b00888a2fb62d465a5aa95e7301e/ddtrace/opentracer/tracer.go#L10
//!
//! Unfortunately, this breaks compatibility with other OpenTelemetry exporters which expect
//! a more granular operation name - as per the OpenTracing specification.
//!
//! This exporter therefore takes a different approach of naming the span with the name of the
//! tracing provider, and using the span name to set the resource_name. This should in most cases
//! lead to the behaviour that users expect.
//!
//! Datadog additionally has a span_type string that alters the rendering of the spans in the web UI.
//! This can be set as the `span.type` OpenTelemetry span attribute.
//!
//! For standard values see here - https://github.com/DataDog/dd-trace-go/blob/ecb0b805ef25b00888a2fb62d465a5aa95e7301e/ddtrace/ext/app_types.go#L31
//!
//! ## Performance
//!
//! For optimal performance, a batch exporter is recommended as the simple
//! exporter will export each span synchronously on drop. You can enable the
//! [`tokio-support`] or [`async-std`] features to have a batch exporter configured for
//! you automatically for either executor when you install the pipeline.
//!
//! ```toml
//! [dependencies]
//! opentelemetry = { version = "*", features = ["tokio-support"] }
//! opentelemetry-datadog = "*"
//! ```
//!
//! [`tokio-support`]: https://tokio.rs
//! [`async-std`]: https://async.rs
//!

//! ## Bring your own http client
//!
//! Users can choose appropriate http clients to align with their runtime.
//!
//! Based on the feature enabled. The default http client will be different. If user doesn't specific
//! features or enabled `reqwest-blocking-client` feature. The blocking reqwest http client will be used as
//! default client. If `reqwest-client` feature is enabled. The async reqwest http client will be used. If
//! `surf-client` feature is enabled. The surf http client will be used.
//!
//! Note that async http clients may need specific runtime otherwise it will panic. User should make
//! sure the http client is running in appropriate runime.
//!
//! Users can always use their own http clients by implementing `HttpClient` trait.
//!
//! ## Kitchen Sink Full Configuration
//!
//! Example showing how to override all configuration options. See the
//! [`DatadogPipelineBuilder`] docs for details of each option.
//!
//! [`DatadogPipelineBuilder`]: struct.DatadogPipelineBuilder.html
//!
//! ```no_run
//! use opentelemetry::{KeyValue, trace::Tracer};
//! use opentelemetry::sdk::{trace::{self, IdGenerator, Sampler}, Resource};
//! use opentelemetry::sdk::export::trace::ExportResult;
//! use opentelemetry_datadog::{new_pipeline, ApiVersion, Error};
//! use opentelemetry_http::HttpClient;
//! use async_trait::async_trait;
//!
//! // `reqwest` and `surf` are supported through features, if you prefer an
//! // alternate http client you can add support by implementing `HttpClient` as
//! // shown here.
//! #[derive(Debug)]
//! struct IsahcClient(isahc::HttpClient);
//!
//! #[async_trait]
//! impl HttpClient for IsahcClient {
//!   async fn send(&self, request: http::Request<Vec<u8>>) -> ExportResult {
//!     let result = self.0.send_async(request).await.map_err(|err| Error::Other(err.to_string()))?;
//!
//!     if result.status().is_success() {
//!       Ok(())
//!     } else {
//!       Err(Error::Other(result.status().to_string()).into())
//!     }
//!   }
//! }
//!
//! fn main() -> Result<(), opentelemetry::trace::TraceError> {
//!     let (tracer, _uninstall) = new_pipeline()
//!         .with_service_name("my_app")
//!         .with_version(ApiVersion::Version05)
//!         .with_agent_endpoint("http://localhost:8126")
//!         .with_trace_config(
//!             trace::config()
//!                 .with_default_sampler(Sampler::AlwaysOn)
//!                 .with_id_generator(IdGenerator::default())
//!         )
//!         .install()?;
//!
//!     tracer.in_span("doing_work", |cx| {
//!         // Traced app logic here...
//!     });
//!
//!     Ok(())
//! }
//! ```

mod exporter;
mod propagator;

pub use exporter::{new_pipeline, ApiVersion, DatadogExporter, DatadogPipelineBuilder, Error};
pub use propagator::DatadogPropagator;
