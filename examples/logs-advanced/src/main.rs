use opentelemetry::logs::{LogRecord, Severity};
use opentelemetry::InstrumentationScope;
use opentelemetry_appender_tracing::layer;
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::logs::{LogProcessor, SdkLogRecord, SdkLoggerProvider, SimpleLogProcessor};
use opentelemetry_sdk::Resource;
use tracing::{error, info};
use tracing_subscriber::{prelude::*, EnvFilter};

fn main() {
    let exporter = opentelemetry_stdout::LogExporter::default();
    let enriching_processor = EnrichmentLogProcessor::new(SimpleLogProcessor::new(exporter));
    let provider: SdkLoggerProvider = SdkLoggerProvider::builder()
        .with_resource(
            Resource::builder()
                .with_service_name("log-appender-tracing-example")
                .build(),
        )
        .with_log_processor(enriching_processor)
        .build();

    // To prevent a telemetry-induced-telemetry loop, OpenTelemetry's own internal
    // logging is properly suppressed. However, logs emitted by external components
    // (such as reqwest, tonic, etc.) are not suppressed as they do not propagate
    // OpenTelemetry context. Until this issue is addressed
    // (https://github.com/open-telemetry/opentelemetry-rust/issues/2877),
    // filtering like this is the best way to suppress such logs.
    //
    // The filter levels are set as follows:
    // - Allow `info` level and above by default.
    // - Completely restrict logs from `hyper`, `tonic`, `h2`, and `reqwest`.
    //
    // Note: This filtering will also drop logs from these components even when
    // they are used outside of the OTLP Exporter.
    let filter_otel = EnvFilter::new("info")
        .add_directive("hyper=off".parse().unwrap())
        .add_directive("tonic=off".parse().unwrap())
        .add_directive("h2=off".parse().unwrap())
        .add_directive("reqwest=off".parse().unwrap());
    let otel_layer = layer::OpenTelemetryTracingBridge::new(&provider).with_filter(filter_otel);

    // Create a new tracing::Fmt layer to print the logs to stdout. It has a
    // default filter of `info` level and above, and `debug` and above for logs
    // from OpenTelemetry crates. The filter levels can be customized as needed.
    let filter_fmt = EnvFilter::new("error").add_directive("opentelemetry=debug".parse().unwrap());
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_thread_names(true)
        .with_filter(filter_fmt);

    tracing_subscriber::registry()
        .with(otel_layer)
        .with(fmt_layer)
        .init();

    info!(name: "my-event-name", target: "my-system", event_id = 20, user_name = "otel", user_email = "otel@opentelemetry.io", message = "This is an example message");
    error!(name: "my-event-name", target: "my-system", event_id = 50, user_name = "otel", user_email = "otel@opentelemetry.io", message = "This is an example message");
    let _ = provider.shutdown();
}

/// A log processor that enriches log records with additional attributes before
/// delegating to an underlying processor.
///
/// If this were implemented as a standalone processor in a chain (e.g.,
/// EnrichmentProcessor -> SimpleLogProcessor), the performance benefits of the
/// `event_enabled` check would be nullified. Here's why:
///
/// - The `event_enabled` method is crucial for performance - it allows processors
///   to skip expensive operations for logs that will ultimately be filtered out
/// - A standalone EnrichmentProcessor would need to implement `event_enabled`,
///   but it has no knowledge of downstream filtering logic
/// - It would have to return `true` by default, causing unnecessary enrichment
///   work even for logs that the downstream processor will discard
///
/// Because this processor wraps another, it must delegate all trait methods
/// to the underlying processor. This ensures the underlying processor receives
/// all necessary lifecycle events.
#[derive(Debug)]
pub struct EnrichmentLogProcessor<P: LogProcessor> {
    /// The wrapped processor that will receive enriched log records
    delegate: P,
}

impl<P: LogProcessor> EnrichmentLogProcessor<P> {
    pub fn new(delegate: P) -> EnrichmentLogProcessor<P> {
        EnrichmentLogProcessor { delegate }
    }
}

impl<P: LogProcessor> LogProcessor for EnrichmentLogProcessor<P> {
    fn emit(&self, data: &mut SdkLogRecord, instrumentation: &InstrumentationScope) {
        data.add_attribute("enriched", true);
        self.delegate.emit(data, instrumentation);
    }

    fn force_flush(&self) -> OTelSdkResult {
        self.delegate.force_flush()
    }

    fn shutdown_with_timeout(&self, timeout: std::time::Duration) -> OTelSdkResult {
        self.delegate.shutdown_with_timeout(timeout)
    }

    fn shutdown(&self) -> OTelSdkResult {
        self.delegate.shutdown()
    }

    fn set_resource(&mut self, resource: &Resource) {
        self.delegate.set_resource(resource);
    }

    fn event_enabled(&self, level: Severity, target: &str, name: Option<&str>) -> bool {
        self.delegate.event_enabled(level, target, name)
    }
}
