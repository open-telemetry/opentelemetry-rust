#![allow(unused_macros)]
///
/// **Note**: These macros (`otel_info!`, `otel_warn!`, `otel_debug!`, and `otel_error!`) are intended to be used
/// **internally within OpenTelemetry code** or for **custom exporters, processors and other plugins**. They are not designed
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
///
// TODO: Remove `name` attribute duplication in logging macros below once `tracing::Fmt` supports displaying `name`.
// See issue: https://github.com/tokio-rs/tracing/issues/2774
#[macro_export]
macro_rules! otel_info {
    (name: $name:expr $(,)?) => {
        #[cfg(feature = "internal-logs")]
        {
            tracing::info!( name: $name, target: env!("CARGO_PKG_NAME"), name = $name, "");
        }
        #[cfg(not(feature = "internal-logs"))]
        {
            let _ = $name; // Compiler will optimize this out as it's unused.
        }
    };
    (name: $name:expr, $($key:ident = $value:expr),+ $(,)?) => {
        #[cfg(feature = "internal-logs")]
        {
            tracing::info!(name: $name, target: env!("CARGO_PKG_NAME"), name = $name, $($key = $value),+, "");
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
            tracing::warn!(name: $name, target: env!("CARGO_PKG_NAME"), name = $name, "");
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
                            name = $name,
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
            tracing::debug!(name: $name, target: env!("CARGO_PKG_NAME"), name = $name, "");
        }
        #[cfg(not(feature = "internal-logs"))]
        {
            let _ = $name; // Compiler will optimize this out as it's unused.
        }
    };
    (name: $name:expr, $($key:ident = $value:expr),+ $(,)?) => {
        #[cfg(feature = "internal-logs")]
        {
            tracing::debug!(name: $name, target: env!("CARGO_PKG_NAME"), name = $name, $($key = $value),+, "");
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
            tracing::error!(name: $name, target: env!("CARGO_PKG_NAME"), name = $name, "");
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
                            name = $name,
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
