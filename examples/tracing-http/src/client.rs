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

async fn send_request(
    client: &Client,
    url: &str,
    body_content: &str,
    span_name: &str,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let tracer = global::tracer("example/client");
    let span = tracer
        .span_builder(span_name)
        .with_kind(SpanKind::Client)
        .start(tracer);
    let cx = Context::current_with_span(span);

    let mut req = hyper::Request::builder().uri(url);
    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(
            &cx,
            &mut HeaderInjector(req.headers_mut().unwrap()),
        )
    });
    let res = client
        .request(req.body(Body::from(body_content))?)
        .await?;

    let response_propagator: &dyn TextMapPropagator = &TraceContextResponsePropagator::new();

    let response_cx = response_propagator
        .extract_with_context(&cx, &HeaderExtractor(res.headers()));

    let response_span = response_cx.span();

    cx.span().add_event(
        "Got response!".to_string(),
        vec![
            KeyValue::new("status", res.status().to_string()),
            KeyValue::new(
                "traceresponse",
                res.headers()
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

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    init_tracer();

    let client = Client::new();
    send_request(&client, &tracer, "http://127.0.0.1:3000/health", "Health Request!", "server_health_check").await?;
    send_request(&client, &tracer, "http://127.0.0.1:3000/echo", "Echo Request!", "server_echo_check").await?;

    Ok(())
}
