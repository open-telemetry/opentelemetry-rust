use log::{error, Level};
use once_cell::sync::Lazy;
use opentelemetry_api::global;
use opentelemetry_api::global::{
    logger_provider, shutdown_logger_provider, shutdown_tracer_provider,
};
use opentelemetry_api::logs::LogError;
use opentelemetry_api::trace::TraceError;
use opentelemetry_api::{
    metrics,
    trace::{TraceContextExt, Tracer},
    Key, KeyValue,
};
use opentelemetry_appender_log::OpenTelemetryLogBridge;
use opentelemetry_otlp::{ExportConfig, WithExportConfig};
use opentelemetry_sdk::logs::Config;
use opentelemetry_sdk::{metrics::MeterProvider, runtime, trace as sdktrace, Resource};
use std::error::Error;

fn init_tracer() -> Result<sdktrace::Tracer, TraceError> {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .with_trace_config(
            sdktrace::config().with_resource(Resource::new(vec![KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                "basic-otlp-tracing-example",
            )])),
        )
        .install_batch(runtime::Tokio)
}

fn init_metrics() -> metrics::Result<MeterProvider> {
    let export_config = ExportConfig {
        endpoint: "http://localhost:4317".to_string(),
        ..ExportConfig::default()
    };
    opentelemetry_otlp::new_pipeline()
        .metrics(runtime::Tokio)
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_export_config(export_config),
        )
        .with_resource(Resource::new(vec![KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
            "basic-otlp-metrics-example",
        )]))
        .build()
}

fn init_logs() -> Result<opentelemetry_sdk::logs::Logger, LogError> {
    opentelemetry_otlp::new_pipeline()
        .logging()
        .with_log_config(
            Config::default().with_resource(Resource::new(vec![KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                "basic-otlp-logging-example",
            )])),
        )
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .install_batch(runtime::Tokio)
}

const LEMONS_KEY: Key = Key::from_static_str("lemons");
const ANOTHER_KEY: Key = Key::from_static_str("ex.com/another");

static COMMON_ATTRIBUTES: Lazy<[KeyValue; 4]> = Lazy::new(|| {
    [
        LEMONS_KEY.i64(10),
        KeyValue::new("A", "1"),
        KeyValue::new("B", "2"),
        KeyValue::new("C", "3"),
    ]
});

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    // By binding the result to an unused variable, the lifetime of the variable
    // matches the containing block, reporting traces and metrics during the whole
    // execution.
    let _ = init_tracer()?;
    let meter_provider = init_metrics()?;

    // Initialize logs, which sets the global loggerprovider.
    let _ = init_logs();

    // Retrieve the global LoggerProvider.
    let logger_provider = logger_provider();

    // Create a new OpenTelemetryLogBridge using the above LoggerProvider.
    let otel_log_appender = OpenTelemetryLogBridge::new(&logger_provider);
    log::set_boxed_logger(Box::new(otel_log_appender)).unwrap();
    log::set_max_level(Level::Error.to_level_filter());

    let tracer = global::tracer("ex.com/basic");
    let meter = global::meter("ex.com/basic");

    let gauge = meter
        .f64_observable_gauge("ex.com.one")
        .with_description("A gauge set to 1.0")
        .init();

    meter.register_callback(&[gauge.as_any()], move |observer| {
        observer.observe_f64(&gauge, 1.0, COMMON_ATTRIBUTES.as_ref())
    })?;

    let histogram = meter.f64_histogram("ex.com.two").init();
    histogram.record(5.5, COMMON_ATTRIBUTES.as_ref());

    tracer.in_span("operation", |cx| {
        let span = cx.span();
        span.add_event(
            "Nice operation!".to_string(),
            vec![Key::new("bogons").i64(100)],
        );
        span.set_attribute(ANOTHER_KEY.string("yes"));

        error!(target: "my-target", "hello from {}. My price is {}. I am also inside a Span!", "banana", 2.99);

        tracer.in_span("Sub operation...", |cx| {
            let span = cx.span();
            span.set_attribute(LEMONS_KEY.string("five"));

            span.add_event("Sub span event", vec![]);

            histogram.record(1.3, &[]);
        });
    });

    error!(target: "my-target", "hello from {}. My price is {}", "banana", 2.99);

    shutdown_tracer_provider();
    shutdown_logger_provider();
    meter_provider.shutdown()?;

    Ok(())
}
