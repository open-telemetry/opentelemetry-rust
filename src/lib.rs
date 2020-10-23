//! The Rust [OpenTelemetry](https://opentelemetry.io/) implementation.
//!
//! OpenTelemetry provides a single set of APIs, libraries, agents, and collector
//! services to capture distributed traces and metrics from your application. You
//! can analyze them using [Prometheus], [Jaeger], and other observability tools.
//!
//! [Prometheus]: https://prometheus.io
//! [Jaeger]: https://www.jaegertracing.io
//!
//! ## Getting Started
//!
//! ```no_run
//! use opentelemetry::{api::trace::Tracer, exporter::trace::stdout};
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
//! ```
//!
//! See the [examples](https://github.com/open-telemetry/opentelemetry-rust/tree/master/examples)
//! directory for different integration patterns.
#![recursion_limit = "256"]
#![allow(clippy::needless_doctest_main)]
#![warn(
    future_incompatible,
    missing_debug_implementations,
    missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    rustdoc,
    unreachable_pub,
    unused
)]
#![cfg_attr(test, deny(warnings))]

pub mod api;
#[cfg(feature = "trace")]
pub mod experimental;
pub mod exporter;
pub mod global;
pub mod sdk;
