use std::thread::sleep;
use std::time::Duration;
use opentelemetry_appender_tracing::layer;
use opentelemetry_sdk::logs::{LogProcessor, SdkLogRecord, SdkLoggerProvider, SimpleLogProcessor};
use opentelemetry_sdk::Resource;
use tracing::{info, error};
use tracing_subscriber::{prelude::*, EnvFilter};
use opentelemetry::InstrumentationScope;
use opentelemetry::logs::{Severity, LogRecord};
use opentelemetry_sdk::error::OTelSdkResult;

fn main() {
    let exporter = opentelemetry_stdout::LogExporter::default();
    let filtering_processor = FilteringLogProcessor::new(
        SimpleLogProcessor::new(exporter),
        Severity::Error,
    );
    let provider: SdkLoggerProvider = SdkLoggerProvider::builder()
        .with_resource(
            Resource::builder()
                .with_service_name("log-appender-tracing-example")
                .build(),
        )
        .with_log_processor(SlowEnrichmentLogProcessor::new(filtering_processor))
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
    let filter_fmt = EnvFilter::new("info").add_directive("opentelemetry=debug".parse().unwrap());
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_thread_names(true)
        .with_filter(filter_fmt);

    tracing_subscriber::registry()
        .with(otel_layer)
        .with(fmt_layer)
        .init();

    for i in 0..10 {
        info!(name: "my-event-name", target: "my-system", event_id = 20 + i, user_name = "otel", user_email = "otel@opentelemetry.io", message = "This is an example message");
    }
    error!(name: "my-event-name", target: "my-system", event_id = 50, user_name = "otel", user_email = "otel@opentelemetry.io", message = "This is an example message");
    let _ = provider.shutdown();
}

#[derive(Debug)]
pub struct FilteringLogProcessor<P: LogProcessor> {
    delegate: P,
    min_severity: Severity,
}

impl<P: LogProcessor> FilteringLogProcessor<P> {
    pub fn new(delegate: P, min_severity: Severity) -> FilteringLogProcessor<P> {
        FilteringLogProcessor {
            delegate: delegate,
            min_severity: min_severity,
        }
    }
}

impl<P: LogProcessor> LogProcessor for FilteringLogProcessor<P> {
    fn emit(&self, data: &mut SdkLogRecord, instrumentation: &InstrumentationScope) {
        self.delegate.emit(data, instrumentation)
    }

    fn force_flush(&self) -> OTelSdkResult {
        self.delegate.force_flush()
    }

    #[cfg(feature = "spec_unstable_logs_enabled")]
    fn event_enabled(&self, level: Severity, target: &str, name: Option<&str>) -> bool {
        self.delegate.event_enabled(level, target, name) && level >= self.min_severity
    }
}

#[derive(Debug)]
pub struct SlowEnrichmentLogProcessor<P: LogProcessor> {
    /// The wrapped processor that will receive enriched log records
    delegate: P,
}

impl<P: LogProcessor> SlowEnrichmentLogProcessor<P> {
    pub fn new(delegate: P) -> SlowEnrichmentLogProcessor<P> {
        SlowEnrichmentLogProcessor {
            delegate: delegate,
        }
    }
}

impl<P: LogProcessor> LogProcessor for SlowEnrichmentLogProcessor<P> {
    fn emit(&self, data: &mut SdkLogRecord, instrumentation: &InstrumentationScope) {
        // Simulate an expensive enrichment step (e.g., fetching from a DB or service)
        sleep(Duration::from_secs(1));
        // Enrich the log record with a custom attribute using the public API
        data.add_attribute("enriched", true);
        self.delegate.emit(data, instrumentation);
    }

    fn force_flush(&self) -> OTelSdkResult {
        self.delegate.force_flush()
    }

    #[cfg(feature = "spec_unstable_logs_enabled")]
    fn event_enabled(&self, level: Severity, target: &str, name: Option<&str>) -> bool {
        // It is important to call the delegate's event_enabled method to ensure that
        // any filtering or logic implemented by downstream processors is respected.
        // Skipping this call could result in logs being emitted that should have been filtered out
        // or in bypassing other custom logic in the processor chain.
        self.delegate.event_enabled(level, target, name)
    }
}
