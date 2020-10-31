//! # OpenTelemetry data exporters
//!
//! There are two main types of exports:
//!
//! - Span exporters, which export instances of `Span`.
//! - Measurement exporters, which export data collected by `Meter` instances.
//!
//! Exporters define the interface that protocol-specific exporters must
//! implement so that they can be plugged into OpenTelemetry SDK and support
//! sending of telemetry data.
#[cfg(feature = "metrics")]
#[cfg_attr(docsrs, doc(cfg(feature = "metrics")))]
pub mod metrics;
#[cfg(feature = "trace")]
#[cfg_attr(docsrs, doc(cfg(feature = "trace")))]
pub mod trace;
