use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use opentelemetry::{
    api::{self, Span, TextMapFormat, Tracer},
    exporter::trace::stdout,
    global, sdk,
};
use std::{convert::Infallible, net::SocketAddr};

async fn handle(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let propagator = api::TraceContextPropagator::new();
    let parent_cx = propagator.extract(req.headers());
    let span = global::tracer("example/server").start_from_context("hello", &parent_cx);
    span.add_event("handling this...".to_string(), Vec::new());

    Ok(Response::new("Hello, World!".into()))
}

fn init_tracer() -> (sdk::Tracer, stdout::Uninstall) {
    // Install stdout exporter pipeline to be able to retrieve the collected spans.

    // For the demonstration, use `Sampler::AlwaysOn` sampler to sample all traces. In a production
    // application, use `Sampler::ParentBased` or `Sampler::TraceIdRatioBased` with a desired ratio.
    stdout::new_pipeline()
        .with_trace_config(sdk::Config {
            default_sampler: Box::new(sdk::Sampler::AlwaysOn),
            ..Default::default()
        })
        .install()
}

#[tokio::main]
async fn main() {
    let _guard = init_tracer();
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle)) });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on {}", addr);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
