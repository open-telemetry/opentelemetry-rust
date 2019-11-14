use opentelemetry::api::{
    Gauge, GaugeHandle, Key, Measure, MeasureHandle, Meter, MetricOptions, Provider, Span, Tracer,
    TracerGenerics,
};
use opentelemetry::{global, sdk};
use std::thread;
use std::time::Duration;

fn main() {
    let tracer = sdk::Provider::new().get_tracer("ex_com_basic");
    global::set_tracer(Box::new(tracer));
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

    global::global_tracer().with_span("operation", move || {
        let mut span = global::global_tracer().get_active_span();
        span.add_event("Nice operation!".to_string());
        span.set_attribute(another_key.string("yes"));

        gauge.set(1.0);

        meter.record_batch(
            &common_labels,
            vec![one_metric.measurement(1.0), measure_two.measurement(2.0)],
        );

        global::global_tracer().with_span("Sub operation...", move || {
            let mut span = global::global_tracer().get_active_span();
            span.set_attribute(lemons_key.string("five"));

            span.add_event("Sub span event".to_string());

            measure.record(1.3);
        });
    });

    // Allow flush
    thread::sleep(Duration::from_millis(250));
}
