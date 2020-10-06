use hyper::{body::Body, Client};
use opentelemetry::api::{Context, TextMapFormat, TraceContextExt, Tracer};
use opentelemetry::{api, exporter::trace::stdout, global, sdk};

fn init_tracer() -> (sdk::trace::Tracer, stdout::Uninstall) {
    // Install stdout exporter pipeline to be able to retrieve the collected spans.
    // For the demonstration, use `Sampler::AlwaysOn` sampler to sample all traces. In a production
    // application, use `Sampler::ParentBased` or `Sampler::TraceIdRatioBased` with a desired ratio.
    stdout::new_pipeline()
        .with_trace_config(sdk::trace::Config {
            default_sampler: Box::new(sdk::trace::Sampler::AlwaysOn),
            ..Default::default()
        })
        .install()
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _guard = init_tracer();

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
