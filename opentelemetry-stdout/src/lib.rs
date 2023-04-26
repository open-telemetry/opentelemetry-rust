//! Export telemetry signals to stdout.
//!
//! # Examples
//!
//! ```no_run
//! # #[cfg(all(feature = "metrics", feature = "trace"))]
//! {
//! use opentelemetry_api::metrics::MeterProvider as _;
//! use opentelemetry_api::trace::{Span, Tracer, TracerProvider as _};
//! use opentelemetry_api::{Context, KeyValue};
//!
//! use opentelemetry_sdk::metrics::{MeterProvider, PeriodicReader};
//! use opentelemetry_sdk::runtime;
//! use opentelemetry_sdk::trace::TracerProvider;
//!
//! fn init_trace() -> TracerProvider {
//!     let exporter = opentelemetry_stdout::SpanExporter::default();
//!     TracerProvider::builder()
//!         .with_simple_exporter(exporter)
//!         .build()
//! }
//!
//! fn init_metrics() -> MeterProvider {
//!     let exporter = opentelemetry_stdout::MetricsExporter::default();
//!     let reader = PeriodicReader::builder(exporter, runtime::Tokio).build();
//!     MeterProvider::builder().with_reader(reader).build()
//! }
//!
//! let tracer_provider = init_trace();
//! let meter_provider = init_metrics();
//!
//! // recorded traces and metrics will now be sent to stdout:
//!
//! // {"resourceMetrics":{"resource":{"attributes":[{"key":"service.name","value":{"str..
//! // {"resourceSpans":[{"resource":{"attributes":[{"key":"service.name","value":{"stri..
//! # }
//! ```
#![warn(missing_debug_implementations, missing_docs)]

pub(crate) mod common;

#[cfg(feature = "metrics")]
mod metrics;
#[cfg_attr(docsrs, doc(cfg(feature = "metrics")))]
#[cfg(feature = "metrics")]
pub use metrics::*;

#[cfg(feature = "trace")]
mod trace;
#[cfg_attr(docsrs, doc(cfg(feature = "trace")))]
#[cfg(feature = "trace")]
pub use trace::*;

#[cfg(feature = "logs")]
mod logs;
#[cfg_attr(docsrs, doc(cfg(feature = "logs")))]
#[cfg(feature = "logs")]
pub use logs::*;
