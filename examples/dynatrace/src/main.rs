use futures::stream::Stream;
use futures::StreamExt;
use opentelemetry::global::shutdown_tracer_provider;
use opentelemetry::sdk::{
    export::metrics::{Aggregator, AggregatorSelector, ExportKind, ExportKindFor},
    metrics::{aggregators, PushController},
};
use opentelemetry::trace::TraceError;
use opentelemetry::{
    baggage::BaggageExt,
    metrics::{self, Descriptor, ObserverResult},
    trace::{TraceContextExt, Tracer},
    Context, Key, KeyValue,
};
use opentelemetry::{
    global,
    sdk::{propagation::TraceContextPropagator, trace as sdktrace, Resource},
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
    map.insert(
        "Authorization".to_string(),
        format!("Api-Token {}", "*****"),
    );

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

// Skip first immediate tick from tokio, not needed for async_std.
fn delayed_interval(duration: Duration) -> impl Stream<Item = tokio::time::Instant> {
    opentelemetry::util::tokio_interval_stream(duration).skip(1)
}

fn init_meter() -> metrics::Result<PushController> {
    opentelemetry_dynatrace::new_pipeline()
        .metrics(tokio::spawn, delayed_interval)
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
        .with_export_kind(CustomExportKindFor())
        .with_aggregator_selector(CustomAggregator())
        .build()
}

#[derive(Debug)]
struct CustomAggregator();

impl AggregatorSelector for CustomAggregator {
    fn aggregator_for(
        &self,
        descriptor: &Descriptor,
    ) -> Option<Arc<(dyn Aggregator + Sync + std::marker::Send + 'static)>> {
        match descriptor.name() {
            "ex.com.one" => Some(Arc::new(aggregators::last_value())),
            "ex.com.two" => Some(Arc::new(aggregators::histogram(
                descriptor,
                &[0.0, 0.5, 1.0, 10.0],
            ))),
            _ => Some(Arc::new(aggregators::sum())),
        }
    }
}

#[derive(Debug, Clone)]
struct CustomExportKindFor();

impl ExportKindFor for CustomExportKindFor {
    fn export_kind_for(&self, _descriptor: &Descriptor) -> ExportKind {
        ExportKind::Delta
    }
}

const FOO_KEY: Key = Key::from_static_str("ex.com/foo");
const BAR_KEY: Key = Key::from_static_str("ex.com/bar");
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
    let _init_meter = init_meter()?;

    let tracer = global::tracer("ex.com/basic");
    let meter = global::meter("ex.com/basic");

    let one_metric_callback =
        |res: ObserverResult<f64>| res.observe(1.0, COMMON_ATTRIBUTES.as_ref());
    let _ = meter
        .f64_value_observer("ex.com.one", one_metric_callback)
        .with_description("A ValueObserver set to 1.0")
        .init();

    let histogram_two = meter.f64_histogram("ex.com.two").init();

    let another_recorder = meter.f64_histogram("ex.com.two").init();
    another_recorder.record(5.5, COMMON_ATTRIBUTES.as_ref());

    let _baggage =
        Context::current_with_baggage(vec![FOO_KEY.string("foo1"), BAR_KEY.string("bar1")])
            .attach();

    let histogram = histogram_two.bind(COMMON_ATTRIBUTES.as_ref());

    tracer.in_span("operation", |cx| {
        let span = cx.span();
        span.add_event(
            "Nice operation!".to_string(),
            vec![Key::new("bogons").i64(100)],
        );
        span.set_attribute(ANOTHER_KEY.string("yes"));

        meter.record_batch_with_context(
            // Note: call-site variables added as context Entries:
            &Context::current_with_baggage(vec![ANOTHER_KEY.string("xyz")]),
            COMMON_ATTRIBUTES.as_ref(),
            vec![histogram_two.measurement(2.0)],
        );

        tracer.in_span("Sub operation...", |cx| {
            let span = cx.span();
            span.set_attribute(LEMONS_KEY.string("five"));

            span.add_event("Sub span event".to_string(), vec![]);

            histogram.record(1.3);
        });
    });

    // wait for 1 minutes so that we could see metrics being pushed via OTLP every 10 seconds.
    tokio::time::sleep(Duration::from_secs(60)).await;

    shutdown_tracer_provider();

    Ok(())
}
