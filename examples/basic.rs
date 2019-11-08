use opentelemetry::api::metrics::{Gauge, GaugeHandle, Measure, MeasureHandle, Meter, Options};
use opentelemetry::api::{Span, Tracer};
use opentelemetry::{global, sdk, Key};

fn main() {
    let tracer = opentelemetry::sdk::Tracer::new("ex_com_basic");
    let meter = opentelemetry::sdk::Meter::new("ex_com_basic");

    let foo_key = Key::new("ex_com_foo");
    let bar_key = Key::new("ex_com_bar");
    let lemons_key = Key::new("ex_com_lemons");
    let another_key = Key::new("ex_com_another");

    let one_metric = meter.new_f64_gauge(
        "ex_com_one",
        Options::default()
            .with_keys(vec![foo_key, bar_key, lemons_key.clone()])
            .with_description("A gauge set to 1.0"),
    );

    let measure_two = meter.new_f64_measure("ex_com_two", Options::default());

    let common_labels = meter.labels(vec![lemons_key.i64(10)]);

    let gauge = one_metric.acquire_handle(&common_labels);

    let measure = measure_two.acquire_handle(&common_labels);

    tracer.with_span(
        "operation".to_string(),
        Box::new(move |mut span: sdk::Span| {
            span.add_event("Nice operation!".to_string());
            span.set_attribute(another_key.string("yes"));

            gauge.set(1.0);

            meter.record_batch(
                &common_labels,
                vec![one_metric.measurement(1.0), measure_two.measurement(2.0)],
            );

            global::get_current_tracer().with_span(
                "Sub operation...".to_string(),
                Box::new(move |mut span| {
                    span.set_attribute(lemons_key.string("five"));

                    span.add_event("Sub span event".to_string());

                    measure.record(1.3);
                }),
            );
        }),
    );
}
