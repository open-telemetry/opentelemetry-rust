use hyper::{body::Body, Client};
use opentelemetry::{
    api::{
        self,
        trace::{TraceContextExt, Tracer},
        Context,
    },
    exporter::trace::stdout,
    global,
    sdk::trace as sdktrace,
};
use opentelemetry_contrib::{XrayIdGenerator, XrayTraceContextPropagator};

fn init_tracer() -> (sdktrace::Tracer, stdout::Uninstall) {
    global::set_text_map_propagator(XrayTraceContextPropagator::new());

    // Install stdout exporter pipeline to be able to retrieve the collected spans.
    // For the demonstration, use `Sampler::AlwaysOn` sampler to sample all traces. In a production
    // application, use `Sampler::ParentBased` or `Sampler::TraceIdRatioBased` with a desired ratio.
    stdout::new_pipeline()
        .with_trace_config(sdktrace::Config {
            default_sampler: Box::new(sdktrace::Sampler::AlwaysOn),
            id_generator: Box::new(XrayIdGenerator::default()),
            ..Default::default()
        })
        .install()
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (_tracer, _guard) = init_tracer();

    let client = Client::new();
    let span = global::tracer("example/client").start("say hello");
    let cx = Context::current_with_span(span);

    let mut req = hyper::Request::builder().uri("http://127.0.0.1:3000");

    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&cx, req.headers_mut().unwrap());

        println!("Headers: {:?}", req.headers_ref());
    });

    let res = client.request(req.body(Body::from("Hallo!"))?).await?;

    cx.span().add_event(
        "Got response!".to_string(),
        vec![api::KeyValue::new("status", res.status().to_string())],
    );

    Ok(())
}
