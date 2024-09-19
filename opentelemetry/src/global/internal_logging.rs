#![allow(unused_macros)]
/// Macro for logging messages at the general log level in OpenTelemetry.
///
/// # Fields:
/// - `target`: The component or module generating the log. This identifies which crate or system part is logging the message (e.g., "opentelemetry-sdk", "opentelemetry-otlp").
/// - `name`: The operation or action being logged. This provides context on what the system is doing at the time the log is emitted (e.g., "sdk_initialization", "exporter_start").
/// - `signal`: The type of telemetry data related to the log. This could be "trace", "metric", or "log", representing the OpenTelemetry signal.
///
/// # Example:
/// ```rust
/// otel_log!(
///     target: "opentelemetry-sdk",
///     name: "sdk_initialization",
///     signal: "trace",
///     "Initializing the OpenTelemetry SDK for traces"
/// );
/// ```
#[macro_export]
macro_rules! otel_log {
    (target: $target:expr, name: $name:expr, signal: $signal:expr, $($arg:tt)*) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::log!(target: $target, { name = $name, signal = $signal }, $($arg)*);
        }
    };
}

/// Macro for logging warning messages in OpenTelemetry.
///
/// # Fields:
/// - `target`: The component or module generating the log (e.g., "opentelemetry-sdk").
/// - `name`: The operation or action being logged (e.g., "export_warning").
/// - `signal`: The type of telemetry data related to the log ("trace", "metric", "log").
///
/// # Example:
/// ```rust
/// otel_warn!(
///     target: "opentelemetry-otlp",
///     name: "export_warning",
///     signal: "metric",
///     "Potential issue detected during metric export"
/// );
/// ```
#[macro_export]
macro_rules! otel_warn {
    (target: $target:expr, name: $name:expr, signal: $signal:expr, $($arg:tt)*) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::warn!(target: $target, { name = $name, signal = $signal }, $($arg)*);
        }
    };
}

/// Macro for logging debug messages in OpenTelemetry.
///
/// # Fields:
/// - `target`: The component or module generating the log (e.g., "opentelemetry-otlp").
/// - `name`: The operation or action being logged (e.g., "debug_operation").
/// - `signal`: The type of telemetry data ("trace", "metric", "log").
///
/// # Example:
/// ```rust
/// otel_debug!(
///     target: "opentelemetry-metrics",
///     name: "metrics_debugging",
///     signal: "metric",
///     "Debugging metric exporter"
/// );
/// ```
#[macro_export]
macro_rules! otel_debug {
    (target: $target:expr, name: $name:expr, signal: $signal:expr, $($arg:tt)*) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::debug!(target: $target, { name = $name, signal = $signal }, $($arg)*);
        }
    };
}

/// Macro for logging error messages in OpenTelemetry.
///
/// # Fields:
/// - `target`: The component or module generating the log (e.g., "opentelemetry-otlp").
/// - `name`: The operation or action being logged (e.g., "export_failure").
/// - `signal`: The type of telemetry data related to the log ("trace", "metric", "log").
///
/// # Example:
/// ```rust
/// otel_error!(
///     target: "opentelemetry-otlp",
///     name: "export_failure",
///     signal: "metric",
///     "Failed to export metric data to collector"
/// );
/// ```
#[macro_export]
macro_rules! otel_error {
    (target: $target:expr, name: $name:expr, signal: $signal:expr, $($arg:tt)*) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::error!(target: $target, { name = $name, signal = $signal }, $($arg)*);
        }
    };
}

/// Macro for logging informational messages in OpenTelemetry.
///
/// # Fields:
/// - `target`: The component or module generating the log (e.g., "opentelemetry-sdk").
/// - `name`: The operation or action being logged (e.g., "sdk_start").
/// - `signal`: The type of telemetry data related to the log ("trace", "metric", "log").
///
/// # Example:
/// ```rust
/// otel_info!(
///     target: "opentelemetry-sdk",
///     name: "sdk_start",
///     signal: "trace",
///     "Starting SDK initialization"
/// );
/// ```
#[macro_export]
macro_rules! otel_info {
    (target: $target:expr, name: $name:expr, signal: $signal:expr, $($arg:tt)*) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::info!(target: $target, { name = $name, signal = $signal }, $($arg)*);
        }
    };
}
