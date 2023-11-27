use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use opentelemetry::{
    global,
    trace::{SpanKind, TraceContextExt, Tracer},
    Context,
};
use opentelemetry_http::HeaderExtractor;
use opentelemetry_sdk::{propagation::TraceContextPropagator, trace::TracerProvider};
use opentelemetry_stdout::SpanExporter;
use std::{convert::Infallible, net::SocketAddr};

async fn handle(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let parent_cx = global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(req.headers()))
    });
    let _cx_guard = parent_cx.attach();

    let tracer = global::tracer("example/server");
    let span = tracer
        .span_builder("say hello")
        .with_kind(SpanKind::Server)
        .start(&tracer);

    let cx = Context::current_with_span(span);
    cx.span().add_event("handling this...", Vec::new());
    let res = Response::new("Hello, World!".into());
    Ok(res)
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
