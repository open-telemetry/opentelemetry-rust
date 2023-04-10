use hyper::http::HeaderValue;
use hyper::{body::Body, Client};
use opentelemetry::global;
use opentelemetry::propagation::TextMapPropagator;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use opentelemetry::sdk::trace::TracerProvider;
use opentelemetry::trace::SpanKind;
use opentelemetry::{
    trace::{TraceContextExt, Tracer},
    Context, KeyValue,
};
use opentelemetry_contrib::trace::propagator::trace_context_response::TraceContextResponsePropagator;
use opentelemetry_http::{HeaderExtractor, HeaderInjector};
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

    let response_propagator: &dyn TextMapPropagator = &TraceContextResponsePropagator::new();

    let response_cx =
        response_propagator.extract_with_context(&cx, &HeaderExtractor(res.headers()));

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
