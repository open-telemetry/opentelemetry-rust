use opentelemetry::api::{Span, Tracer};
use opentelemetry::{global, sdk};
use std::thread;
use std::time::Duration;

fn init_tracer() {
    let exporter = opentelemetry_zipkin::Exporter::from_config(
        opentelemetry_zipkin::ExporterConfig::builder()
            .with_service_name("trace-demo".to_owned())
            .with_service_endpoint("127.0.0.7:9411".parse().unwrap())
            .build(),
    );

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
}

fn bar() {
    let tracer = global::tracer("component-bar");
    let span = tracer.start("bar");
    thread::sleep(Duration::from_millis(6));
    span.end()
}

fn main() {
    init_tracer();
    let tracer = global::tracer("component-main");

    tracer.in_span("foo", |_cx| {
        thread::sleep(Duration::from_millis(6));
        bar();
        thread::sleep(Duration::from_millis(6));
    });
}
