//! Implements the [`API`] component of [OpenTelemetry].
//!
//! *[Supported Rust Versions](#supported-rust-versions)*
//!
//! [`API`]: https://opentelemetry.io/docs/specs/otel/overview/#api
//! [OpenTelemetry]: https://opentelemetry.io/docs/what-is-opentelemetry/
//!
//! [Jaeger]: https://www.jaegertracing.io/
//! [Prometheus]: https://www.prometheus.io/
//! 
//! # Overview

//! OpenTelemetry is an Observability framework and toolkit designed to create and
//! manage telemetry data such as traces, metrics, and logs. OpenTelemetry is
//! vendor- and tool-agnostic, meaning that it can be used with a broad variety of
//! Observability backends, including open source tools like [Jaeger] and
//! [Prometheus], as well as commercial offerings.

//! OpenTelemetry is *not* an observability backend like Jaeger, Prometheus, or other
//! commercial vendors. OpenTelemetry is focused on the generation, collection,
//! management, and export of telemetry. A major goal of OpenTelemetry is that you
//! can easily instrument your applications or systems, no matter their language,
//! infrastructure, or runtime environment. Crucially, the storage and visualization
//! of telemetry is intentionally left to other tools.
//! 
//! ## What does this crate contain?

//! This crate is basic foundation for integrating OpenTelemetry into libraries and
//! applications, encompassing several aspects of OpenTelemetry, such as context
//! management and propagation, baggage, logging, tracing, and metrics. It follows
//! the [OpenTelemetry
//! specification](https://github.com/open-telemetry/opentelemetry-specification).
//! Here's a breakdown of its components:
//!
//! - **[Context
//!   API](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/context/README.md):**
//!   Provides a way to manage and propagate context, which is essential for keeping
//!   track of trace execution across asynchronous tasks.
//! - **[Propagators
//!   API](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/context/api-propagators.md):**
//!   Defines how context can be shared across process boundaries, ensuring
//!   continuity across microservices or distributed systems.
//! - **[Baggage
//!   API](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/baggage/api.md):**
//!   Allows for the attachment of metadata (baggage) to telemetry, which can be
//!   used for sharing application-specific information across service boundaries.
//! - **[Logs Bridge
//!   API](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/logs/api.md):**
//!   Allows to bridge existing logging mechanisms with OpenTelemetry logging. This
//!   is **NOT** meant for end users to call, instead it is meant to enable writing
//!   bridges/appenders for existing logging mechanisms such as
//!   [log](https://crates.io/crates/log) or
//!   [tracing](https://crates.io/crates/tracing).
//! - **[Tracing
//!   API](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/trace/api.md):**
//!   Offers a set of primitives to produce distributed traces to understand the
//!   flow of a request across system boundaries.
//! - **[Metrics
//!   API](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/api.md):**
//!   Offers a set of primitives to produce measurements of operational metrics like
//!   latency, throughput, or error rates.
//!
//! This crate serves as a facade or no-op implementation, meaning it defines the
//! traits for instrumentation but does not itself implement the processing or
//! exporting of telemetry data. This separation of concerns allows library authors
//! to depend on the API crate without tying themselves to a specific
//! implementation.
//!
//! Actual implementation and the heavy lifting of telemetry data collection,
//! processing, and exporting are delegated to the
//! [opentelemetry-sdk](https://crates.io/crates/opentelemetry-sdk) crate and
//! various exporter crates such as
//! [opentelemetry-otlp](https://crates.io/crates/opentelemetry-otlp). This
//! architecture ensures that the final application can light up the instrumentation
//! by integrating an SDK implementation.
//!
//! Library authors are recommended to depend on this crate *only*. This approach is
//! also aligned with the design philosophy of existing telemetry solutions in the
//! Rust ecosystem, like `tracing` or `log`, where these crates only offer a facade
//! and the actual functionality is enabled through additional crates.
//!
//! ## Related crates

//! Unless you are a library author, you will almost always need to use additional
//! crates along with this. Given this crate has no-op implementation only, an
//! OpenTelemetry SDK is always required.
//! [opentelemetry-sdk](https://crates.io/crates/opentelemetry-sdk) is the official
//! SDK implemented by OpenTelemetry itself, though it is possible to use a
//! different sdk.
//!
//! Additionally one or more exporters are also required to export telemetry to a
//! destination. OpenTelemetry provides the following exporters:
//!
//! - **[opentelemetry-stdout](https://crates.io/crates/opentelemetry-stdout):**
//!   Prints telemetry to stdout, primarily used for learning/debugging purposes.
//! - **[opentelemetry-otlp](https://crates.io/crates/opentelemetry-otlp):** Exports
//!   telemetry (logs, metrics and traces) in the [OTLP
//!   format](https://github.com/open-telemetry/opentelemetry-specification/tree/main/specification/protocol)
//!   to an endpoint accepting OTLP. This could be the [OTel
//!   Collector](https://github.com/open-telemetry/opentelemetry-collector),
//!   telemetry backends like [Jaeger](https://www.jaegertracing.io/),
//!   [Prometheus](https://prometheus.io/docs/prometheus/latest/feature_flags/#otlp-receiver)
//!   or [vendor specific endpoints](https://opentelemetry.io/ecosystem/vendors/).
//! - **[opentelemetry-zipkin](https://crates.io/crates/opentelemetry-zipkin):**
//!   Exports telemetry (traces only) to Zipkin following [OpenTelemetry to Zipkin
//!   specification](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/trace/sdk_exporters/zipkin.md)
//! - **[opentelemetry-prometheus](https://crates.io/crates/opentelemetry-prometheus):**
//!   Exports telemetry (metrics only) to Prometheus following [OpenTelemetry to
//!   Prometheus
//!   specification](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/sdk_exporters/prometheus.md)
//!
//! OpenTelemetry Rust also has a [contrib
//! repo](https://github.com/open-telemetry/opentelemetry-rust-contrib), where
//! additional exporters could be found. Check [OpenTelemetry
//! Registry](https://opentelemetry.io/ecosystem/registry/?language=rust) for
//! additional exporters and other related components as well.
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
//! ## Traces
//!
//! The [`trace`] module includes types for tracking the progression of a single
//! request while it is handled by services that make up an application. A trace
//! is a tree of [`Span`]s which are objects that represent the work being done
//! by individual services or components involved in a request as it flows
//! through a system.
//!
//! ### Creating spans
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
//! ## Metrics
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
//! use opentelemetry::{global, KeyValue};
//!
//! // get a meter from a provider
//! let meter = global::meter("my_service");
//!
//! // create an instrument
//! let counter = meter.u64_counter("my_counter").build();
//!
//! // record a measurement
//! counter.add(1, &[KeyValue::new("http.client_ip", "83.164.160.102")]);
//! # }
//! ```
//!
//! See the [`metrics`] module docs for more information on creating and
//! managing instruments.
//!
//!
//! ## Logs
//!
//!  The [`logs`] module contains the Logs Bridge API. It is not intended to be
//!  called by application developers directly. It is provided for logging
//!  library authors to build log appenders, that bridges existing logging
//!  systems with OpenTelemetry. Bridges for
//!  [`log`](https://crates.io/crates/log) and
//!  [`tracing`](https://crates.io/crates/tracing) libraries are provided via
//!  the
//!  [`opentelemetry-appender-log`](https://crates.io/crates/opentelemetry-appender-log)
//!  and
//!  [`opentelemetry-appender-tracing`](https://crates.io/crates/opentelemetry-appender-tracing)
//!  crates.
//!
//! ## Feature Flags
//!
//! The following core crate feature flags are available:
//!
//! * `trace`: Includes the trace API.
//! * `metrics`: Includes the metrics API.
//! * `logs`: Includes the logs bridge API.
//! * `internal-logs`: Enables internal logging via `tracing`.
//!
//! The default feature flags are ["trace", "metrics", "logs", "internal-logs"].
//!
//! The following feature flags provides additional configuration for `logs`:
//! * `spec_unstable_logs_enabled`: Allow users to control the log level
//!
//! The following feature flags enable APIs defined in OpenTelemetry specification that is in experimental phase:
//! * `otel_unstable`: Includes unstable APIs.
//!
//!
//! [`http`]: https://crates.io/crates/http
//! [`open-telemetry/opentelemetry-rust`]: https://github.com/open-telemetry/opentelemetry-rust
//! [`opentelemetry_sdk`]: https://crates.io/crates/opentelemetry_sdk
//! [`opentelemetry-http`]: https://crates.io/crates/opentelemetry-http
//! [`opentelemetry-otlp`]: https://crates.io/crates/opentelemetry-otlp
//! [`opentelemetry-prometheus`]: https://crates.io/crates/opentelemetry-prometheus
//! [`opentelemetry-zipkin`]: https://crates.io/crates/opentelemetry-zipkin
//! [`Prometheus`]: https://prometheus.io
//! [`Zipkin`]: https://zipkin.io
//!
//! ## Supported Rust Versions
//!
//! OpenTelemetry is built against the latest stable release. The minimum
//! supported version is 1.70. The current OpenTelemetry version is not
//! guaranteed to build on Rust versions earlier than the minimum supported
//! version.
//! This crate is built against the latest stable release. The minimum supported
//! version is 1.70. The current version is not guaranteed to build on Rust
//! versions earlier than the minimum supported version.
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

mod context;

pub use context::{Context, ContextGuard};

mod common;

#[cfg(any(feature = "testing", test))]
#[doc(hidden)]
pub mod testing;

pub use common::{
    Array, InstrumentationScope, InstrumentationScopeBuilder, Key, KeyValue, StringValue, Value,
};

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