use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::{body::Incoming, service::service_fn, Request, Response, StatusCode};
use hyper_util::rt::{TokioExecutor, TokioIo};
use opentelemetry::{
    global,
    trace::{FutureExt, Span, SpanKind, TraceContextExt, Tracer},
    Context, KeyValue,
};
use opentelemetry_http::{Bytes, HeaderExtractor};
use opentelemetry_sdk::{propagation::TraceContextPropagator, trace::SdkTracerProvider};
use opentelemetry_semantic_conventions::trace;
use opentelemetry_stdout::SpanExporter;
use std::{convert::Infallible, net::SocketAddr};
use tokio::net::TcpListener;

// Utility function to extract the context from the incoming request headers
fn extract_context_from_request(req: &Request<Incoming>) -> Context {
    global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(req.headers()))
    })
}

// Separate async function for the handle endpoint
async fn handle_health_check(
    _req: Request<Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, Infallible> {
    let tracer = global::tracer("example/server");
    let mut span = tracer
        .span_builder("health_check")
        .with_kind(SpanKind::Internal)
        .start(&tracer);
    span.add_event("Health check accessed", vec![]);

    let res = Response::new(
        Full::new(Bytes::from_static(b"Server is up and running!"))
            .map_err(|err| match err {})
            .boxed(),
    );

    Ok(res)
}

// Separate async function for the echo endpoint
async fn handle_echo(
    req: Request<Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, Infallible> {
    let tracer = global::tracer("example/server");
    let mut span = tracer
        .span_builder("echo")
        .with_kind(SpanKind::Internal)
        .start(&tracer);
    span.add_event("Echoing back the request", vec![]);

    let res = Response::new(req.into_body().boxed());

    Ok(res)
}

async fn router(
    req: Request<Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, Infallible> {
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
                cx.span()
                    .set_attribute(KeyValue::new(trace::HTTP_RESPONSE_STATUS_CODE, 404));
                let mut not_found = Response::new(BoxBody::default());
                *not_found.status_mut() = StatusCode::NOT_FOUND;
                Ok(not_found)
            }
        }
    };

    response
}

fn init_tracer() {
    global::set_text_map_propagator(TraceContextPropagator::new());

    // Setup tracerprovider with stdout exporter
    // that prints the spans to stdout.
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(SpanExporter::default())
        .build();

    global::set_tracer_provider(provider);
}

#[tokio::main]
async fn main() {
    use hyper_util::server::conn::auto::Builder;

    init_tracer();
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await.unwrap();

    while let Ok((stream, _addr)) = listener.accept().await {
        if let Err(err) = Builder::new(TokioExecutor::new())
            .serve_connection(TokioIo::new(stream), service_fn(router))
            .await
        {
            eprintln!("{err}");
        }
    }
}
