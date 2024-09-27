#![allow(unused_macros)]

/// Macro for logging informational messages in OpenTelemetry.
/// The `target` is optional and defaults to `"opentelemetry"` if not provided.
///
/// # Fields:
/// - `target`: The component or module generating the log (optional, default: "opentelemetry").
/// - `name`: The operation or action being logged (e.g., "sdk_start").
/// - Additional optional key-value pairs can be passed as attributes (e.g., `version`, `schema_url`).
///
/// # Example:
/// ```rust
/// // Without extra attributes
/// otel_info!(
///     name: "sdk_start"
/// );
///
/// // With explicit target and extra attributes
/// otel_info!(
///     target: "opentelemetry-sdk",
///     name: "sdk_start",
///     version = "1.0.0",
///     schema_url = "http://example.com"
/// );
/// ```
#[macro_export]
macro_rules! otel_info {
    // Without extra attributes, default target
    (name: $name:expr) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::info!(target: "opentelemetry", name = $name);
        }
    };
    // With explicit target
    (target: $target:expr, name: $name:expr) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::info!(target: $target, name = $name);
        }
    };
    // With additional attributes, default target
    (name: $name:expr, $($key:ident = $value:expr),+ $(,)?) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::info!(target: "opentelemetry", name = $name, $($key= $value),+);
        }
    };
    // With explicit target and additional attributes
    (target: $target:expr, name: $name:expr, $($key:ident = $value:expr),+ $(,)?) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::info!(target: $target, name = $name, $($key= $value),+);
        }
    };
}

/// Macro for logging warning messages in OpenTelemetry.
/// The `target` is optional and defaults to `"opentelemetry"` if not provided.
///
/// # Fields:
/// - `target`: The component or module generating the log (optional, default: "opentelemetry").
/// - `name`: The operation or action being logged (e.g., "export_warning").
/// - Additional optional key-value pairs can be passed as attributes (e.g., `version`, `error_code`).
///
/// # Example:
/// ```rust
/// // Without extra attributes
/// otel_warn!(
///     name: "export_warning"
/// );
///
/// // With explicit target and extra attributes
/// otel_warn!(
///     target: "opentelemetry-otlp",
///     name: "export_warning",
///     error_code = 404,
///     version = "1.0.0"
/// );
/// ```
#[macro_export]
macro_rules! otel_warn {
    // Without extra attributes, default target
    (name: $name:expr) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::warn!(target: "opentelemetry", name = $name );
        }
    };
    // With explicit target
    (target: $target:expr, name: $name:expr) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::warn!(target: $target, name = $name );
        }
    };
    // With additional attributes, default target
    (name: $name:expr, $($key:ident = $value:expr),+ $(,)?) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::warn!(target: "opentelemetry", name = $name , $($key= $value),+ );
        }
    };
    // With explicit target and additional attributes
    (target: $target:expr, name: $name:expr, $($key:ident = $value:expr),+ $(,)?) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::warn!(target: $target, name = $name , $($key= $value),+ );
        }
    };
}

/// Macro for logging debug messages in OpenTelemetry.
/// The `target` is optional and defaults to `"opentelemetry"` if not provided.
///
/// # Fields:
/// - `target`: The component or module generating the log (optional, default: "opentelemetry").
/// - `name`: The operation or action being logged (e.g., "debug_operation").
/// - Additional optional key-value pairs can be passed as attributes (e.g., `version`, `debug_level`).
///
/// # Example:
/// ```rust
/// // Without extra attributes
/// otel_debug!(
///     name: "debug_operation"
/// );
///
/// // With explicit target and extra attributes
/// otel_debug!(
///     target: "opentelemetry-otlp",
///     name: "debug_operation",
///     debug_level = "high",
///     version = "1.0.0"
/// );
/// ```
#[macro_export]
macro_rules! otel_debug {
    // Without extra attributes, default target
    (name: $name:expr) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::debug!(target: "opentelemetry", name = $name );
        }
    };
    // With explicit target
    (target: $target:expr, name: $name:expr) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::debug!(target: $target, name = $name );
        }
    };
    // With additional attributes, default target
    (name: $name:expr, $($key:ident = $value:expr),+ $(,)?) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::debug!(target: "opentelemetry", name = $name , $($key = $value),+ );
        }
    };
    // With explicit target and additional attributes
    (target: $target:expr, name: $name:expr, $($key:ident = $value:expr),+ $(,)?) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::debug!(target: $target, name = $name , $($key = $value),+ );
        }
    };
}

/// Macro for logging error messages in OpenTelemetry.
/// The `target` is optional and defaults to `"opentelemetry"` if not provided.
///
/// # Fields:
/// - `target`: The component or module generating the log (optional, default: "opentelemetry").
/// - `name`: The operation or action being logged (e.g., "export_failure").
/// - Additional optional key-value pairs can be passed as attributes (e.g., `error_code`, `version`).
///
/// # Example:
/// ```rust
/// // Without extra attributes
/// otel_error!(
///     name: "export_failure"
/// );
///
/// // With explicit target and extra attributes
/// otel_error!(
///     target: "opentelemetry-otlp",
///     name: "export_failure",
///     error_code = 500,
///     version = "1.0.0"
/// );
/// ```
#[macro_export]
macro_rules! otel_error {
    // Without extra attributes, default target
    (name: $name:expr) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::error!(target: "opentelemetry", name = $name );
        }
    };
    // With explicit target
    (target: $target:expr, name: $name:expr) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::error!(target: $target, name = $name);
        }
    };
    // With additional attributes, default target
    (name: $name:expr, $($key:ident = $value:expr),+ $(,)?) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::error!(target: "opentelemetry", name = $name , $($key= $value),+ );
        }
    };
    // With explicit target and additional attributes
    (target: $target:expr, name: $name:expr, $($key:ident = $value:expr),+ $(,)?) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::error!(target: $target, name = $name, $($key= $value),+ );
        }
    };
}
