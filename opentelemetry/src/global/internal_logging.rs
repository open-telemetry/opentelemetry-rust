#![allow(unused_macros)]

/// Macro for logging messages at the general log level in OpenTelemetry.
///
/// This macro is used to emit log messages if the feature `experimental-internal-debugging` is enabled.
/// Under the hood, it calls the `tracing::log!` macro with the provided arguments.
///
/// # Usage
/// ```
/// otel_log!("This is a general log message");
/// ```
#[macro_export]
macro_rules! otel_log {
    ($($arg:tt)*) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::log!($($arg)*);
        }
    }
}

/// Macro for logging warning messages in OpenTelemetry.
///
/// This macro emits warning messages using `tracing::warn!` if the feature `experimental-internal-debugging` is enabled.
///
/// # Usage
/// ```
/// otel_warn!("This is a warning message");
/// ```
#[macro_export]
macro_rules! otel_warn {
    ($($arg:tt)*) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::warn!($($arg)*);
        }
    }
}

/// Macro for logging debug messages in OpenTelemetry.
///
/// This macro emits debug messages using `tracing::debug!` if the feature `experimental-internal-debugging` is enabled.
///
/// # Usage
/// ```
/// otel_debug!("This is a debug message");
/// ```
#[macro_export]
macro_rules! otel_debug {
    ($($arg:tt)*) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::debug!($($arg)*);
        }
    }
}

/// Macro for logging error messages in OpenTelemetry.
///
/// This macro emits error messages using `tracing::error!` if the feature `experimental-internal-debugging` is enabled.
///
/// # Usage
/// ```
/// otel_error!("This is an error message");
/// ```
#[macro_export]
macro_rules! otel_error {
    ($($arg:tt)*) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::error!($($arg)*);
        }
    }
}

/// Macro for logging informational messages in OpenTelemetry.
///
/// This macro emits informational messages using `tracing::info!` if the feature `experimental-internal-debugging` is enabled.
///
/// # Usage
/// ```
/// otel_info!("This is an info message");
/// ```
#[macro_export]
macro_rules! otel_info {
    ($($arg:tt)*) => {
        #[cfg(all(feature = "experimental-internal-debugging"))]
        {
            tracing::info!($($arg)*);
        }
    }
}
