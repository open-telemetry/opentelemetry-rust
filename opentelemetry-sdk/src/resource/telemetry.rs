use crate::resource::ResourceDetector;
use crate::Resource;
use opentelemetry_api::KeyValue;
use std::time::Duration;

/// Detect the telemetry SDK information used to capture data recorded by the instrumentation libraries.
///
/// It provides:
/// - The name of the telemetry SDK(`telemetry.sdk.name`). It will be `opentelemetry` for SDK provided by opentelemetry project.
/// - The language of the telemetry SDK(`telemetry.sdk.language`). It will be `rust` for this SDK.
/// - The version of the telemetry SDK(`telemetry.sdk.version`). It will be current `opentelemetry_sdk` crate version.
///
/// Note that the `telemetry.auto.version` is not provided as of now.
///
/// See [semantic conventions](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/resource/semantic_conventions/README.md#telemetry-sdk) for details.
#[derive(Debug)]
pub struct TelemetryResourceDetector;

impl ResourceDetector for TelemetryResourceDetector {
    fn detect(&self, _timeout: Duration) -> Resource {
        Resource::new(vec![
            KeyValue::new("telemetry.sdk.name", "opentelemetry"),
            KeyValue::new("telemetry.sdk.language", "rust"),
            KeyValue::new("telemetry.sdk.version", env!("CARGO_PKG_VERSION")),
        ])
    }
}
