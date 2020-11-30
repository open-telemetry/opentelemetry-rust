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

/// Marker trait for errors returned by exporters
pub trait ExportError: std::error::Error + Send + Sync + 'static {
    /// The name of exporter that returned this error
    fn exporter_name(&self) -> &'static str;
}
