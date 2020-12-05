//! Collects OpenTelemetry spans and reports them to a given Jaeger
//! `agent` or `collector` endpoint. See the [Jaeger Docs] for details
//! about Jaeger and deployment information.
//!
//! *Compiler support: [requires `rustc` 1.42+][msrv]*
//!
//! [Jaeger Docs]: https://www.jaegertracing.io/docs/
//! [msrv]: #supported-rust-versions
//!
//! ### Quickstart
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
//! use opentelemetry::trace::Tracer;
//! use opentelemetry::global;
//!
//! fn main() -> Result<(), opentelemetry::trace::TraceError> {
//!     global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
//!     let (tracer, _uninstall) = opentelemetry_jaeger::new_pipeline().install()?;
//!
//!     tracer.in_span("doing_work", |cx| {
//!         // Traced app logic here...
//!     });
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Performance
//!
//! For optimal performance, a batch exporter is recommended as the simple
//! exporter will export each span synchronously on drop. You can enable the
//! [`tokio`] or [`async-std`] features to have a batch exporter configured for
//! you automatically for either executor when you install the pipeline.
//!
//! ```toml
//! [dependencies]
//! opentelemetry = { version = "*", features = ["tokio"] }
//! opentelemetry-jaeger = { version = "*", features = ["tokio"] }
//! ```
//!
//! [`tokio`]: https://tokio.rs
//! [`async-std`]: https://async.rs
//!
//! ### Jaeger Exporter From Environment Variables
//!
//! The jaeger pipeline builder can be configured dynamically via the
//! [`from_env`] method. All variables are optinal, a full list of accepted
//! options can be found in the [jaeger variables spec].
//!
//! [`from_env`]: struct.PipelineBuilder.html#method.from_env
//! [jaeger variables spec]: https://github.com/open-telemetry/opentelemetry-specification/blob/master/specification/sdk-environment-variables.md#jaeger-exporter
//!
//! ```no_run
//! use opentelemetry::trace::{Tracer, TraceError};
//! use opentelemetry::global;
//!
//! fn main() -> Result<(), TraceError> {
//!     global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
//!     // export OTEL_SERVICE_NAME=my-service-name
//!     let (tracer, _uninstall) = opentelemetry_jaeger::new_pipeline().from_env().install()?;
//!
//!     tracer.in_span("doing_work", |cx| {
//!         // Traced app logic here...
//!     });
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Jaeger Collector Example
//!
//! If you want to skip the agent and submit spans directly to a Jaeger collector,
//! you can enable the optional `collector_client` feature for this crate. This
//! example expects a Jaeger collector running on `http://localhost:14268`.
//!
//! ```toml
//! [dependencies]
//! opentelemetry-jaeger = { version = "..", features = ["collector_client"] }
//! ```
//!
//! Then you can use the [`with_collector_endpoint`] method to specify the endpoint:
//!
//! [`with_collector_endpoint`]: struct.PipelineBuilder.html#method.with_collector_endpoint
//!
//! ```ignore
//! // Note that this requires the `collector_client` feature.
//! use opentelemetry::trace::{Tracer, TraceError};
//!
//! fn main() -> Result<(), TraceError> {
//!     let (tracer, _uninstall) = opentelemetry_jaeger::new_pipeline()
//!         .with_collector_endpoint("http://localhost:14268/api/traces")
//!         // optionally set username and password as well.
//!         .with_collector_username("username")
//!         .with_collector_password("s3cr3t")
//!         .install()?;
//!
//!     tracer.in_span("doing_work", |cx| {
//!         // Traced app logic here...
//!     });
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Kitchen Sink Full Configuration
//!
//! Example showing how to override all configuration options. See the
//! [`PipelineBuilder`] docs for details of each option.
//!
//! [`PipelineBuilder`]: struct.PipelineBuilder.html
//!
//! ```no_run
//! use opentelemetry::{KeyValue, trace::{Tracer, TraceError}};
//! use opentelemetry::sdk::{trace::{self, IdGenerator, Sampler}, Resource};
//! use opentelemetry::global;
//!
//! fn main() -> Result<(), TraceError> {
//!     global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
//!     let (tracer, _uninstall) = opentelemetry_jaeger::new_pipeline()
//!         .from_env()
//!         .with_agent_endpoint("localhost:6831")
//!         .with_service_name("my_app")
//!         .with_tags(vec![KeyValue::new("process_key", "process_value")])
//!         .with_trace_config(
//!             trace::config()
//!                 .with_default_sampler(Sampler::AlwaysOn)
//!                 .with_id_generator(IdGenerator::default())
//!                 .with_max_events_per_span(64)
//!                 .with_max_attributes_per_span(16)
//!                 .with_max_events_per_span(16)
//!                 .with_resource(Resource::new(vec![KeyValue::new("key", "value")])),
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
//!
//! ## Crate Feature Flags
//!
//! The following crate feature flags are available:
//!
//! * `collector_client`: Export span data directly to a Jaeger collector using
//!   isahc http client.
//!
//! ## Supported Rust Versions
//!
//! OpenTelemetry is built against the latest stable release. The minimum
//! supported version is 1.42. The current OpenTelemetry version is not
//! guaranteed to build on Rust versions earlier than the minimum supported
//! version.
//!
//! The current stable Rust compiler and the three most recent minor versions
//! before it will always be supported. For example, if the current stable
//! compiler version is 1.45, the minimum supported version will not be
//! increased past 1.42, three minor versions prior. Increasing the minimum
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
#![cfg_attr(docsrs, feature(doc_cfg), deny(broken_intra_doc_links))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/master/assets/logo.svg"
)]
#![cfg_attr(test, deny(warnings))]

mod exporter;
mod propagator;

pub use exporter::{new_pipeline, Error, Exporter, PipelineBuilder, Process, Uninstall};
pub use propagator::Propagator;
