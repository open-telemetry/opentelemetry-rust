use opentelemetry::global;
use opentelemetry::trace::Tracer;
use opentelemetry_sdk::runtime;
use opentelemetry_sdk::trace::{Sampler, TracerProvider as SdkTracerProvider};
use std::time::Duration;

fn setup() {
    let client = reqwest::Client::new();

    let sampler = Sampler::jaeger_remote(runtime::Tokio, client, Sampler::AlwaysOff, "foo")
        .with_endpoint("http://localhost:5778/sampling") // setup jaeger remote sampler endpoint
        .with_update_interval(Duration::from_secs(5)) // will call jaeger sampling endpoint every 5 secs.
        .build()
        .unwrap();

    let config = opentelemetry_sdk::trace::config().with_sampler(sampler);

    let provider = SdkTracerProvider::builder()
        .with_config(config)
        .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
        .build();

    global::set_tracer_provider(provider);
}

#[tokio::main]
async fn main() {
    setup();
    let tracer = global::tracer("test");

    {
        let _not_sampled_span = tracer.start("test");
    }

    tokio::time::sleep(Duration::from_secs(10)).await;

    {
        let _sampled_span = tracer.start("should_record");
    }
}
