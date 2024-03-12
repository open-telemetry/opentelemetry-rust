//! # Resource attributes module
//!
//! This module defines constants related to telemetry metadata.
//!
//! # TODO
//!
//! This file should be removed once the `opentelemetry-semantic-conventions` crate is stable.
//! Currently, these constants are needed for the operation of different parts of the sdk, but
//! once the official crate is stable, it is more efficient and maintainable to use the definitions
//! from there.

/// The operating system type.
pub(crate) const OS_TYPE: &str = "os.type";

/// All the command arguments (including the command/executable itself) as received by the process. On Linux-based systems (and some other Unixoid systems supporting procfs), can be set according to the list of null-delimited strings extracted from `proc/[pid]/cmdline`. For libc-based executables, this would be the full argv vector passed to `main`.
///
/// # Examples
///
/// - `cmd/otecol`
/// - `--config=config.yaml`
pub(crate) const PROCESS_COMMAND_ARGS: &str = "process.command_args";

/// Process identifier (PID).
///
/// # Examples
///
/// - `1234`
pub(crate) const PROCESS_PID: &str = "process.pid";

/// Logical name of the service.
///
/// MUST be the same for all instances of horizontally scaled services. If the value was not specified, SDKs MUST fallback to `unknown_service:` concatenated with [`process.executable.name`](process.md#process), e.g. `unknown_service:bash`. If `process.executable.name` is not available, the value MUST be set to `unknown_service`.
///
/// # Examples
///
/// - `shoppingcart`
pub(crate) const SERVICE_NAME: &str = "service.name";

/// The language of the telemetry SDK.
pub(crate) const TELEMETRY_SDK_LANGUAGE: &str = "telemetry.sdk.language";

/// The name of the telemetry SDK as defined above.
///
/// The OpenTelemetry SDK MUST set the `telemetry.sdk.name` attribute to `opentelemetry`.
/// If another SDK, like a fork or a vendor-provided implementation, is used, this SDK MUST set the
/// `telemetry.sdk.name` attribute to the fully-qualified class or module name of this SDK&#39;s main entry point
/// or another suitable identifier depending on the language.
/// The identifier `opentelemetry` is reserved and MUST NOT be used in this case.
/// All custom identifiers SHOULD be stable across different versions of an implementation.
///
/// # Examples
///
/// - `opentelemetry`
pub(crate) const TELEMETRY_SDK_NAME: &str = "telemetry.sdk.name";

/// The version string of the telemetry SDK.
///
/// # Examples
///
/// - `1.2.3`
pub(crate) const TELEMETRY_SDK_VERSION: &str = "telemetry.sdk.version";
