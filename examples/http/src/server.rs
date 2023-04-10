use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use opentelemetry::sdk::trace::TracerProvider;
use opentelemetry::{
    global,
    sdk::propagation::TraceContextPropagator,
    trace::{Span, Tracer},
};
use opentelemetry_http::HeaderExtractor;
use opentelemetry_stdout::SpanExporter;
use std::{convert::Infallible, net::SocketAddr};

async fn handle(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let parent_cx = global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(req.headers()))
    });
    let mut span = global::tracer("example/server").start_with_context("hello", &parent_cx);
    span.add_event("handling this...", Vec::new());

    Ok(Response::new("Hello, World!".into()))
}

fn init_tracer() {
    global::set_text_map_propagator(TraceContextPropagator::new());

    // Install stdout exporter pipeline to be able to retrieve the collected spans.
    // For the demonstration, use `Sampler::AlwaysOn` sampler to sample all traces. In a production
    // application, use `Sampler::ParentBased` or `Sampler::TraceIdRatioBased` with a desired ratio.
    let provider = TracerProvider::builder()
        .with_simple_exporter(SpanExporter::default())
        .build();

    global::set_tracer_provider(provider);
}

#[tokio::main]
async fn main() {
    init_tracer();
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle)) });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on {addr}");
    if let Err(e) = server.await {
        eprintln!("server error: {e}");
    }
}
