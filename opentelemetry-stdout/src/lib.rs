//! Export telemetry signals to stdout.
//! <div class="warning">This exporter is designed for debugging and learning purposes. It is not
//! recommended for use in production environments. The output format might not be
//! exhaustive and is subject to change at any time.
//! </div>
//!
//! # Feature Flags
//! The following feature flags can enable exporters for different telemetry signals:
//!
//! * `trace`: Includes the trace exporters.
//! * `metrics`: Includes the metrics exporters.
//! * `logs`: Includes the logs exporters.
//!
//! # Examples
//!
//! ```no_run
//! # #[cfg(all(feature = "metrics", feature = "trace", feature = "logs"))]
//! {
//! use opentelemetry::metrics::MeterProvider;
//! use opentelemetry::trace::{Span, Tracer, TracerProvider};
//! use opentelemetry::{Context, KeyValue};
//!
//! use opentelemetry_sdk::metrics::{SdkMeterProvider, PeriodicReader};
//! use opentelemetry_sdk::trace::SdkTracerProvider;
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
//!     let exporter = opentelemetry_stdout::MetricExporter::default();
//!     SdkMeterProvider::builder().with_periodic_exporter(exporter).build()
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
//! # }
//! ```
#![warn(missing_debug_implementations, missing_docs)]
#![cfg_attr(
    docsrs,
    feature(doc_cfg, doc_auto_cfg),
    deny(rustdoc::broken_intra_doc_links)
)]

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
