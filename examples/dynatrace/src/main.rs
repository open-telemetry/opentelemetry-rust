use opentelemetry::global::shutdown_tracer_provider;
use opentelemetry::runtime;
use opentelemetry::sdk::export::metrics::aggregation::{
    AggregationKind, Temporality, TemporalitySelector,
};
use opentelemetry::sdk::metrics::aggregators::Aggregator;
use opentelemetry::sdk::metrics::controllers::BasicController;
use opentelemetry::sdk::metrics::sdk_api::Descriptor;
use opentelemetry::sdk::{export::metrics::AggregatorSelector, metrics::aggregators};
use opentelemetry::trace::TraceError;
use opentelemetry::{
    global,
    sdk::{propagation::TraceContextPropagator, trace as sdktrace, Resource},
};
use opentelemetry::{
    metrics,
    trace::{TraceContextExt, Tracer},
    Context, Key, KeyValue,
};
use opentelemetry_dynatrace::transform::DimensionSet;
use opentelemetry_dynatrace::ExportConfig;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_semantic_conventions as semcov;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

fn init_tracer() -> Result<sdktrace::Tracer, TraceError> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let mut map = HashMap::with_capacity(1);
    map.insert("Authorization".to_string(), "Api-Token *****".to_string());

    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint("https://example.live.dynatrace.com/api/v2/otlp/v1/traces")
                .with_headers(map),
        )
        // Key value pairs that will be added to all trace data
        .with_trace_config(sdktrace::config().with_resource(Resource::new(vec![
            semcov::resource::SERVICE_NAME.string("rust-quickstart"),
            semcov::resource::SERVICE_VERSION.string(env!("CARGO_PKG_VERSION")),
        ])))
        .install_batch(opentelemetry::runtime::Tokio)
}

fn init_metrics() -> metrics::Result<BasicController> {
    opentelemetry_dynatrace::new_pipeline()
        .metrics(
            CustomAggregator(),
            CustomTemporalitySelector(),
            runtime::Tokio,
        )
        .with_exporter(
            opentelemetry_dynatrace::new_exporter().with_export_config(
                ExportConfig::default()
                    .with_endpoint("https://example.live.dynatrace.com/api/v2/metrics/ingest")
                    .with_token("*****".to_string()),
            ),
        )
        // Send metric data in batches every 3 seconds
        .with_period(Duration::from_secs(3))
        .with_timeout(Duration::from_secs(10))
        // Prefix all metric data keys with a custom prefix
        .with_prefix("quickstart".to_string())
        // Key value pairs that will be added to all metric data
        .with_default_dimensions(DimensionSet::from(vec![
            KeyValue::new(semcov::resource::SERVICE_NAME, "rust-quickstart"),
            KeyValue::new(semcov::resource::SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
        ]))
        .build()
}

#[derive(Debug, Clone)]
struct CustomAggregator();

impl AggregatorSelector for CustomAggregator {
    fn aggregator_for(
        &self,
        descriptor: &Descriptor,
    ) -> Option<Arc<(dyn Aggregator + Sync + std::marker::Send + 'static)>> {
        match descriptor.name() {
            "ex.com.one" => Some(Arc::new(aggregators::last_value())),
            "ex.com.two" => Some(Arc::new(aggregators::histogram(&[0.0, 0.5, 1.0, 10.0]))),
            _ => Some(Arc::new(aggregators::sum())),
        }
    }
}

#[derive(Debug, Clone)]
struct CustomTemporalitySelector();

impl TemporalitySelector for CustomTemporalitySelector {
    fn temporality_for(&self, _descriptor: &Descriptor, _kind: &AggregationKind) -> Temporality {
        Temporality::Delta
    }
}

const LEMONS_KEY: Key = Key::from_static_str("lemons");
const ANOTHER_KEY: Key = Key::from_static_str("ex.com/another");

lazy_static::lazy_static! {
    static ref COMMON_ATTRIBUTES: [KeyValue; 4] = [
        LEMONS_KEY.i64(10),
        KeyValue::new("A", "1"),
        KeyValue::new("B", "2"),
        KeyValue::new("C", "3"),
    ];
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    // By binding the result to an unused variable, the lifetime of the variable
    // matches the containing block, reporting traces and metrics during the whole
    // execution.
    let _init_tracer = init_tracer()?;
    let metrics_controller = init_metrics()?;
    let cx = Context::new();

    let tracer = global::tracer("ex.com/basic");
    let meter = global::meter("ex.com/basic");

    let gauge = meter
        .f64_observable_gauge("ex.com.one")
        .with_description("A GaugeObserver set to 1.0")
        .init();
    meter.register_callback(move |cx| gauge.observe(cx, 1.0, COMMON_ATTRIBUTES.as_ref()))?;

    let histogram = meter.f64_histogram("ex.com.two").init();

    let another_recorder = meter.f64_histogram("ex.com.two").init();
    another_recorder.record(&cx, 5.5, COMMON_ATTRIBUTES.as_ref());

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

    // wait for 1 minutes so that we could see metrics being pushed via OTLP every 10 seconds.
    tokio::time::sleep(Duration::from_secs(60)).await;

    shutdown_tracer_provider();
    metrics_controller.stop(&cx)?;

    Ok(())
}
