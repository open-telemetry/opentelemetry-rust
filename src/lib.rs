//! The Rust [OpenTelemetry](https://opentelemetry.io/) implementation.
//!
//! OpenTelemetry provides a single set of APIs, libraries, agents, and collector
//! services to capture distributed traces and metrics from your application. You
//! can analyze them using [Prometheus], [Jaeger], and other observability tools.
//!
//! *Compiler support: [requires `rustc` 1.42+][msrv]*
//!
//! [Prometheus]: https://prometheus.io
//! [Jaeger]: https://www.jaegertracing.io
//! [msrv]: #supported-rust-versions
//!
//! ## Getting Started
//!
//! ```no_run
//! use opentelemetry::{exporter::trace::stdout, trace::Tracer};
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
#![allow(clippy::needless_doctest_main)]
#![cfg_attr(docsrs, feature(doc_cfg), deny(broken_intra_doc_links))]
#![cfg_attr(test, deny(warnings))]

mod api;
#[cfg(feature = "trace")]
#[cfg_attr(docsrs, doc(cfg(feature = "trace")))]
pub mod experimental;
pub mod exporter;
pub mod global;
pub mod sdk;

#[cfg(test)]
pub mod testing;

#[cfg(feature = "metrics")]
#[cfg_attr(docsrs, doc(cfg(feature = "metrics")))]
pub use api::metrics;
#[cfg(feature = "trace")]
#[cfg_attr(docsrs, doc(cfg(feature = "trace")))]
pub use api::trace;
pub use api::{
    baggage,
    context::{Context, ContextGuard},
    core::{Key, KeyValue, Unit, Value},
    labels, propagation,
};
