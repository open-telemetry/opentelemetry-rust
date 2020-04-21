use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use opentelemetry::{
    api,
    api::{HttpTextFormat, Span, Tracer},
    exporter::trace::stdout,
    global, sdk,
};
use std::{convert::Infallible, net::SocketAddr};

struct HttpHeaderMapCarrier<'a>(&'a hyper::header::HeaderMap);
impl<'a> api::Carrier for HttpHeaderMapCarrier<'a> {
    fn get(&self, key: &'static str) -> Option<&str> {
        self.0.get(key).and_then(|value| value.to_str().ok())
    }

    fn set(&mut self, _key: &'static str, _value: String) {
        unimplemented!()
    }
}

async fn handle(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let propagator = api::TraceContextPropagator::new();
    let parent_context = propagator.extract(&HttpHeaderMapCarrier(req.headers()));
    let span = global::tracer("example/server").start("hello", Some(parent_context));
    span.add_event("handling this...".to_string(), Vec::new());

    Ok(Response::new("Hello, World!".into()))
}

fn init_tracer() {
    // Create stdout exporter to be able to retrieve the collected spans.
    let exporter = stdout::Builder::default().init();

    // For the demonstration, use `Sampler::Always` sampler to sample all traces. In a production
    // application, use `Sampler::Parent` or `Sampler::Probability` with a desired probability.
    let provider = sdk::Provider::builder()
        .with_simple_exporter(exporter)
        .with_config(sdk::Config {
            default_sampler: Box::new(sdk::Sampler::Always),
            ..Default::default()
        })
        .build();

    global::set_provider(provider);
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
