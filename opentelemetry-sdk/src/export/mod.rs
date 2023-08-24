//! Telemetry Export

#[cfg(feature = "logs")]
#[cfg_attr(docsrs, doc(cfg(feature = "logs")))]
pub mod logs;

#[cfg(feature = "trace")]
#[cfg_attr(docsrs, doc(cfg(feature = "trace")))]
pub mod trace;

pub use opentelemetry::ExportError;
