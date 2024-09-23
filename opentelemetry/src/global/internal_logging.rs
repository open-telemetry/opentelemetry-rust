#![allow(unused_macros)]

/// Macro to dynamically generate the logging `target` string with the format `crate:signal:module`.
/// The `signal` is optional.
///
/// This macro constructs a fully-qualified target string for `otel_*` logging macros by
/// concatenating the current crate name, the provided `signal`, and the module name (via `module_path!()`).
///
/// # Example:
/// ```rust
/// // If `signal` is passed
/// otel_target!("log")
/// ```
///
/// If used inside a module called `my_module::sub_module`, the resulting target string would be:
/// ```
/// "crate_name:log:my_module::sub_module"
/// ```
///
/// If no `signal` is passed:
/// ```
/// "crate_name:my_module::sub_module"
/// ```
#[macro_export]
macro_rules! otel_target {
    ($signal:expr) => {
        concat!(env!("CARGO_PKG_NAME"), ".", $signal, ".", module_path!())
    };
    () => {
        concat!(env!("CARGO_PKG_NAME"), ".", module_path!())
    };
}

/// Macro for logging informational messages in OpenTelemetry.
///
/// # Fields:
/// - `target`: The component or module generating the log (e.g., "opentelemetry-sdk").
/// - `name`: The operation or action being logged (e.g., "sdk_start").
/// - Additional optional key-value pairs can be passed as attributes (e.g., `version`, `schema_url`).
///
/// # Example:
/// ```rust
/// // Without extra attributes
/// otel_info!(
///     target: "opentelemetry-sdk",
///     name: "sdk_start"
/// );
///
/// // With extra attributes
/// otel_info!(
///     target: "opentelemetry-sdk",
///     name: "sdk_start",
///     version = "1.0.0",
///     schema_url = "http://example.com"
/// );
/// ```
#[macro_export]
macro_rules! otel_info {
    // Without extra attributes
    (target: $target:expr, name: $name:expr) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::info!(target: $target, name: $name);
        }
    };

    // With additional attributes
    (target: $target:expr, name: $name:expr, $($key:ident = $value:expr),+ $(,)?) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::info!(target: $target, name: $name, $($key: $value),+);
        }
    };
}

/// Macro for logging warning messages in OpenTelemetry.
///
/// # Fields:
/// - `target`: The component or module generating the log (e.g., "opentelemetry-otlp").
/// - `name`: The operation or action being logged (e.g., "export_warning").
/// - Additional optional key-value pairs can be passed as attributes (e.g., `version`, `error_code`).
///
/// # Example:
/// ```rust
/// // Without extra attributes
/// otel_warn!(
///     target: "opentelemetry-otlp",
///     name: "export_warning"
/// );
///
/// // With extra attributes
/// otel_warn!(
///     target: "opentelemetry-otlp",
///     name: "export_warning",
///     error_code = 404,
///     version = "1.0.0"
/// );
/// ```
#[macro_export]
macro_rules! otel_warn {
    // Without extra attributes
    (target: $target:expr, name: $name:expr) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::warn!(target: $target, name: $name );
        }
    };

    // With additional attributes
    (target: $target:expr, name: $name:expr, $($key:ident = $value:expr),+ $(,)?) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::warn!(target: $target, name: $name, $($key = $value),+ );
        }
    };
}

/// Macro for logging debug messages in OpenTelemetry.
///
/// # Fields:
/// - `target`: The component or module generating the log (e.g., "opentelemetry-otlp").
/// - `name`: The operation or action being logged (e.g., "debug_operation").
/// - Additional optional key-value pairs can be passed as attributes (e.g., `version`, `debug_level`).
///
/// # Example:
/// ```rust
/// // Without extra attributes
/// otel_debug!(
///     target: "opentelemetry-otlp",
///     name: "debug_operation"
/// );
///
/// // With extra attributes
/// otel_debug!(
///     target: "opentelemetry-otlp",
///     name: "debug_operation",
///     debug_level = "high",
///     version = "1.0.0"
/// );
/// ```
#[macro_export]
macro_rules! otel_debug {
    // Without extra attributes
    (target: $target:expr, name: $name:expr) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::debug!(target: $target, name: $name);
        }
    };

    // With additional attributes
    (target: $target:expr, name: $name:expr, $($key:ident = $value:expr),+ $(,)?) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::debug!(target: $target, name = $name, $($key = $value),+ );
        }
    };
}

/// Macro for logging error messages in OpenTelemetry.
///
/// # Fields:
/// - `target`: The component or module generating the log (e.g., "opentelemetry-otlp").
/// - `name`: The operation or action being logged (e.g., "export_failure").
/// - Additional optional key-value pairs can be passed as attributes (e.g., `error_code`, `version`).
///
/// # Example:
/// ```rust
/// // Without extra attributes
/// otel_error!(
///     target: "opentelemetry-otlp",
///     name: "export_failure"
/// );
///
/// // With extra attributes
/// otel_error!(
///     target: "opentelemetry-otlp",
///     name: "export_failure",
///     error_code = 500,
///     version = "1.0.0"
/// );
/// ```
#[macro_export]
macro_rules! otel_error {
    // Without extra attributes
    (target: $target:expr, name: $name:expr) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::error!(target: $target, name: $name);
        }
    };

    // With additional attributes
    (target: $target:expr, name: $name:expr, $($key:ident = $value:expr),+ $(,)?) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::error!(target: $target, name: $name, $($key = $value),+ );
        }
    };
}
