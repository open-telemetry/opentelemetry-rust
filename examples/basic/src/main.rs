use futures::stream::{Stream, StreamExt};
use opentelemetry::api::metrics::{self, MetricsError, ObserverResult};
use opentelemetry::api::{Context, CorrelationContextExt, Key, KeyValue, TraceContextExt, Tracer};
use opentelemetry::exporter;
use opentelemetry::sdk::metrics::PushController;
use opentelemetry::{global, sdk};
use std::time::Duration;

fn init_tracer() -> thrift::Result<()> {
    let exporter = opentelemetry_jaeger::Exporter::builder()
        .with_agent_endpoint("127.0.0.1:6831".parse().unwrap())
        .with_process(opentelemetry_jaeger::Process {
            service_name: "trace-demo".to_string(),
            tags: vec![
                Key::new("exporter").string("jaeger"),
                Key::new("float").f64(312.23),
            ],
        })
        .init()?;

    // For the demonstration, use `Sampler::AlwaysOn` sampler to sample all traces. In a production
    // application, use `Sampler::ParentBased` or `Sampler::Probability` with a desired probability.
    let provider = sdk::Provider::builder()
        .with_simple_exporter(exporter)
        .with_config(sdk::Config {
            default_sampler: Box::new(sdk::Sampler::AlwaysOn),
            ..Default::default()
        })
        .build();
    global::set_provider(provider);

    Ok(())
}

// Skip first immediate tick from tokio, not needed for async_std.
fn delayed_interval(duration: Duration) -> impl Stream<Item = tokio::time::Instant> {
    tokio::time::interval(duration).skip(1)
}

fn init_meter() -> metrics::Result<PushController> {
    exporter::metrics::stdout(tokio::spawn, delayed_interval)
        .with_quantiles(vec![0.5, 0.9, 0.99])
        .with_formatter(|batch| {
            serde_json::to_value(batch)
                .map(|value| value.to_string())
                .map_err(|err| MetricsError::Other(err.to_string()))
        })
        .try_init()
}

lazy_static::lazy_static! {
    static ref FOO_KEY: Key = Key::new("ex.com/foo");
    static ref BAR_KEY: Key = Key::new("ex.com/bar");
    static ref LEMONS_KEY: Key = Key::new("ex.com/lemons");
    static ref ANOTHER_KEY: Key = Key::new("ex.com/another");
    static ref COMMON_LABELS: [KeyValue; 4] = [
        LEMONS_KEY.i64(10),
        KeyValue::new("A", "1"),
        KeyValue::new("B", "2"),
        KeyValue::new("C", "3"),
    ];
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracer()?;
    let _started = init_meter()?;

    let tracer = global::tracer("ex.com/basic");
    let meter = global::meter("ex.com/basic");

    let one_metric_callback = |res: ObserverResult<f64>| res.observe(1.0, COMMON_LABELS.as_ref());
    let _ = meter
        .f64_value_observer("ex.com.one", one_metric_callback)
        .with_description("A ValueObserver set to 1.0")
        .init();

    let value_recorder_two = meter.f64_value_recorder("ex.com.two").init();

    let _correlations =
        Context::current_with_correlations(vec![FOO_KEY.string("foo1"), BAR_KEY.string("bar1")])
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
            &Context::current_with_correlations(vec![ANOTHER_KEY.string("xyz")]),
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

    Ok(())
}
