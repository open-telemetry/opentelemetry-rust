use opentelemetry::{
    global,
    trace::{TraceContextExt, Tracer},
    InstrumentationScope, KeyValue,
};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::WithExportConfig;
#[cfg(any(feature = "gzip", feature = "zstd"))]
use opentelemetry_otlp::{Compression, WithHttpConfig};
use opentelemetry_otlp::{LogExporter, MetricExporter, Protocol, SpanExporter};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::{
    logs::SdkLoggerProvider, metrics::SdkMeterProvider, trace::SdkTracerProvider,
};
use std::{error::Error, sync::OnceLock};
use tracing::info;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

fn get_resource() -> Resource {
    static RESOURCE: OnceLock<Resource> = OnceLock::new();
    RESOURCE
        .get_or_init(|| {
            Resource::builder()
                .with_service_name("basic-otlp-example-grpc")
                .build()
        })
        .clone()
}

fn init_logs() -> SdkLoggerProvider {
    #[cfg(any(feature = "gzip", feature = "zstd"))]
    let mut exporter_builder = LogExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary);

    #[cfg(not(any(feature = "gzip", feature = "zstd")))]
    let exporter_builder = LogExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary);

    #[cfg(feature = "gzip")]
    {
        exporter_builder = exporter_builder.with_compression(Compression::Gzip);
        println!("Using gzip compression for logs");
    }

    #[cfg(all(feature = "zstd", not(feature = "gzip")))]
    {
        exporter_builder = exporter_builder.with_compression(Compression::Zstd);
        println!("Using zstd compression for logs");
    }

    let exporter = exporter_builder
        .build()
        .expect("Failed to create log exporter");

    SdkLoggerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(get_resource())
        .build()
}

fn init_traces() -> SdkTracerProvider {
    #[cfg(any(feature = "gzip", feature = "zstd"))]
    let mut exporter_builder = SpanExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary); //can be changed to `Protocol::HttpJson` to export in JSON format

    #[cfg(not(any(feature = "gzip", feature = "zstd")))]
    let exporter_builder = SpanExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary); //can be changed to `Protocol::HttpJson` to export in JSON format

    #[cfg(feature = "gzip")]
    {
        exporter_builder = exporter_builder.with_compression(Compression::Gzip);
        println!("Using gzip compression for traces");
    }

    #[cfg(all(feature = "zstd", not(feature = "gzip")))]
    {
        exporter_builder = exporter_builder.with_compression(Compression::Zstd);
        println!("Using zstd compression for traces");
    }

    let exporter = exporter_builder
        .build()
        .expect("Failed to create trace exporter");

    SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(get_resource())
        .build()
}

fn init_metrics() -> SdkMeterProvider {
    #[cfg(any(feature = "gzip", feature = "zstd"))]
    let mut exporter_builder = MetricExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary); //can be changed to `Protocol::HttpJson` to export in JSON format

    #[cfg(not(any(feature = "gzip", feature = "zstd")))]
    let exporter_builder = MetricExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary); //can be changed to `Protocol::HttpJson` to export in JSON format

    #[cfg(feature = "gzip")]
    {
        exporter_builder = exporter_builder.with_compression(Compression::Gzip);
        println!("Using gzip compression for metrics");
    }

    #[cfg(all(feature = "zstd", not(feature = "gzip")))]
    {
        exporter_builder = exporter_builder.with_compression(Compression::Zstd);
        println!("Using zstd compression for metrics");
    }

    let exporter = exporter_builder
        .build()
        .expect("Failed to create metric exporter");

    SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .with_resource(get_resource())
        .build()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let logger_provider = init_logs();

    // Create a new OpenTelemetryTracingBridge using the above LoggerProvider.
    let otel_layer = OpenTelemetryTracingBridge::new(&logger_provider);

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
    let otel_layer = otel_layer.with_filter(filter_otel);

    // Create a new tracing::Fmt layer to print the logs to stdout. It has a
    // default filter of `info` level and above, and `debug` and above for logs
    // from OpenTelemetry crates. The filter levels can be customized as needed.
    let filter_fmt = EnvFilter::new("info").add_directive("opentelemetry=debug".parse().unwrap());
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_thread_names(true)
        .with_filter(filter_fmt);

    // Initialize the tracing subscriber with the OpenTelemetry layer and the
    // Fmt layer.
    tracing_subscriber::registry()
        .with(otel_layer)
        .with(fmt_layer)
        .init();

    // At this point Logs (OTel Logs and Fmt Logs) are initialized, which will
    // allow internal-logs from Tracing/Metrics initializer to be captured.

    let tracer_provider = init_traces();
    // Set the global tracer provider using a clone of the tracer_provider.
    // Setting global tracer provider is required if other parts of the application
    // uses global::tracer() or global::tracer_with_version() to get a tracer.
    // Cloning simply creates a new reference to the same tracer provider. It is
    // important to hold on to the tracer_provider here, so as to invoke
    // shutdown on it when application ends.
    global::set_tracer_provider(tracer_provider.clone());

    let meter_provider = init_metrics();
    // Set the global meter provider using a clone of the meter_provider.
    // Setting global meter provider is required if other parts of the application
    // uses global::meter() or global::meter_with_version() to get a meter.
    // Cloning simply creates a new reference to the same meter provider. It is
    // important to hold on to the meter_provider here, so as to invoke
    // shutdown on it when application ends.
    global::set_meter_provider(meter_provider.clone());

    let common_scope_attributes = vec![KeyValue::new("scope-key", "scope-value")];
    let scope = InstrumentationScope::builder("basic")
        .with_version("1.0")
        .with_attributes(common_scope_attributes)
        .build();

    let tracer = global::tracer_with_scope(scope.clone());
    let meter = global::meter_with_scope(scope);

    let counter = meter
        .u64_counter("test_counter")
        .with_description("a simple counter for demo purposes.")
        .with_unit("my_unit")
        .build();
    for _ in 0..10 {
        counter.add(1, &[KeyValue::new("test_key", "test_value")]);
    }
    counter.add(1, &[KeyValue::new("test_key", "test_value")]);

    tracer.in_span("Main operation", |cx| {
        let span = cx.span();
        span.add_event(
            "Nice operation!".to_string(),
            vec![KeyValue::new("some.key", 100)],
        );
        span.set_attribute(KeyValue::new("another.key", "yes"));

        info!(target: "my-target", "hello from {}. My price is {}. I am also inside a Span!", "banana", 2.99);

        tracer.in_span("Sub operation...", |cx| {
            let span = cx.span();
            span.set_attribute(KeyValue::new("another.key", "yes"));
            span.add_event("Sub span event", vec![]);
        });
    });

    info!(target: "my-target", "hello from {}. My price is {}", "apple", 1.99);

    // Collect all shutdown errors
    let mut shutdown_errors = Vec::new();
    if let Err(e) = tracer_provider.shutdown() {
        shutdown_errors.push(format!("tracer provider: {e}"));
    }

    if let Err(e) = meter_provider.shutdown() {
        shutdown_errors.push(format!("meter provider: {e}"));
    }

    if let Err(e) = logger_provider.shutdown() {
        shutdown_errors.push(format!("logger provider: {e}"));
    }

    // Return an error if any shutdown failed
    if !shutdown_errors.is_empty() {
        return Err(format!(
            "Failed to shutdown providers:{}",
            shutdown_errors.join("\n")
        )
        .into());
    }
    Ok(())
}
