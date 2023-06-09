#![warn(missing_debug_implementations, missing_docs)]

#[cfg(feature = "logs")]
mod logs;

#[cfg_attr(docsrs, doc(cfg(feature = "logs")))]
#[cfg(feature = "logs")]
pub use logs::*;

