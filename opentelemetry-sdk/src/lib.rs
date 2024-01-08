//! Implements the [`SDK`] component of [OpenTelemetry].
//!
//! *Compiler support: [requires `rustc` 1.64+][msrv]*
//!
//! [`SDK`]: https://opentelemetry.io/docs/specs/otel/overview/#sdk
//! [OpenTelemetry]: https://opentelemetry.io/docs/what-is-opentelemetry/
//! [msrv]: #supported-rust-versions
//!
//! # Getting Started
//!
//! ```no_run
//! # #[cfg(feature = "trace")]
//! # {
//! use opentelemetry::{global, trace::{Tracer, TracerProvider as _}};
//! use opentelemetry_sdk::trace::TracerProvider;
//!
//! fn main() {
//!     // Choose an exporter like `opentelemetry_stdout::SpanExporter`
//!     # fn example<T: opentelemetry_sdk::export::trace::SpanExporter + 'static>(new_exporter: impl Fn() -> T) {
//!     let exporter = new_exporter();
//!
//!     // Create a new trace pipeline that prints to stdout
//!     let provider = TracerProvider::builder()
//!         .with_simple_exporter(exporter)
//!         .build();
//!     let tracer = provider.tracer("readme_example");
//!
//!     tracer.in_span("doing_work", |cx| {
//!         // Traced app logic here...
//!     });
//!
//!     // Shutdown trace pipeline
//!     global::shutdown_tracer_provider();
//!     # }
//! }
//! # }
//! ```
//!
//! See the [examples] directory for different integration patterns.
//!
//! See the API [`trace`] module docs for more information on creating and managing
//! spans.
//!
//! [examples]: https://github.com/open-telemetry/opentelemetry-rust/tree/main/examples
//! [`trace`]: https://docs.rs/opentelemetry/latest/opentelemetry/trace/index.html
//!
//! # Metrics (Beta)
//!
//! Note: the metrics implementation is **still in progress** and **subject to major
//! changes**.
//!
//! ### Creating instruments and recording measurements
//!
//! ```
//! # use opentelemetry::attributes::AttributeSet;
//! #[cfg(feature = "metrics")]
//! # {
//! use opentelemetry::{attributes::AttributeSet, global, KeyValue};
//!
//! // get a meter from a provider
//! let meter = global::meter("my_service");
//!
//! // create an instrument
//! let counter = meter.u64_counter("my_counter").init();
//!
//! // record a measurement
//! let attributes = AttributeSet::from(&[KeyValue::new("http.client_ip", "83.164.160.102")]);
//! counter.add(1, attributes);
//! # }
//! ```
//!
//! See the [examples] directory for different integration patterns.
//!
//! See the API [`metrics`] module docs for more information on creating and
//! managing instruments.
//!
//! [examples]: https://github.com/open-telemetry/opentelemetry-rust/tree/main/examples
//! [`metrics`]: https://docs.rs/opentelemetry/latest/opentelemetry/metrics/index.html
//!
//! ## Crate Feature Flags
//!
//! The following core crate feature flags are available:
//!
//! * `trace`: Includes the trace SDK (enabled by default).
//! * `metrics`: Includes the unstable metrics SDK.
//!
//! Support for recording and exporting telemetry asynchronously can be added
//! via the following flags:
//!
//! * `rt-tokio`: Spawn telemetry tasks using [tokio]'s multi-thread runtime.
//! * `rt-tokio-current-thread`: Spawn telemetry tasks on a separate runtime so that the main runtime won't be blocked.
//! * `rt-async-std`: Spawn telemetry tasks using [async-std]'s runtime.
//!
//! [tokio]: https://crates.io/crates/tokio
//! [async-std]: https://crates.io/crates/async-std
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

pub mod export;
mod instrumentation;
#[cfg(feature = "logs")]
#[cfg_attr(docsrs, doc(cfg(feature = "logs")))]
pub mod logs;
#[cfg(feature = "metrics")]
#[cfg_attr(docsrs, doc(cfg(feature = "metrics")))]
pub mod metrics;
#[cfg(feature = "trace")]
#[cfg_attr(docsrs, doc(cfg(feature = "trace")))]
pub mod propagation;
pub mod resource;
pub mod runtime;
#[cfg(any(feature = "testing", test))]
#[doc(hidden)]
pub mod testing;
#[cfg(feature = "trace")]
#[cfg_attr(docsrs, doc(cfg(feature = "trace")))]
pub mod trace;

#[doc(hidden)]
pub mod util;

pub use instrumentation::{InstrumentationLibrary, Scope};
#[doc(inline)]
pub use resource::Resource;
