use opentelemetry_api::trace::{Tracer, TracerProvider};
use opentelemetry_sdk::export::trace::stdout::Exporter as StdoutExporter;
use opentelemetry_sdk::runtime;
use opentelemetry_sdk::trace::{Sampler, TracerProvider as SdkTracerProvider};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let client = isahc::HttpClient::builder().build().unwrap();

    let sampler = Sampler::jaeger_remote(runtime::Tokio, client, Sampler::AlwaysOff)
        .with_endpoint("http://localhost:5778/sampling?service=foo")
        .with_update_interval(Duration::from_secs(5))
        .build();

    let config = opentelemetry_sdk::trace::config().with_sampler(sampler);

    let provider = SdkTracerProvider::builder()
        .with_config(config)
        .with_simple_exporter(StdoutExporter::new(std::io::stdout(), true))
        .build();

    let tracer = provider.tracer("test");

    {
        let span = tracer.start("test");
    }

    tokio::time::sleep(Duration::from_secs(10)).await;

    {
        let span = tracer.start("should_record");
    }
}
