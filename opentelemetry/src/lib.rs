//! Implements the [`API`] component of [OpenTelemetry].
//!
//! *Compiler support: [requires `rustc` 1.64+][msrv]*
//!
//! [`API`]: https://opentelemetry.io/docs/specs/otel/overview/#api
//! [OpenTelemetry]: https://opentelemetry.io/docs/what-is-opentelemetry/
//! [msrv]: #supported-rust-versions
//!
//! # Getting Started
//!
//! ```no_run
//! # #[cfg(feature = "trace")]
//! # {
//! use opentelemetry::{global, trace::{TraceContextExt, Tracer}, Context };
//!
//! fn do_something() {
//!     let tracer = global::tracer("my_component");
//!     let _guard = Context::current_with_span(tracer.start("my_span")).attach();
//!     // do work tracked by the now current span
//! }
//! # }
//! ```
//!
//! See the [examples] directory for different integration patterns.
//!
//! [examples]: https://github.com/open-telemetry/opentelemetry-rust/tree/main/examples
//!
//! # Traces
//!
//! The [`trace`] module includes types for tracking the progression of a single
//! request while it is handled by services that make up an application. A trace
//! is a tree of [`Span`]s which are objects that represent the work being done
//! by individual services or components involved in a request as it flows
//! through a system.
//!
//! ### Creating and exporting spans
//!
//! ```
//! # #[cfg(feature = "trace")]
//! # {
//! use opentelemetry::{global, trace::{Span, Tracer}, KeyValue};
//!
//! // get a tracer from a provider
//! let tracer = global::tracer("my_service");
//!
//! // start a new span
//! let mut span = tracer.start("my_span");
//!
//! // set some attributes
//! span.set_attribute(KeyValue::new("http.client_ip", "83.164.160.102"));
//!
//! // perform some more work...
//!
//! // end or drop the span to export
//! span.end();
//! # }
//! ```
//!
//! See the [`trace`] module docs for more information on creating and managing
//! spans.
//!
//! [`Span`]: crate::trace::Span
//!
//! # Metrics
//!
//!
//! The [`metrics`] module includes types for recording measurements about a
//! service at runtime.
//!
//! ### Creating instruments and recording measurements
//!
//! ```
//! # #[cfg(feature = "metrics")]
//! # {
//! use opentelemetry::{attributes::AttributeSet, global, KeyValue};
//!
//! // get a meter from a provider
//! let meter = global::meter("my_service");
//!
//! // create an instrument
//! let counter = meter.u64_counter("my_counter").init();
//!
//! // Form the attributes
//! let attributes = AttributeSet::from(&[KeyValue::new("http.client_ip", "83.164.160.102")]);
//!
//! // record a measurement
//! counter.add(1, attributes);
//! # }
//! ```
//!
//! See the [`metrics`] module docs for more information on creating and
//! managing instruments.
//!
//! ## Crate Feature Flags
//!
//! The following core crate feature flags are available:
//!
//! * `trace`: Includes the trace API (enabled by default).
//! * `metrics`: Includes the unstable metrics API.
//! * `logs`: Includes the logs bridge API.
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
//! - [`opentelemetry_sdk`] provides the SDK used to configure providers.
//! - [`opentelemetry-http`] provides an interface for injecting and extracting
//!   trace information from [`http`] headers.
//! - [`opentelemetry-jaeger`] provides a pipeline and exporter for sending
//!   trace information to [`Jaeger`].
//! - [`opentelemetry-otlp`] exporter for sending trace and metric data in the
//!   OTLP format to the OpenTelemetry collector.
//! - [`opentelemetry-prometheus`] provides a pipeline and exporter for sending
//!   metrics information to [`Prometheus`].
//! - [`opentelemetry-zipkin`] provides a pipeline and exporter for sending
//!   trace information to [`Zipkin`].
//! - [`opentelemetry-datadog`] provides additional exporters to [`Datadog`].
//! - [`opentelemetry-aws`] provides unofficial propagators for AWS X-ray.
//! - [`opentelemetry-contrib`] provides additional exporters and propagators that are
//!   experimental.
//! - [`opentelemetry-semantic-conventions`] provides standard names and
//!   semantic otel conventions.
//! - [`opentelemetry-stackdriver`] provides an exporter for Google's [Cloud Trace]
//!   (which used to be called StackDriver).
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
//!
//! If you're the maintainer of an `opentelemetry` ecosystem crate not listed
//! above, please let us know! We'd love to add your project to the list!
//!
//! [`actix-web-opentelemetry`]: https://crates.io/crates/actix-web-opentelemetry
//! [`actix-web`]: https://crates.io/crates/actix-web
//! [`Datadog`]: https://www.datadoghq.com
//! [`http`]: https://crates.io/crates/http
//! [`Jaeger`]: https://www.jaegertracing.io
//! [`open-telemetry/opentelemetry-rust`]: https://github.com/open-telemetry/opentelemetry-rust
//! [`opentelemetry_sdk`]: https://crates.io/crates/opentelemetry_sdk
//! [`opentelemetry-application-insights`]: https://crates.io/crates/opentelemetry-application-insights
//! [`opentelemetry-aws`]: https://crates.io/crates/opentelemetry-aws
//! [`opentelemetry-contrib`]: https://crates.io/crates/opentelemetry-contrib
//! [`opentelemetry-datadog`]: https://crates.io/crates/opentelemetry-datadog
//! [`opentelemetry-http`]: https://crates.io/crates/opentelemetry-http
//! [`opentelemetry-jaeger`]: https://crates.io/crates/opentelemetry-jaeger
//! [`opentelemetry-otlp`]: https://crates.io/crates/opentelemetry-otlp
//! [`opentelemetry-prometheus`]: https://crates.io/crates/opentelemetry-prometheus
//! [`opentelemetry-semantic-conventions`]: https://crates.io/crates/opentelemetry-semantic-conventions
//! [`opentelemetry-stackdriver`]: https://crates.io/crates/opentelemetry-stackdriver
//! [`opentelemetry-tide`]: https://crates.io/crates/opentelemetry-tide
//! [`opentelemetry-zipkin`]: https://crates.io/crates/opentelemetry-zipkin
//! [`Prometheus`]: https://prometheus.io
//! [`Tide`]: https://crates.io/crates/tide
//! [`tracing-opentelemetry`]: https://crates.io/crates/tracing-opentelemetry
//! [`tracing`]: https://crates.io/crates/tracing
//! [`Zipkin`]: https://zipkin.io
//! [Azure Application Insights]: https://docs.microsoft.com/en-us/azure/azure-monitor/app/app-insights-overview
//! [Cloud Trace]: https://cloud.google.com/trace/
//!
//! ## Supported Rust Versions
//!
//! OpenTelemetry is built against the latest stable release. The minimum
//! supported version is 1.64. The current OpenTelemetry version is not
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
#![cfg_attr(
    docsrs,
    feature(doc_cfg, doc_auto_cfg),
    deny(rustdoc::broken_intra_doc_links)
)]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/open-telemetry/opentelemetry-rust/main/assets/logo.svg"
)]
#![cfg_attr(test, deny(warnings))]

pub mod global;

pub mod baggage;

pub mod attributes;

mod context;

pub use context::{Context, ContextGuard};

mod common;

#[cfg(any(feature = "testing", test))]
#[doc(hidden)]
pub mod testing;

pub use common::{Array, ExportError, InstrumentationLibrary, Key, KeyValue, StringValue, Value};

#[cfg(feature = "metrics")]
#[cfg_attr(docsrs, doc(cfg(feature = "metrics")))]
pub mod metrics;

pub mod propagation;

#[cfg(feature = "trace")]
#[cfg_attr(docsrs, doc(cfg(feature = "trace")))]
pub mod trace;

#[cfg(feature = "logs")]
#[cfg_attr(docsrs, doc(cfg(feature = "logs")))]
pub mod logs;

#[doc(hidden)]
#[cfg(any(feature = "metrics", feature = "trace"))]
pub mod time {
    use std::time::SystemTime;

    #[doc(hidden)]
    #[cfg(any(
        not(target_arch = "wasm32"),
        all(target_arch = "wasm32", target_os = "wasi")
    ))]
    pub fn now() -> SystemTime {
        SystemTime::now()
    }

    #[doc(hidden)]
    #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
    pub fn now() -> SystemTime {
        SystemTime::UNIX_EPOCH + std::time::Duration::from_millis(js_sys::Date::now() as u64)
    }
}
