//! Telemetry Export

#[cfg(feature = "logs")]
#[cfg_attr(docsrs, doc(cfg(feature = "logs")))]
pub mod logs;

#[cfg(feature = "trace")]
#[cfg_attr(docsrs, doc(cfg(feature = "trace")))]
pub mod trace;

/// Trait for errors returned by exporters
pub trait ExportError: std::error::Error + Send + Sync + 'static {
    /// The name of exporter that returned this error
    fn exporter_name(&self) -> &'static str;
}
