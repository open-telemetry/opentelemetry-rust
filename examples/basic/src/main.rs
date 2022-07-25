use opentelemetry::global::shutdown_tracer_provider;
use opentelemetry::metrics::MetricsError;
use opentelemetry::sdk::metrics::controllers::BasicController;
use opentelemetry::sdk::metrics::{controllers, processors, selectors};
use opentelemetry::sdk::trace::Config;
use opentelemetry::sdk::{export, trace as sdktrace, Resource};
use opentelemetry::trace::TraceError;
use opentelemetry::{
    baggage::BaggageExt,
    trace::{TraceContextExt, Tracer},
    Context, Key, KeyValue,
};
use opentelemetry::{global, runtime};
use std::error::Error;

fn init_tracer() -> Result<sdktrace::Tracer, TraceError> {
    opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name("trace-demo")
        .with_trace_config(Config::default().with_resource(Resource::new(vec![
            KeyValue::new("service.name", "new_service"),
            KeyValue::new("exporter", "otlp-jaeger"),
        ])))
        .install_batch(runtime::Tokio)
}

fn init_metrics() -> Result<BasicController, MetricsError> {
    let exporter = export::metrics::stdout().build()?;
    let pusher = controllers::basic(processors::factory(
        selectors::simple::inexpensive(),
        exporter.temporality_selector(),
    ))
    .with_exporter(exporter)
    .build();

    let cx = Context::new();
    pusher.start(&cx, runtime::Tokio)?;

    global::set_meter_provider(pusher.clone());

    Ok(pusher)
}

const FOO_KEY: Key = Key::from_static_str("ex.com/foo");
const BAR_KEY: Key = Key::from_static_str("ex.com/bar");
const LEMONS_KEY: Key = Key::from_static_str("ex.com/lemons");
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
    let _tracer = init_tracer()?;
    let controller = init_metrics()?;
    let cx = Context::new();

    let tracer = global::tracer("ex.com/basic");
    let meter = global::meter("ex.com/basic");

    let observable_counter = meter
        .u64_observable_counter("ex.com.one")
        .with_description("An observable counter set to 1.0")
        .init();

    let histogram = meter.f64_histogram("ex.com.three").init();

    let observable_gauge = meter.f64_observable_gauge("ex.com.two").init();

    let _baggage =
        Context::current_with_baggage(vec![FOO_KEY.string("foo1"), BAR_KEY.string("bar1")])
            .attach();

    tracer.in_span("operation", |cx| {
        let span = cx.span();
        span.add_event(
            "Nice operation!".to_string(),
            vec![Key::new("bogons").i64(100)],
        );
        span.set_attribute(ANOTHER_KEY.string("yes"));

        let _ = meter.register_callback(move |cx| {
            observable_counter.observe(cx, 1, &[]);
            observable_gauge.observe(cx, 2.0, &[]);
        });

        tracer.in_span("Sub operation...", |cx| {
            let span = cx.span();
            span.set_attribute(LEMONS_KEY.string("five"));

            span.add_event("Sub span event", vec![]);

            histogram.record(&cx, 1.3, &[]);
        });
    });

    shutdown_tracer_provider(); // sending remaining spans.
    controller.stop(&cx)?; // send remaining metrics.

    Ok(())
}
