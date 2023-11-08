use hyper::http::HeaderValue;
use hyper::{body::Body, Client};
use opentelemetry::{
    global,
    propagation::TextMapPropagator,
    trace::{SpanKind, TraceContextExt, Tracer},
    Context, KeyValue,
};
use opentelemetry_contrib::trace::propagator::trace_context_response::TraceContextResponsePropagator;
use opentelemetry_http::{HeaderExtractor, HeaderInjector};
use opentelemetry_sdk::{propagation::TraceContextPropagator, trace::TracerProvider};
use opentelemetry_stdout::SpanExporter;

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
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    init_tracer();

    let client = Client::new();
    let tracer = global::tracer("example/client");

    // Send /health request
    let health_span = tracer
        .span_builder("server_health_check")
        .with_kind(SpanKind::Client)
        .start(&tracer);
    let health_cx = Context::current_with_span(health_span);

    let mut health_req = hyper::Request::builder().uri("http://127.0.0.1:3000/health");
    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(
            &health_cx,
            &mut HeaderInjector(health_req.headers_mut().unwrap()),
        )
    });
    let health_res = client
        .request(health_req.body(Body::from("Health Request!"))?)
        .await?;

    let response_propagator: &dyn TextMapPropagator = &TraceContextResponsePropagator::new();

    let response_cx = response_propagator
        .extract_with_context(&health_cx, &HeaderExtractor(health_res.headers()));

    let response_span = response_cx.span();

    health_cx.span().add_event(
        "Got response!".to_string(),
        vec![
            KeyValue::new("status", health_res.status().to_string()),
            KeyValue::new(
                "traceresponse",
                health_res
                    .headers()
                    .get("traceresponse")
                    .unwrap_or(&HeaderValue::from_static(""))
                    .to_str()
                    .unwrap()
                    .to_string(),
            ),
            KeyValue::new("child_sampled", response_span.span_context().is_sampled()),
        ],
    );

    // Send /echo request
    let echo_span = tracer
        .span_builder("server_health_check")
        .with_kind(SpanKind::Client)
        .start(&tracer);
    let echo_cx = Context::current_with_span(echo_span);

    let mut echo_req = hyper::Request::builder().uri("http://127.0.0.1:3000/health");
    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(
            &echo_cx,
            &mut HeaderInjector(echo_req.headers_mut().unwrap()),
        )
    });
    let echo_res = client
        .request(echo_req.body(Body::from("Echo Request!"))?)
        .await?;

    let response_propagator: &dyn TextMapPropagator = &TraceContextResponsePropagator::new();

    let response_cx =
        response_propagator.extract_with_context(&health_cx, &HeaderExtractor(echo_res.headers()));

    let response_span = response_cx.span();

    echo_cx.span().add_event(
        "Got response!".to_string(),
        vec![
            KeyValue::new("status", health_res.status().to_string()),
            KeyValue::new(
                "traceresponse",
                health_res
                    .headers()
                    .get("traceresponse")
                    .unwrap_or(&HeaderValue::from_static(""))
                    .to_str()
                    .unwrap()
                    .to_string(),
            ),
            KeyValue::new("child_sampled", response_span.span_context().is_sampled()),
        ],
    );

    Ok(())
}
