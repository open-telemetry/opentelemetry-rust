use opentelemetry::api::{
    Gauge, GaugeHandle, Key, Measure, MeasureHandle, Meter, MetricOptions, Span, TracerGenerics,
};
use opentelemetry::{global, sdk};

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

    // For the demonstration, use `Sampler::Always` sampler to sample all traces. In a production
    // application, use `Sampler::Parent` or `Sampler::Probability` with a desired probability.
    let provider = sdk::Provider::builder()
        .with_simple_exporter(exporter)
        .with_config(sdk::Config {
            default_sampler: Box::new(sdk::Sampler::Always),
            ..Default::default()
        })
        .build();
    global::set_provider(provider);

    Ok(())
}

fn main() -> thrift::Result<()> {
    init_tracer()?;
    let meter = sdk::Meter::new("ex_com_basic");

    let lemons_key = Key::new("ex_com_lemons");
    let another_key = Key::new("ex_com_another");

    let one_metric = meter.new_f64_gauge(
        "ex_com_one",
        MetricOptions::default()
            .with_keys(vec![lemons_key.clone()])
            .with_description("A gauge set to 1.0"),
    );

    let measure_two = meter.new_f64_measure(
        "ex_com_two",
        MetricOptions::default().with_keys(vec![lemons_key.clone()]),
    );

    let common_labels = meter.labels(vec![lemons_key.i64(10)]);

    let gauge = one_metric.acquire_handle(&common_labels);

    let measure = measure_two.acquire_handle(&common_labels);

    global::tracer("component-main").with_span("operation", move |span| {
        span.add_event(
            "Nice operation!".to_string(),
            vec![Key::new("bogons").i64(100)],
        );
        span.set_attribute(another_key.string("yes"));

        gauge.set(1.0);

        meter.record_batch(
            &common_labels,
            vec![one_metric.measurement(1.0), measure_two.measurement(2.0)],
        );

        global::tracer("component-bar").with_span("Sub operation...", move |span| {
            span.set_attribute(lemons_key.string("five"));

            span.add_event("Sub span event".to_string(), vec![]);

            measure.record(1.3);
        });
    });

    Ok(())
}
