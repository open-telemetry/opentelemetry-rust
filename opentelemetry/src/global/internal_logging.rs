#![allow(unused_macros)]
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
    };
    (name: $name:expr, $($key:ident = $value:expr),+ $(,)?) => {
        #[cfg(feature = "internal-logs")]
        {
            tracing::info!(name: $name, target: env!("CARGO_PKG_NAME"), $($key = $value),+, "");
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
    };
    (name: $name:expr, $($key:ident = $value:expr),+ $(,)?) => {
        #[cfg(feature = "internal-logs")]
        {
            tracing::warn!(name: $name,
                            target: env!("CARGO_PKG_NAME"),
                            $($key = {
                                    $crate::format_value!($value)
                            }),+,
                            ""
                    )
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
    };
    (name: $name:expr, $($key:ident = $value:expr),+ $(,)?) => {
        #[cfg(feature = "internal-logs")]
        {
            tracing::debug!(name: $name, target: env!("CARGO_PKG_NAME"), $($key = $value),+, "");
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
    };
    (name: $name:expr, $($key:ident = $value:expr),+ $(,)?) => {
        #[cfg(feature = "internal-logs")]
        {
            tracing::error!(name: $name,
                            target: env!("CARGO_PKG_NAME"),
                            $($key = {
                                    $crate::format_value!($value)
                            }),+,
                            ""
                    )
        }
    };
}

/// Helper macro to format a value using Debug if available, falling back to Display
#[macro_export]
macro_rules! format_value {
    ($value:expr) => {{
        // Try Debug first
        let debug_result = std::fmt::format(format_args!("{:?}", $value));
        if debug_result.starts_with('<') || debug_result.contains("::") {
            // Contains module path or starts with generic angle brackets
            format!("{}", $value)
        } else {
            debug_result
        }
    }};
}
