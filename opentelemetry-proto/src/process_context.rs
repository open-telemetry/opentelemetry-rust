//! Process context sharing via memory-mapped regions.
//!
//! Implements [OTEP-4719]: publishing SDK resource attributes to a named
//! memory mapping so external readers (e.g. the OpenTelemetry eBPF Profiler)
//! can discover them without direct integration.
//!
//! The process context is a singleton — only one may be active per process.
//!
//! This module is Linux-only. On other platforms, [`publish`] is a no-op.
//!
//! [OTEP-4719]: https://github.com/open-telemetry/oteps/pull/4719

use opentelemetry_sdk::Resource;

/// Publish the given [`Resource`] as the process context.
///
/// On Linux, this creates (or updates) a named memory mapping containing
/// a protobuf-serialized representation of the resource attributes, making
/// it discoverable by external readers such as the OpenTelemetry eBPF Profiler.
///
/// On non-Linux platforms, this is a no-op.
///
/// # Example
///
/// ```no_run
/// use opentelemetry_sdk::Resource;
///
/// let resource = Resource::builder()
///     .with_service_name("my-service")
///     .build();
///
/// opentelemetry_proto::process_context::publish(&resource);
/// ```
pub fn publish(_resource: &Resource) {
    #[cfg(target_os = "linux")]
    {
        // TODO: implement Linux process context publication
    }

    #[cfg(not(target_os = "linux"))]
    {
        // Process context sharing is only supported on Linux.
    }
}
