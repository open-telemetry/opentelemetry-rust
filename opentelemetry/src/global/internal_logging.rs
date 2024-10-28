#![allow(unused_macros)]

#[cfg(feature = "logs")]
use crate::logs::LogError;
#[cfg(feature = "metrics")]
use crate::metrics::MetricError;
use crate::propagation::PropagationError;
#[cfg(feature = "trace")]
use crate::trace::TraceError;
use std::sync::PoisonError;

///
/// **Note**: These macros (`otel_info!`, `otel_warn!`, `otel_debug!`, and `otel_error!`) are intended to be used
/// **internally within OpenTelemetry code** or for **custom exporters and processors**. They are not designed
/// for general application logging and should not be used for that purpose.
///

/// Macro for logging informational messages in OpenTelemetry.
///
/// # Fields:
/// - `name`: The operation or action being logged.
/// - Additional optional key-value pairs can be passed as attributes.
///
/// # Example:
/// ```rust
/// use opentelemetry::otel_info;
/// otel_info!(name: "sdk_start", version = "1.0.0", schema_url = "http://example.com");
/// ```
#[macro_export]
macro_rules! otel_info {
    (name: $name:expr $(,)?) => {
        #[cfg(feature = "internal-logs")]
        {
            tracing::info!( name: $name, target: env!("CARGO_PKG_NAME"), "");
        }
        #[cfg(not(feature = "internal-logs"))]
        {
            let _ = $name; // Compiler will optimize this out as it's unused.
        }
    };
    (name: $name:expr, $($key:ident = $value:expr),+ $(,)?) => {
        #[cfg(feature = "internal-logs")]
        {
            tracing::info!(name: $name, target: env!("CARGO_PKG_NAME"), $($key = $value),+, "");
        }
        #[cfg(not(feature = "internal-logs"))]
        {
            let _ = ($name, $($value),+); // Compiler will optimize this out as it's unused.
        }
    };
}

/// Macro for logging warning messages in OpenTelemetry.
///
/// # Fields:
/// - `name`: The operation or action being logged.
/// - Additional optional key-value pairs can be passed as attributes.
///
/// # Example:
/// ```rust
/// use opentelemetry::otel_warn;
/// otel_warn!(name: "export_warning", error_code = 404, version = "1.0.0");
/// ```
#[macro_export]
macro_rules! otel_warn {
    (name: $name:expr $(,)?) => {
        #[cfg(feature = "internal-logs")]
        {
            tracing::warn!(name: $name, target: env!("CARGO_PKG_NAME"), "");
        }
        #[cfg(not(feature = "internal-logs"))]
        {
            let _ = $name; // Compiler will optimize this out as it's unused.
        }
    };
    (name: $name:expr, $($key:ident = $value:expr),+ $(,)?) => {
        #[cfg(feature = "internal-logs")]
        {
            tracing::warn!(name: $name,
                            target: env!("CARGO_PKG_NAME"),
                            $($key = {
                                    $value
                            }),+,
                            ""
                    )
        }
        #[cfg(not(feature = "internal-logs"))]
        {
            let _ = ($name, $($value),+); // Compiler will optimize this out as it's unused.
        }
    };
}

/// Macro for logging debug messages in OpenTelemetry.
///
/// # Fields:
/// - `name`: The operation or action being logged.
/// - Additional optional key-value pairs can be passed as attributes.
///
/// # Example:
/// ```rust
/// use opentelemetry::otel_debug;
/// otel_debug!(name: "debug_operation", debug_level = "high", version = "1.0.0");
/// ```
#[macro_export]
macro_rules! otel_debug {
    (name: $name:expr $(,)?) => {
        #[cfg(feature = "internal-logs")]
        {
            tracing::debug!(name: $name, target: env!("CARGO_PKG_NAME"),"");
        }
        #[cfg(not(feature = "internal-logs"))]
        {
            let _ = $name; // Compiler will optimize this out as it's unused.
        }
    };
    (name: $name:expr, $($key:ident = $value:expr),+ $(,)?) => {
        #[cfg(feature = "internal-logs")]
        {
            tracing::debug!(name: $name, target: env!("CARGO_PKG_NAME"), $($key = $value),+, "");
        }
        #[cfg(not(feature = "internal-logs"))]
        {
            let _ = ($name, $($value),+); // Compiler will optimize this out as it's unused.
        }
    };
}

/// Macro for logging error messages in OpenTelemetry.
///
/// # Fields:
/// - `name`: The operation or action being logged.
/// - Additional optional key-value pairs can be passed as attributes.
///
/// # Example:
/// ```rust
/// use opentelemetry::otel_error;
/// otel_error!(name: "export_failure", error_code = 500, version = "1.0.0");
/// ```
#[macro_export]
macro_rules! otel_error {
    (name: $name:expr $(,)?) => {
        #[cfg(feature = "internal-logs")]
        {
            tracing::error!(name: $name, target: env!("CARGO_PKG_NAME"), "");
        }
        #[cfg(not(feature = "internal-logs"))]
        {
            let _ = $name; // Compiler will optimize this out as it's unused.
        }
    };
    (name: $name:expr, $($key:ident = $value:expr),+ $(,)?) => {
        #[cfg(feature = "internal-logs")]
        {
            tracing::error!(name: $name,
                            target: env!("CARGO_PKG_NAME"),
                            $($key = {
                                    $value
                            }),+,
                            ""
                    )
        }
        #[cfg(not(feature = "internal-logs"))]
        {
            let _ = ($name, $($value),+); // Compiler will optimize this out as it's unused.
        }
    };
}

/// Wrapper for error from tracing, log, and metrics part of open telemetry.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[cfg(feature = "trace")]
    #[cfg_attr(docsrs, doc(cfg(feature = "trace")))]
    #[error(transparent)]
    /// Failed to export traces.
    Trace(#[from] TraceError),
    #[cfg(feature = "metrics")]
    #[cfg_attr(docsrs, doc(cfg(feature = "metrics")))]
    #[error(transparent)]
    /// An issue raised by the metrics module.
    Metric(#[from] MetricError),

    #[cfg(feature = "logs")]
    #[cfg_attr(docsrs, doc(cfg(feature = "logs")))]
    #[error(transparent)]
    /// Failed to export logs.
    Log(#[from] LogError),

    #[error(transparent)]
    /// Error happens when injecting and extracting information using propagators.
    Propagation(#[from] PropagationError),

    #[error("{0}")]
    /// Other types of failures not covered by the variants above.
    Other(String),
}

impl<T> From<PoisonError<T>> for Error {
    fn from(err: PoisonError<T>) -> Self {
        Error::Other(err.to_string())
    }
}
