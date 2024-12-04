#![cfg(unix)]

use integration_test_runner::trace_asserter::{read_spans_from_json, TraceAsserter};
use opentelemetry::global;
use opentelemetry::trace::TraceError;
use opentelemetry::{
    trace::{TraceContextExt, Tracer},
    Key, KeyValue,
};
use opentelemetry_otlp::SpanExporter;

use opentelemetry_proto::tonic::trace::v1::TracesData;
use opentelemetry_sdk::{runtime, trace as sdktrace, Resource};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::os::unix::fs::MetadataExt;

fn init_tracer_provider() -> Result<sdktrace::TracerProvider, TraceError> {
    let exporter_builder = SpanExporter::builder();
    #[cfg(feature = "tonic-client")]
    let exporter_builder = exporter_builder.with_tonic();
    #[cfg(not(feature = "tonic-client"))]
    #[cfg(any(
        feature = "hyper-client",
        feature = "reqwest-client",
        feature = "reqwest-blocking-client"
    ))]
    let exporter_builder = exporter_builder.with_http();

    let exporter = exporter_builder.build()?;

    Ok(opentelemetry_sdk::trace::TracerProvider::builder()
        .with_batch_exporter(exporter, runtime::Tokio)
        .with_resource(Resource::new(vec![KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
            "basic-otlp-tracing-example",
        )]))
        .build())
}

const LEMONS_KEY: Key = Key::from_static_str("lemons");
const ANOTHER_KEY: Key = Key::from_static_str("ex.com/another");

pub async fn traces() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let tracer_provider = init_tracer_provider().expect("Failed to initialize tracer provider.");
    global::set_tracer_provider(tracer_provider.clone());

    let tracer = global::tracer("ex.com/basic");

    tracer.in_span("operation", |cx| {
        let span = cx.span();
        span.add_event(
            "Nice operation!".to_string(),
            vec![KeyValue::new("bogons", 100)],
        );
        span.set_attribute(KeyValue::new(ANOTHER_KEY, "yes"));

        tracer.in_span("Sub operation...", |cx| {
            let span = cx.span();
            span.set_attribute(KeyValue::new(LEMONS_KEY, "five"));

            span.add_event("Sub span event", vec![]);
        });
    });

    tracer_provider.shutdown()?;

    Ok(())
}

pub fn assert_traces_results(result: &str, expected: &str) {
    let left = read_spans_from_json(File::open(expected).unwrap());
    let right = read_spans_from_json(File::open(result).unwrap());

    TraceAsserter::new(left, right).assert();

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

#[test]
pub fn test_serde() {
    let spans = read_spans_from_json(
        File::open("./expected/traces.json").expect("Failed to read traces.json"),
    );
    let json = serde_json::to_string_pretty(&TracesData {
        resource_spans: spans,
    })
    .expect("Failed to serialize spans to json");

    // Write to file.
    let mut file = File::create("./expected/serialized_traces.json").unwrap();
    file.write_all(json.as_bytes()).unwrap();

    let left = read_spans_from_json(
        File::open("./expected/traces.json").expect("Failed to read traces.json"),
    );
    let right = read_spans_from_json(
        File::open("./expected/serialized_traces.json")
            .expect("Failed to read serialized_traces.json"),
    );

    TraceAsserter::new(left, right).assert();
}
