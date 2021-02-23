use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use opentelemetry::{
    global,
    sdk::export::trace::stdout,
    sdk::trace as sdktrace,
    trace::{Span, Tracer},
};
use opentelemetry_aws::XrayPropagator;
use opentelemetry_http::HeaderExtractor;
use std::{convert::Infallible, net::SocketAddr};

async fn handle(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let parent_context = global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(req.headers()))
    });

    let x_amzn_trace_id = req
        .headers()
        .get("x-amzn-trace-id")
        .unwrap()
        .to_str()
        .unwrap();

    let span = global::tracer("example/server").start_with_context("hello", parent_context);
    span.add_event(format!("Handling - {}", x_amzn_trace_id), Vec::new());

    Ok(Response::new(
        format!("Hello!, X-Ray Trace Header: {}", x_amzn_trace_id).into(),
    ))
}

fn init_tracer() -> sdktrace::Tracer {
    global::set_text_map_propagator(XrayPropagator::new());

    // Install stdout exporter pipeline to be able to retrieve the collected spans.
    // For the demonstration, use `Sampler::AlwaysOn` sampler to sample all traces. In a production
    // application, use `Sampler::ParentBased` or `Sampler::TraceIdRatioBased` with a desired ratio.
    stdout::new_pipeline()
        .with_trace_config(
            sdktrace::config()
                .with_default_sampler(sdktrace::Sampler::AlwaysOn)
                .with_id_generator(sdktrace::XrayIdGenerator::default()),
        )
        .install()
}

#[tokio::main]
async fn main() {
    let _tracer = init_tracer();
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle)) });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on {}", addr);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
