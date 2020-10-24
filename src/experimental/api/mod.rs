//! ## OpenTelemetry Experimental API: What applications use and SDKs implement.

pub mod context;

#[cfg(feature = "base64_format")]
#[cfg_attr(docsrs, doc(cfg(feature = "base64_format")))]
pub use context::propagation::base64_format::Base64Format;
#[cfg(feature = "binary_propagator")]
#[cfg_attr(docsrs, doc(cfg(feature = "binary_propagator")))]
pub use context::propagation::binary_propagator::BinaryFormat;
