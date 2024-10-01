#![allow(unused_macros)]
/// Macro for logging informational messages in OpenTelemetry.
///
/// # Fields:
/// - `name`: The operation or action being logged.
/// - Additional optional key-value pairs can be passed as attributes.
///
/// # Example:
/// ```rust
/// otel_info!("sdk_start", version = "1.0.0", schema_url = "http://example.com");
/// ```
#[macro_export]
macro_rules! otel_info {
    (name: $name:expr $(,)?) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::info!(name: $name, target: env!("CARGO_PKG_NAME"),"");
        }
    };
    (name: $name:expr, $($key:ident = $value:expr),+ $(,)?) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
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
/// otel_warn!("export_warning", error_code = 404, version = "1.0.0");
/// ```
#[macro_export]
macro_rules! otel_warn {
    (name: $name:expr $(,)?) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::warn!(name: $name, target: env!("CARGO_PKG_NAME"), "");
        }
    };
    (name: $name:expr, $($key:ident = $value:expr),+ $(,)?) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::warn!(name: $name, target: env!("CARGO_PKG_NAME"), $($key = $value),+, "");
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
/// otel_debug!("debug_operation", debug_level = "high", version = "1.0.0");
/// ```
#[macro_export]
macro_rules! otel_debug {
    (name: $name:expr $(,)?) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::debug!(name: $name, target: env!("CARGO_PKG_NAME"), "");
        }
    };
    (name: $name:expr, $($key:ident = $value:expr),+ $(,)?) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
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
/// otel_error!("export_failure", error_code = 500, version = "1.0.0");
/// ```
#[macro_export]
macro_rules! otel_error {
    (name: $name:expr $(,)?) => {
        {
            tracing::error!(name: $name, target: env!("CARGO_PKG_NAME"), "");
        }
    };
    (name: $name:expr, $($key:ident = $value:expr),+ $(,)?) => {
        {
            tracing::error!(name: $name, target: env!("CARGO_PKG_NAME"), $($key = $value),+, "");
        }
    };
}
