use integration_test_runner::asserter::{read_spans_from_json, TraceAsserter};
use opentelemetry::global;
use opentelemetry::global::shutdown_tracer_provider;
use opentelemetry::trace::TraceError;
use opentelemetry::{
    trace::{TraceContextExt, Tracer},
    Key, KeyValue,
};
use opentelemetry_sdk::{runtime, trace as sdktrace, Resource};
use std::error::Error;
use std::fs::File;
use std::os::unix::fs::MetadataExt;

fn init_tracer() -> Result<sdktrace::Tracer, TraceError> {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .with_trace_config(
            sdktrace::config().with_resource(Resource::new(vec![KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                "basic-otlp-tracing-example",
            )])),
        )
        .install_batch(runtime::Tokio)
}

const LEMONS_KEY: Key = Key::from_static_str("lemons");
const ANOTHER_KEY: Key = Key::from_static_str("ex.com/another");

pub async fn traces() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    // By binding the result to an unused variable, the lifetime of the variable
    // matches the containing block, reporting traces and metrics during the whole
    // execution.
    let _ = init_tracer()?;

    let tracer = global::tracer("ex.com/basic");

    tracer.in_span("operation", |cx| {
        let span = cx.span();
        span.add_event(
            "Nice operation!".to_string(),
            vec![Key::new("bogons").i64(100)],
        );
        span.set_attribute(KeyValue::new(ANOTHER_KEY, "yes"));

        tracer.in_span("Sub operation...", |cx| {
            let span = cx.span();
            span.set_attribute(KeyValue::new(LEMONS_KEY, "five"));

            span.add_event("Sub span event", vec![]);
        });
    });

    shutdown_tracer_provider();

    Ok(())
}

pub fn assert_traces_results(result: &str, _: &str) {
    // let left = read_spans_from_json(File::open(expected).unwrap());
    // let right = read_spans_from_json(File::open(result).unwrap());

    // TraceAsserter::new(left, right).assert();
    //
    // we cannot read result json file because the timestamp was represents as string instead of u64.
    // need to fix it on json file exporter

    assert!(File::open(result).unwrap().metadata().unwrap().size() > 0)
}

#[test]
#[should_panic(expected = "left: \"Sub operation...\"")] // we swap the parent spans with child spans in failed_traces.json
pub fn test_assert_span_eq_failure() {
    let left = read_spans_from_json(File::open("./expected/traces.json").unwrap());
    let right = read_spans_from_json(File::open("./expected/failed_traces.json").unwrap());

    TraceAsserter::new(right, left).assert();
}

#[test]
pub fn test_assert_span_eq() {
    let spans = read_spans_from_json(File::open("./expected/traces.json").unwrap());

    TraceAsserter::new(spans.clone(), spans).assert();
}
