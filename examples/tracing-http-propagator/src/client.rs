use http_body_util::Full;
use hyper_util::{client::legacy::Client, rt::TokioExecutor};
use opentelemetry::{
    global,
    trace::{SpanKind, TraceContextExt, Tracer},
    Context, KeyValue,
};
use opentelemetry_http::{Bytes, HeaderInjector};
use opentelemetry_sdk::{propagation::TraceContextPropagator, trace::SdkTracerProvider};
use opentelemetry_stdout::SpanExporter;

fn init_tracer() {
    global::set_text_map_propagator(TraceContextPropagator::new());
    // Install stdout exporter pipeline to be able to retrieve the collected spans.
    // For the demonstration, use `Sampler::AlwaysOn` sampler to sample all traces.
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(SpanExporter::default())
        .build();

    global::set_tracer_provider(provider);
}

async fn send_request(
    url: &str,
    body_content: &str,
    span_name: &str,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let client = Client::builder(TokioExecutor::new()).build_http();
    let tracer = global::tracer("example/client");
    let span = tracer
        .span_builder(String::from(span_name))
        .with_kind(SpanKind::Client)
        .start(&tracer);
    let cx = Context::current_with_span(span);

    let mut req = hyper::Request::builder().uri(url);
    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&cx, &mut HeaderInjector(req.headers_mut().unwrap()))
    });
    let res = client
        .request(req.body(Full::new(Bytes::from(body_content.to_string())))?)
        .await?;

    cx.span().add_event(
        "Got response!",
        vec![KeyValue::new("status", res.status().to_string())],
    );

    Ok(())
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    init_tracer();

    send_request(
        "http://127.0.0.1:3000/health",
        "Health Request!",
        "server_health_check",
    )
    .await?;
    send_request(
        "http://127.0.0.1:3000/echo",
        "Echo Request!",
        "server_echo_check",
    )
    .await?;

    Ok(())
}
