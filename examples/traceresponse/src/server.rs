use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use opentelemetry::propagation::TextMapPropagator;
use opentelemetry::trace::FutureExt;
use opentelemetry::{
    global,
    sdk::export::trace::stdout,
    sdk::{
        propagation::TraceContextPropagator,
        trace::{self, Sampler},
    },
    trace::{Span, Tracer},
};
use opentelemetry_contrib::trace::propagator::trace_context_response::TraceContextResponsePropagator;
use opentelemetry_http::{HeaderExtractor, HeaderInjector};
use std::{convert::Infallible, net::SocketAddr};

async fn handle(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let parent_cx = global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(req.headers()))
    });
    let _cx_guard = parent_cx.attach();
    let mut span = global::tracer("example/server").start("hello");
    span.add_event("handling this...", Vec::new());

    let mut res = Response::new("Hello, World!".into());

    let response_propagator: &dyn TextMapPropagator = &TraceContextResponsePropagator::new();
    response_propagator.inject(&mut HeaderInjector(res.headers_mut()));

    Ok(res)
}

fn init_tracer() -> impl Tracer {
    global::set_text_map_propagator(TraceContextPropagator::new());

    // Install stdout exporter pipeline to be able to retrieve the collected spans.
    // For the demonstration, use `Sampler::AlwaysOn` sampler to sample all traces. In a production
    // application, use `Sampler::ParentBased` or `Sampler::TraceIdRatioBased` with a desired ratio.
    stdout::new_pipeline()
        .with_trace_config(trace::config().with_sampler(Sampler::AlwaysOn))
        .install_simple()
}

#[tokio::main]
async fn main() {
    let _tracer = init_tracer();
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle)) });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on {addr}");
    if let Err(e) = server.await {
        eprintln!("server error: {e}");
    }
}
