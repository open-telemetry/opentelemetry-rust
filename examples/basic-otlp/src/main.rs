use opentelemetry::api::{
    Context, CorrelationContextExt, Gauge, GaugeHandle, Key, Measure, MeasureHandle, Meter,
    MetricOptions, TraceContextExt, Tracer,
};
use opentelemetry::{global, sdk};

fn init_tracer() {
    let exporter = opentelemetry_otlp::Exporter::default();

    // For the demonstration, use `Sampler::AlwaysOn` sampler to sample all traces. In a production
    // application, use `Sampler::ParentOrElse` or `Sampler::Probability` with a desired probability.
    let provider = sdk::Provider::builder()
        .with_simple_exporter(exporter)
        .with_config(sdk::Config {
            default_sampler: Box::new(sdk::Sampler::AlwaysOn),
            ..Default::default()
        })
        .build();
    global::set_provider(provider);
}

fn main() {
    init_tracer();
    let meter = sdk::Meter::new("ex_com_basic");

    let foo_key = Key::new("otlp.com/foo");
    let bar_key = Key::new("otlp.com/bar");
    let lemons_key = Key::new("otlp_com_lemons");
    let another_key = Key::new("otlp_com_another");

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

    let _correlations =
        Context::current_with_correlations(vec![foo_key.string("foo1"), bar_key.string("bar1")])
            .attach();

    global::tracer("component-main").in_span("operation", move |cx| {
        let span = cx.span();
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

        global::tracer("component-bar").in_span("Sub operation...", move |cx| {
            let span = cx.span();
            span.set_attribute(lemons_key.string("five"));

            span.add_event("Sub span event".to_string(), vec![]);

            measure.record(1.3);
        });
    });
}
