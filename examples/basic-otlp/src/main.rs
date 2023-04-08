use once_cell::sync::Lazy;
use opentelemetry_api::global;
use opentelemetry_api::global::shutdown_tracer_provider;
use opentelemetry_api::trace::TraceError;
use opentelemetry_api::{
    metrics,
    trace::{TraceContextExt, Tracer},
    Context, Key, KeyValue,
};
use opentelemetry_otlp::{ExportConfig, WithExportConfig};
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
                "trace-demo",
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
        .build()
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
    let _ = init_metrics()?;
    let cx = Context::new();

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
    histogram.record(&cx, 5.5, COMMON_ATTRIBUTES.as_ref());

    tracer.in_span("operation", |cx| {
        let span = cx.span();
        span.add_event(
            "Nice operation!".to_string(),
            vec![Key::new("bogons").i64(100)],
        );
        span.set_attribute(ANOTHER_KEY.string("yes"));

        tracer.in_span("Sub operation...", |cx| {
            let span = cx.span();
            span.set_attribute(LEMONS_KEY.string("five"));

            span.add_event("Sub span event", vec![]);

            histogram.record(&cx, 1.3, &[]);
        });
    });

    shutdown_tracer_provider();

    Ok(())
}
