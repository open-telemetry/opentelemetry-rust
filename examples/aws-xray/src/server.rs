use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use opentelemetry::{
    api::{Span, Tracer},
    exporter::trace::stdout,
    global, sdk,
};
use opentelemetry_contrib::{XrayIdGenerator, XrayTraceContextPropagator};
use std::{convert::Infallible, net::SocketAddr};

async fn handle(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let parent_context =
        global::get_text_map_propagator(|propagator| propagator.extract(req.headers()));

    let x_amzn_trace_id = req
        .headers()
        .get("x-amzn-trace-id")
        .unwrap()
        .to_str()
        .unwrap();

    let span = global::tracer("example/server").start_from_context("hello", &parent_context);
    span.add_event(format!("Handling - {}", x_amzn_trace_id), Vec::new());

    Ok(Response::new(
        format!("Hello!, X-Ray Trace Header: {}", x_amzn_trace_id).into(),
    ))
}

fn init_tracer() {
    // Create stdout exporter to be able to retrieve the collected spans.
    let exporter = stdout::Builder::default().init();

    // For the demonstration, use `Sampler::AlwaysOn` sampler to sample all traces. In a production
    // application, use `Sampler::ParentBased` or `Sampler::TraceIdRatioBased` with a desired ratio.
    let provider = sdk::Provider::builder()
        .with_simple_exporter(exporter)
        .with_config(sdk::Config {
            default_sampler: Box::new(sdk::Sampler::AlwaysOn),
            id_generator: Box::new(XrayIdGenerator::default()),
            ..Default::default()
        })
        .build();

    global::set_provider(provider);
    global::set_text_map_propagator(XrayTraceContextPropagator::new());
}

#[tokio::main]
async fn main() {
    init_tracer();
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle)) });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on {}", addr);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
