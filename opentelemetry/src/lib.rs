//! The Rust [OpenTelemetry](https://opentelemetry.io/) implementation.
//!
//! OpenTelemetry provides a single set of APIs, libraries, agents, and collector
//! services to capture distributed traces and metrics from your application. You
//! can analyze them using [Prometheus], [Jaeger], and other observability tools.
//!
//! *Compiler support: [requires `rustc` 1.46+][msrv]*
//!
//! [Prometheus]: https://prometheus.io
//! [Jaeger]: https://www.jaegertracing.io
//! [msrv]: #supported-rust-versions
//!
//! ## Getting Started
//!
//! ```no_run
//! # #[cfg(feature = "trace")]
//! # {
//! use opentelemetry::{sdk::export::trace::stdout, trace::Tracer};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
//!     // Create a new instrumentation pipeline
//!     let (tracer, _uninstall) = stdout::new_pipeline().install();
//!
//!     tracer.in_span("doing_work", |cx| {
//!         // Traced app logic here...
//!     });
//!
//!     Ok(())
//! }
//! }
//! ```
//!
//! See the [examples](https://github.com/open-telemetry/opentelemetry-rust/tree/main/examples)
//! directory for different integration patterns.
//!
//! ## Crate Feature Flags
//!
//! The following core crate feature flags are available:
//!
//! * `trace`: Includes the trace API and SDK (enabled by default).
//! * `metrics`: Includes the unstable metrics API and SDK.
//! * `serialize`: Adds [serde] serializers for common types.
//!
//! Support for recording and exporting telemetry asynchronously can be added
//! via the following flags:
//!
//! * `tokio-support`: Spawn telemetry tasks using [tokio]'s runtime.
//! * `async-std`: Spawn telemetry tasks using [async-std]'s runtime.
//!
//! The following flags enable propagating information in other crate
//! ecosystems:
//!
//! * `http`: Propagate information via [http] header maps.
//! * `tonic`: Propagate information via [tonic]'s metadata.
//!
//! Finally the following flags can be used by exporter authors:
//!
//! * `reqwest`: Implementation of [`HttpClient`] for [reqwest].
//! * `surf`: Implementation of [`HttpClient`] for [surf]`.
//!
//! [tokio]: https://crates.io/crates/tokio
//! [async-std]: https://crates.io/crates/async-std
//! [serde]: https://crates.io/crates/serde
//! [http]: https://crates.io/crates/http
//! [tonic]: https://crates.io/crates/tonic
//! [`HttpClient`]: crate::sdk::export::trace::HttpClient
//! [reqwest]: https://crates.io/crates/reqwest
//! [surf]: https://crates.io/crates/surf
//!
//! ## Related Crates
//!
//! In addition to `opentelemetry`, the [`open-telemetry/opentelemetry-rust`]
//! repository contains several additional crates designed to be used with the
//! `opentelemetry` ecosystem. This includes a collection of trace
//! `SpanExporter` and metrics pull and push controller implementations, as well
//! as utility and adapter crates to assist in propagating state and
//! instrumenting applications.
//!
//! In particular, the following crates are likely to be of interest:
//!
//! - [`opentelemetry-jaeger`] provides a pipeline and exporter for sending
//!   trace information to [`Jaeger`].
//! - [`opentelemetry-otlp`] exporter for sending trace and metric data in the
//!   OTLP format to the OpenTelemetry collector.
//! - [`opentelemetry-prometheus`] provides a pipeline and exporter for sending
//!   metrics information to [`Prometheus`].
//! - [`opentelemetry-zipkin`] provides a pipeline and exporter for sending
//!   trace information to [`Zipkin`].
//! - [`opentelemetry-contrib`] provides additional exporters to vendors like
//!   [`Datadog`].
//! - [`opentelemetry-semantic-conventions`] provides standard names and
//!   semantic otel conventions.
//!
//! Additionally, there are also several third-party crates which are not
//! maintained by the `opentelemetry` project. These include:
//!
//! - [`tracing-opentelemetry`] provides integration for applications
//!   instrumented using the [`tracing`] API and ecosystem.
//! - [`actix-web-opentelemetry`] provides integration for the [`actix-web`] web
//!   server and ecosystem.
//! - [`opentelemetry-application-insights`] provides an unofficial [Azure
//!   Application Insights] exporter.
//! - [`opentelemetry-tide`] provides integration for the [`Tide`] web server
//!   and ecosystem.
//! - [`opentelemetry-stackdriver`] provides an exporter for Google's [Cloud Trace]
//!   (which used to be called StackDriver).
//!
//! If you're the maintainer of an `opentelemetry` ecosystem crate not listed
//! above, please let us know! We'd love to add your project to the list!
//!
//! [`open-telemetry/opentelemetry-rust`]: https://github.com/open-telemetry/opentelemetry-rust
//! [`opentelemetry-jaeger`]: https://crates.io/crates/opentelemetry-jaeger
//! [`Jaeger`]: https://www.jaegertracing.io
//! [`opentelemetry-otlp`]: https://crates.io/crates/opentelemetry-otlp
//! [`opentelemetry-prometheus`]: https://crates.io/crates/opentelemetry-prometheus
//! [`Prometheus`]: https://prometheus.io
//! [`opentelemetry-zipkin`]: https://crates.io/crates/opentelemetry-zipkin
//! [`Zipkin`]: https://zipkin.io
//! [`opentelemetry-contrib`]: https://crates.io/crates/opentelemetry-contrib
//! [`Datadog`]: https://www.datadoghq.com
//! [`opentelemetry-semantic-conventions`]: https://crates.io/crates/opentelemetry-semantic-conventions
//!
//! [`tracing-opentelemetry`]: https://crates.io/crates/tracing-opentelemetry
//! [`tracing`]: https://crates.io/crates/tracing
//! [`actix-web-opentelemetry`]: https://crates.io/crates/actix-web-opentelemetry
//! [`actix-web`]: https://crates.io/crates/actix-web
//! [`opentelemetry-application-insights`]: https://crates.io/crates/opentelemetry-application-insights
//! [Azure Application Insights]: https://docs.microsoft.com/en-us/azure/azure-monitor/app/app-insights-overview
//! [`opentelemetry-tide`]: https://crates.io/crates/opentelemetry-tide
//! [`Tide`]: https://crates.io/crates/tide
//! [`opentelemetry-stackdriver`]: https://crates.io/crates/opentelemetry-stackdriver
//! [Cloud Trace]: https://cloud.google.com/trace/
//!
//! ## Supported Rust Versions
//!
//! OpenTelemetry is built against the latest stable release. The minimum
//! supported version is 1.46. The current OpenTelemetry version is not
//! guaranteed to build on Rust versions earlier than the minimum supported
//! version.
//!
//! The current stable Rust compiler and the three most recent minor versions
//! before it will always be supported. For example, if the current stable
//! compiler version is 1.49, the minimum supported version will not be
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
#![allow(clippy::needless_doctest_main)]
#![cfg_attr(docsrs, feature(doc_cfg), deny(broken_intra_doc_links))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo.svg"
)]
#![cfg_attr(test, deny(warnings))]

pub mod global;
pub mod sdk;

#[cfg(feature = "testing")]
#[allow(missing_docs)]
pub mod testing;

pub mod baggage;

mod context;
pub use context::{Context, ContextGuard};

mod core;
pub use crate::core::{Array, Key, KeyValue, Unit, Value};

pub mod util;

#[cfg(feature = "metrics")]
#[cfg_attr(docsrs, doc(cfg(feature = "metrics")))]
pub mod labels;

#[cfg(feature = "metrics")]
#[cfg_attr(docsrs, doc(cfg(feature = "metrics")))]
pub mod metrics;

pub mod propagation;

#[cfg(feature = "trace")]
#[cfg_attr(docsrs, doc(cfg(feature = "trace")))]
pub mod trace;

#[cfg(any(feature = "metrics", feature = "trace"))]
pub(crate) mod time {
    use std::time::SystemTime;

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn now() -> SystemTime {
        SystemTime::now()
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn now() -> SystemTime {
        SystemTime::UNIX_EPOCH + std::time::Duration::from_millis(js_sys::Date::now() as u64)
    }
}
