use hyper::{body::Body, Client};
use opentelemetry::{
    global,
    trace::{SpanKind, TraceContextExt, Tracer},
    Context, KeyValue,
};
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
    let span = tracer
        .span_builder("say hello")
        .with_kind(SpanKind::Client)
        .start(&tracer);
    let cx = Context::current_with_span(span);

    let mut req = hyper::Request::builder().uri("http://127.0.0.1:3000");
    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&cx, &mut HeaderInjector(req.headers_mut().unwrap()))
    });
    let res = client.request(req.body(Body::from("Hello!"))?).await?;

    let response_cx = global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(res.headers()))
    });

    let response_span = response_cx.span();

    cx.span().add_event(
        "Got response!".to_string(),
        vec![
            KeyValue::new("status", res.status().to_string()),
            KeyValue::new("child_sampled", response_span.span_context().is_sampled()),
        ],
    );

    Ok(())
}
