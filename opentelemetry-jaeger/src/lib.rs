//! Collects OpenTelemetry spans and reports them to a given Jaeger
//! `agent` or `collector` endpoint, propagate the tracing context between the applications using [Jaeger propagation format].
//!
//! *Warning*: Note that the exporter component from this crate will be [deprecated][jaeger-deprecation]
//! in the future. Users are advised to move to [opentelemetry_otlp][otlp-exporter] instead as [Jaeger][jaeger-otlp]
//! supports accepting data in the OTLP protocol.
//! See the [Jaeger Docs] for details about Jaeger and deployment information.
//!
//! *Compiler support: [requires `rustc` 1.64+][msrv]*
//!
//! [Jaeger Docs]: https://www.jaegertracing.io/docs/
//! [jaeger-deprecation]: https://github.com/open-telemetry/opentelemetry-specification/pull/2858/files
//! [jaeger-otlp]: https://www.jaegertracing.io/docs/1.38/apis/#opentelemetry-protocol-stable
//! [otlp-exporter]: https://docs.rs/opentelemetry-otlp/latest/opentelemetry_otlp/
//! [msrv]: #supported-rust-versions
//! [jaeger propagation format]: https://www.jaegertracing.io/docs/1.18/client-libraries/#propagation-format
//!
//! ## Quickstart
//!
//! First make sure you have a running version of the Jaeger instance
//! you want to send data to:
//!
//! ```shell
//! $ docker run -d -p6831:6831/udp -p6832:6832/udp -p16686:16686 -p14268:14268 jaegertracing/all-in-one:latest
//! ```
//!
//! Then install a new jaeger pipeline with the recommended defaults to start
//! exporting telemetry:
//!
//! ```no_run
//! use opentelemetry::{global, trace::{Tracer, TraceError}};
//! use opentelemetry_jaeger_propagator;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), TraceError> {
//!     global::set_text_map_propagator(opentelemetry_jaeger_propagator::Propagator::new());
//!     let tracer = opentelemetry_jaeger::new_agent_pipeline().install_simple()?;
//!
//!     tracer.in_span("doing_work", |cx| {
//!         // Traced app logic here...
//!     });
//!
//!     global::shutdown_tracer_provider(); // export remaining spans
//!
//!     Ok(())
//! }
//! ```
//!
//! Or if you are running on an async runtime like Tokio and want to report spans in batches
//! ```no_run
//! use opentelemetry::{global, trace::{Tracer, TraceError}};
//! use opentelemetry_sdk::runtime::Tokio;
//! use opentelemetry_jaeger_propagator;
//!
//! fn main() -> Result<(), TraceError> {
//!     global::set_text_map_propagator(opentelemetry_jaeger_propagator::Propagator::new());
//!     let tracer = opentelemetry_jaeger::new_agent_pipeline().install_batch(Tokio)?;
//!
//!     tracer.in_span("doing_work", |cx| {
//!         // Traced app logic here...
//!     });
//!
//!     global::shutdown_tracer_provider(); // export remaining spans
//!
//!     Ok(())
//! }
//! ```
//! ## Performance
//!
//! For optimal performance, a batch exporter is recommended as the simple exporter
//! will export each span synchronously on drop. You can enable the `rt-tokio`,
//! `rt-tokio-current-thread` or `rt-async-std` features and specify a runtime
//! on the pipeline builder to have a batch exporter configured for you
//! automatically.
//!
//! ```toml
//! [dependencies]
//! opentelemetry_sdk = { version = "*", features = ["rt-tokio"] }
//! opentelemetry-jaeger = { version = "*", features = ["rt-tokio"] }
//! ```
//!
//! ```no_run
//! # fn main() -> Result<(), opentelemetry::trace::TraceError> {
//! let tracer = opentelemetry_jaeger::new_agent_pipeline()
//!     .install_batch(opentelemetry_sdk::runtime::Tokio)?;
//! # Ok(())
//! # }
//! ```
//!
//! [`tokio`]: https://tokio.rs
//! [`async-std`]: https://async.rs
//!
//! ## Jaeger Exporter From Environment Variables
//!
//! The jaeger pipeline builder can be configured dynamically via environment
//! variables. All variables are optional, a full list of accepted options can
//! be found in the [jaeger variables spec].
//!
//! [jaeger variables spec]: https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/configuration/sdk-environment-variables.md
//!
//! ## Jaeger Collector Example
//!
//! If you want to skip the agent and submit spans directly to a Jaeger collector,
//! you can enable the optional `collector_client` feature for this crate. This
//! example expects a Jaeger collector running on `http://localhost:14268`.
//!
//! ```toml
//! [dependencies]
//! opentelemetry-jaeger = { version = "..", features = ["collector_client", "isahc_collector_client"] }
//! ```
//!
//! Then you can use the [`with_endpoint`] method to specify the endpoint:
//!
//! [`with_endpoint`]: exporter::config::collector::CollectorPipeline::with_endpoint
//!
//! ```ignore
//! // Note that this requires the `collector_client` feature.
//! // We enabled the `isahc_collector_client` feature for a default isahc http client.
//! // You can also provide your own implementation via .with_http_client() method.
//! use opentelemetry::trace::{Tracer, TraceError};
//!
//! fn main() -> Result<(), TraceError> {
//!     let tracer = opentelemetry_jaeger::new_collector_pipeline()
//!         .with_endpoint("http://localhost:14268/api/traces")
//!         // optionally set username and password for authentication of the exporter.
//!         .with_username("username")
//!         .with_password("s3cr3t")
//!         .with_isahc()
//!         //.with_http_client(<your client>) provide custom http client implementation
//!         .install_batch(opentelemetry_sdk::runtime::Tokio)?;
//!
//!     tracer.in_span("doing_work", |cx| {
//!         // Traced app logic here...
//!     });
//!
//!     Ok(())
//! }
//! ```
//! ## Resource, tags and service name
//! In order to export the spans in different format. opentelemetry uses its own
//! model internally. Most of the jaeger spans' concept can be found in this model.
//! The full list of this mapping can be found in [OpenTelemetry to Jaeger Transformation].
//!
//! The **process tags** in jaeger spans will be mapped as resource in opentelemetry. You can
//! set it through `OTEL_RESOURCE_ATTRIBUTES` environment variable or using [`with_trace_config`].
//!
//! Note that to avoid copying data multiple times. Jaeger exporter will uses resource stored in [`Exporter`].
//!
//! The **tags** in jaeger spans will be mapped as attributes in opentelemetry spans. You can
//! set it through [`set_attribute`] method.
//!
//! Each jaeger span requires a **service name**. This will be mapped as a resource with `service.name` key.
//! You can set it using one of the following methods from highest priority to lowest priority.
//! 1. [`with_service_name`].
//! 2. include a `service.name` key value pairs when configure resource using [`with_trace_config`].
//! 3. set the service name as `OTEL_SERVICE_NAME` environment variable.
//! 4. set the `service.name` attributes in `OTEL_RESOURCE_ATTRIBUTES`.
//! 5. if the service name is not provided by the above method. `unknown_service` will be used.
//!
//! Based on the service name, we update/append the `service.name` process tags in jaeger spans.
//!
//! [`with_service_name`]: crate::exporter::config::agent::AgentPipeline::with_service_name
//! [`with_trace_config`]: crate::exporter::config::agent::AgentPipeline::with_trace_config
//! [`set_attribute`]: opentelemetry::trace::Span::set_attribute
//! [OpenTelemetry to Jaeger Transformation]:https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/trace/sdk_exporters/jaeger.md
//!
//! ## Kitchen Sink Full Configuration
//!
//! Example showing how to override all configuration options. See the
//! [`CollectorPipeline`] and [`AgentPipeline`] docs for details of each option.
//!
//! [`CollectorPipeline`]: config::collector::CollectorPipeline
//! [`AgentPipeline`]: config::agent::AgentPipeline
//!
//! ### Export to agents
//! ```no_run
//! use opentelemetry::{global, KeyValue, trace::{Tracer, TraceError}};
//! use opentelemetry_sdk::{trace::{config, RandomIdGenerator, Sampler}, Resource};
//! use opentelemetry_jaeger_propagator;
//!
//! fn main() -> Result<(), TraceError> {
//!     global::set_text_map_propagator(opentelemetry_jaeger_propagator::Propagator::new());
//!     let tracer = opentelemetry_jaeger::new_agent_pipeline()
//!         .with_endpoint("localhost:6831")
//!         .with_service_name("my_app")
//!         .with_max_packet_size(9_216)
//!         .with_auto_split_batch(true)
//!         .with_instrumentation_library_tags(false)
//!         .with_trace_config(
//!             config()
//!                 .with_sampler(Sampler::AlwaysOn)
//!                 .with_id_generator(RandomIdGenerator::default())
//!                 .with_max_events_per_span(64)
//!                 .with_max_attributes_per_span(16)
//!                  // resources will translated to tags in jaeger spans
//!                 .with_resource(Resource::new(vec![KeyValue::new("key", "value"),
//!                           KeyValue::new("process_key", "process_value")])),
//!         )
//!         .install_batch(opentelemetry_sdk::runtime::Tokio)?;
//!
//!     tracer.in_span("doing_work", |cx| {
//!         // Traced app logic here...
//!     });
//!
//!     // export remaining spans. It's optional if you can accept spans loss for the last batch.
//!     global::shutdown_tracer_provider();
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Export to collectors
//! Note that this example requires `collector_client` and `isahc_collector_client` feature.
//! ```ignore
//! use opentelemetry::{global, KeyValue, trace::{Tracer, TraceError}};
//! use opentelemetry_sdk::{trace::{config, RandomIdGenerator, Sampler}, Resource};
//! use opentelemetry_jaeger_propagator;
//!
//! fn main() -> Result<(), TraceError> {
//!     global::set_text_map_propagator(opentelemetry_jaeger_propagator::Propagator::new());
//!     let tracer = opentelemetry_jaeger::new_collector_pipeline()
//!         .with_endpoint("http://localhost:14250/api/trace") // set collector endpoint
//!         .with_service_name("my_app") // the name of the application
//!         .with_trace_config(
//!             config()
//!                 .with_sampler(Sampler::AlwaysOn)
//!                 .with_id_generator(RandomIdGenerator::default())
//!                 .with_max_events_per_span(64)
//!                 .with_max_attributes_per_span(16)
//!                 .with_max_events_per_span(16)
//!                 // resources will translated to tags in jaeger spans
//!                 .with_resource(Resource::new(vec![KeyValue::new("key", "value"),
//!                           KeyValue::new("process_key", "process_value")])),
//!         )
//!         // we config a surf http client with 2 seconds timeout
//!         // and have basic authentication header with username=username, password=s3cr3t
//!         .with_isahc() // requires `isahc_collector_client` feature
//!         .with_username("username")
//!         .with_password("s3cr3t")
//!         .with_timeout(std::time::Duration::from_secs(2))
//!         .install_batch(opentelemetry_sdk::runtime::Tokio)?;
//!
//!     tracer.in_span("doing_work", |cx| {
//!         // Traced app logic here...
//!     });
//!
//!     // export remaining spans. It's optional if you can accept spans loss for the last batch.
//!     global::shutdown_tracer_provider();
//!
//!     Ok(())
//! }
//! ```
//!
//! # Crate Feature Flags
//!
//! The following crate feature flags are available:
//!
//! * `collector_client`: Export span data directly to a Jaeger collector. User MUST provide the http client.
//!
//! * `hyper_collector_client`: Export span data with Jaeger collector backed by a hyper default http client.
//!
//! * `surf_collector_client`: Export span data with Jaeger collector backed by a surf default http client.
//!
//! * `reqwest_collector_client`: Export span data with Jaeger collector backed by a reqwest http client.
//!
//! * `reqwest_blocking_collector_client`: Export span data with Jaeger collector backed by a reqwest blocking http client.
//!
//! * `isahc_collector_client`: Export span data with Jaeger collector backed by a isahc http client.
//!
//! * `wasm_collector_client`: Enable collector in wasm.
//!
//! Support for recording and exporting telemetry asynchronously can be added
//! via the following flags, it extends the [`opentelemetry`] feature:
//!
//! * `rt-tokio`: Enable sending UDP packets to Jaeger agent asynchronously when the tokio
//!   [`Multi-Threaded Scheduler`] is used.
//!
//! * `rt-tokio-current-thread`: Enable sending UDP packets to Jaeger agent asynchronously when the
//!   tokio [`Current-Thread Scheduler`] is used.
//!
//! * `rt-async-std`: Enable sending UDP packets to Jaeger agent asynchronously when the
//!   [`async-std`] runtime is used.
//!
//! [`Multi-Threaded Scheduler`]: https://docs.rs/tokio/latest/tokio/runtime/index.html#multi-thread-scheduler
//! [`Current-Thread Scheduler`]: https://docs.rs/tokio/latest/tokio/runtime/index.html#current-thread-scheduler
//! [`async-std`]: https://async.rs
//! [`opentelemetry`]: https://crates.io/crates/opentelemetry
//!
//! # Supported Rust Versions
//!
//! OpenTelemetry is built against the latest stable release. The minimum
//! supported version is 1.64. The current OpenTelemetry version is not
//! guaranteed to build on Rust versions earlier than the minimum supported
//! version.
//!
//! The current stable Rust compiler and the three most recent minor versions
//! before it will always be supported. For example, if the current stable
//! compiler version is 1.64, the minimum supported version will not be
//! increased past 1.46, three minor versions prior. Increasing the minimum
//! supported compiler version is not considered a semver breaking change as
//! long as doing so complies with this policy.
#![warn(
    future_incompatible,
    missing_debug_implementations,
    missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    unreachable_pub,
    unused
)]
#![cfg_attr(
    docsrs,
    feature(doc_cfg, doc_auto_cfg),
    deny(rustdoc::broken_intra_doc_links)
)]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo.svg"
)]
#![cfg_attr(test, deny(warnings))]

pub use exporter::config;
#[cfg(feature = "collector_client")]
pub use exporter::config::collector::new_collector_pipeline;
#[cfg(feature = "wasm_collector_client")]
pub use exporter::config::collector::new_wasm_collector_pipeline;
pub use exporter::{
    config::agent::new_agent_pipeline, runtime::JaegerTraceRuntime, Error, Exporter, Process,
};

mod exporter;

#[cfg(feature = "integration_test")]
#[doc(hidden)]
pub mod testing;
