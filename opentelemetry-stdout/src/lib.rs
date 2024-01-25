//! Export telemetry signals to stdout.
//! This exporter is designed for debugging and learning purposes. It is not
//! recommended for use in production environments. The output format might not be
//! exhaustive and is subject to change at any time.
//!
//! # Examples
//!
//! ```no_run
//! # #[cfg(all(feature = "metrics", feature = "trace", feature = "logs"))]
//! {
//! use opentelemetry::metrics::MeterProvider;
//! use opentelemetry::trace::{Span, Tracer, TracerProvider as _};
//! use opentelemetry::{Context, KeyValue};
//!
//! use opentelemetry_sdk::metrics::{SdkMeterProvider, PeriodicReader};
//! use opentelemetry_sdk::runtime;
//! use opentelemetry_sdk::trace::TracerProvider;
//!
//! use opentelemetry_sdk::logs::LoggerProvider;
//!
//! fn init_trace() -> TracerProvider {
//!     let exporter = opentelemetry_stdout::SpanExporter::default();
//!     TracerProvider::builder()
//!         .with_simple_exporter(exporter)
//!         .build()
//! }
//!
//! fn init_metrics() -> SdkMeterProvider {
//!     let exporter = opentelemetry_stdout::MetricsExporter::default();
//!     let reader = PeriodicReader::builder(exporter, runtime::Tokio).build();
//!     SdkMeterProvider::builder().with_reader(reader).build()
//! }
//!
//! fn init_logs() -> LoggerProvider {
//!     let exporter = opentelemetry_stdout::LogExporter::default();
//!     LoggerProvider::builder()
//!         .with_simple_exporter(exporter)
//!         .build()
//! }
//!
//! let tracer_provider = init_trace();
//! let meter_provider = init_metrics();
//! let logger_provider = init_logs();
//!
//! // recorded traces, metrics and logs will now be sent to stdout:
//!
//! // {"resourceMetrics":{"resource":{"attributes":[{"key":"service.name","value":{"str..
//! // {"resourceSpans":[{"resource":{"attributes":[{"key":"service.name","value":{"stri..
//! // {"resourceLogs": [{"resource": {"attributes": [{"key": "service.name", "value": {"str..
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
