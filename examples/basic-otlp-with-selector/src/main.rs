use futures::stream::Stream;
use futures::StreamExt;
use opentelemetry::global::shutdown_tracer_provider;
use opentelemetry::sdk::export::metrics::{ExportKind, ExportKindFor};
use opentelemetry::sdk::{
    export::metrics::{Aggregator, AggregatorSelector},
    metrics::{aggregators, PushController},
};
use opentelemetry::trace::TraceError;
use opentelemetry::{
    baggage::BaggageExt,
    metrics::{self, Descriptor, ObserverResult},
    trace::{TraceContextExt, Tracer},
    Context, Key, KeyValue,
};
use opentelemetry::{global, sdk::trace as sdktrace};
use opentelemetry_otlp::ExporterConfig;
use opentelemetry_otlp::Protocol;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

fn init_tracer() -> Result<sdktrace::Tracer, TraceError> {
    opentelemetry_otlp::new_pipeline()
        .with_endpoint("http://localhost:4317")
        .with_tonic()
        .install_batch(opentelemetry::runtime::Tokio)
}

// Skip first immediate tick from tokio, not needed for async_std.
fn delayed_interval(duration: Duration) -> impl Stream<Item = tokio::time::Instant> {
    opentelemetry::util::tokio_interval_stream(duration).skip(1)
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

fn init_meter() -> metrics::Result<PushController> {
    let export_config = ExporterConfig {
        endpoint: "http://localhost:4317".to_string(),
        protocol: Protocol::Grpc,
        ..ExporterConfig::default()
    };
    opentelemetry_otlp::new_metrics_pipeline(tokio::spawn, delayed_interval)
        .with_export_config(export_config)
        .with_export_kind(CustomExportKindFor())
        .with_aggregator_selector(CustomAggregator())
        .build()
}

const FOO_KEY: Key = Key::from_static_str("ex.com/foo");
const BAR_KEY: Key = Key::from_static_str("ex.com/bar");
const LEMONS_KEY: Key = Key::from_static_str("lemons");
const ANOTHER_KEY: Key = Key::from_static_str("ex.com/another");

lazy_static::lazy_static! {
    static ref COMMON_LABELS: [KeyValue; 4] = [
        LEMONS_KEY.i64(10),
        KeyValue::new("A", "1"),
        KeyValue::new("B", "2"),
        KeyValue::new("C", "3"),
    ];
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let _ = init_tracer()?;
    let _started = init_meter()?;

    let tracer = global::tracer("ex.com/basic");
    let meter = global::meter("ex.com/basic");

    let one_metric_callback = |res: ObserverResult<f64>| res.observe(1.0, COMMON_LABELS.as_ref());
    let _ = meter
        .f64_value_observer("ex.com.one", one_metric_callback)
        .with_description("A ValueObserver set to 1.0")
        .init();

    let value_recorder_two = meter.f64_value_recorder("ex.com.two").init();

    let another_recorder = meter.f64_value_recorder("ex.com.two").init();
    another_recorder.record(5.5, COMMON_LABELS.as_ref());

    let _baggage =
        Context::current_with_baggage(vec![FOO_KEY.string("foo1"), BAR_KEY.string("bar1")])
            .attach();

    let value_recorder = value_recorder_two.bind(COMMON_LABELS.as_ref());
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
            COMMON_LABELS.as_ref(),
            vec![value_recorder_two.measurement(2.0)],
        );

        tracer.in_span("Sub operation...", |cx| {
            let span = cx.span();
            span.set_attribute(LEMONS_KEY.string("five"));

            span.add_event("Sub span event".to_string(), vec![]);

            value_recorder.record(1.3);
        });
    });

    // wait for 1 minutes so that we could see metrics being pushed via OTLP every 10 seconds.
    tokio::time::sleep(Duration::from_secs(60)).await;

    shutdown_tracer_provider();

    Ok(())
}
