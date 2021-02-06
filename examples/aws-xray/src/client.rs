use hyper::{body::Body, Client};
use opentelemetry::{
    global,
    sdk::export::trace::stdout,
    sdk::trace as sdktrace,
    trace::{TraceContextExt, Tracer},
    Context, KeyValue,
};
use opentelemetry_aws::XrayPropagator;
use opentelemetry_http::HeaderInjector;

fn init_tracer() -> (sdktrace::Tracer, stdout::Uninstall) {
    global::set_text_map_propagator(XrayPropagator::new());

    // Install stdout exporter pipeline to be able to retrieve the collected spans.
    // For the demonstration, use `Sampler::AlwaysOn` sampler to sample all traces. In a production
    // application, use `Sampler::ParentBased` or `Sampler::TraceIdRatioBased` with a desired ratio.
    stdout::new_pipeline()
        .with_trace_config(sdktrace::Config {
            default_sampler: Box::new(sdktrace::Sampler::AlwaysOn),
            id_generator: Box::new(sdktrace::XrayIdGenerator::default()),
            ..Default::default()
        })
        .install()
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let (_tracer, _guard) = init_tracer();

    let client = Client::new();
    let span = global::tracer("example/client").start("say hello");
    let cx = Context::current_with_span(span);

    let mut req = hyper::Request::builder().uri("http://127.0.0.1:3000");

    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&cx, &mut HeaderInjector(req.headers_mut().unwrap()));

        println!("Headers: {:?}", req.headers_ref());
    });

    let res = client.request(req.body(Body::from("Hallo!"))?).await?;

    cx.span().add_event(
        "Got response!".to_string(),
        vec![KeyValue::new("status", res.status().to_string())],
    );

    Ok(())
}
