use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server, StatusCode,
};
use opentelemetry::{
    global,
    propagation::TextMapPropagator,
    trace::{FutureExt, Span, SpanKind, TraceContextExt, Tracer},
    Context,
};
use opentelemetry_contrib::trace::propagator::trace_context_response::TraceContextResponsePropagator;
use opentelemetry_http::{HeaderExtractor, HeaderInjector};
use opentelemetry_sdk::{propagation::TraceContextPropagator, trace::TracerProvider};
use opentelemetry_stdout::SpanExporter;
use std::{convert::Infallible, net::SocketAddr};

// Utility function to extract the context from the incoming request headers
fn extract_context_from_request(req: &Request<Body>) -> Context {
    global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(req.headers()))
    })
}

// Separate async function for the handle endpoint
async fn handle_health_check(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let tracer = global::tracer("example/server");
    let mut span = tracer
        .span_builder("health_check")
        .with_kind(SpanKind::Internal)
        .start(&tracer);
    span.add_event("Health check accessed", vec![]);
    let cx = Context::default().with_span(span);
    let mut res = Response::new(Body::from("Server is up and running!"));
    // returning context is optional - https://w3c.github.io/trace-context/#traceresponse-header
    let response_propagator: &dyn TextMapPropagator = &TraceContextResponsePropagator::new();
    response_propagator.inject_context(&cx, &mut HeaderInjector(res.headers_mut()));
    Ok(res)
}

// Separate async function for the echo endpoint
async fn handle_echo(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let tracer = global::tracer("example/server");
    let mut span = tracer
        .span_builder("echo")
        .with_kind(SpanKind::Internal)
        .start(&tracer);
    span.add_event("Echoing back the request", vec![]);
    let cx = Context::default().with_span(span);
    let mut res = Response::new(req.into_body());
    // returning context is optional - https://w3c.github.io/trace-context/#traceresponse-header
    let response_propagator: &dyn TextMapPropagator = &TraceContextResponsePropagator::new();
    response_propagator.inject_context(&cx, &mut HeaderInjector(res.headers_mut()));
    Ok(res)
}

async fn router(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    // Extract the context from the incoming request headers
    let parent_cx = extract_context_from_request(&req);
    let response = {
        // Create a span parenting the remote client span.
        let tracer = global::tracer("example/server");
        let mut span = tracer
            .span_builder("router")
            .with_kind(SpanKind::Server)
            .start_with_context(&tracer, &parent_cx);

        span.add_event("dispatching request", vec![]);

        let cx = Context::default().with_span(span);
        match (req.method(), req.uri().path()) {
            (&hyper::Method::GET, "/health") => handle_health_check(req).with_context(cx).await,
            (&hyper::Method::GET, "/echo") => handle_echo(req).with_context(cx).await,
            _ => {
                let error_status = opentelemetry::trace::Status::Error {
                    description: "Not Found".into(),
                };
                cx.span().set_status(error_status);
                let mut not_found = Response::default();
                *not_found.status_mut() = StatusCode::NOT_FOUND;
                Ok(not_found)
            }
        }
    };
    response
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

    let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(router)) });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on {addr}");
    if let Err(e) = server.await {
        eprintln!("server error: {e}");
    }
}
