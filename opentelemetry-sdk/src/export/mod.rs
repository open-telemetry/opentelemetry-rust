//! Telemetry Export

#[cfg(feature = "trace")]
#[cfg_attr(docsrs, doc(cfg(feature = "trace")))]
pub mod trace;

pub use opentelemetry_api::ExportError;
