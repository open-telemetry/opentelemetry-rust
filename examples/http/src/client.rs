use hyper::{body::Body, Client};
use opentelemetry::api::{Context, HttpTextFormat, TraceContextExt, Tracer};
use opentelemetry::{api, exporter::trace::stdout, global, sdk};

fn init_tracer() {
    // Create stdout exporter to be able to retrieve the collected spans.
    let exporter = stdout::Builder::default().init();

    // For the demonstration, use `Sampler::AlwaysOn` sampler to sample all traces. In a production
    // application, use `Sampler::ParentBased` or `Sampler::TraceIdRatioBased` with a desired ratio.
    let provider = sdk::Provider::builder()
        .with_simple_exporter(exporter)
        .with_config(sdk::Config {
            default_sampler: Box::new(sdk::Sampler::AlwaysOn),
            ..Default::default()
        })
        .build();

    global::set_provider(provider);
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    init_tracer();

    let client = Client::new();
    let propagator = api::TraceContextPropagator::new();
    let span = global::tracer("example/client").start("say hello");
    let cx = Context::current_with_span(span);

    let mut req = hyper::Request::builder().uri("http://127.0.0.1:3000");
    propagator.inject_context(&cx, req.headers_mut().unwrap());
    let res = client.request(req.body(Body::from("Hallo!"))?).await?;

    cx.span().add_event(
        "Got response!".to_string(),
        vec![api::KeyValue::new("status", res.status().to_string())],
    );

    Ok(())
}
