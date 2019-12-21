use opentelemetry::api::{Key, Provider, Span, TracerGenerics};
use opentelemetry::{global, sdk};

use actix_service::Service;
use actix_web::{web, App, HttpServer};
use futures::future::Future;

fn init_tracer() -> thrift::Result<()> {
    let exporter = opentelemetry_jaeger::Exporter::builder()
        .with_agent_endpoint("127.0.0.1:6831".parse().unwrap())
        .with_process(opentelemetry_jaeger::Process {
            service_name: "trace-demo".to_string(),
            tags: vec![
                Key::new("exporter").string("jaeger"),
                Key::new("float").f64(312.23),
            ],
        })
        .init()?;
    let provider = sdk::Provider::builder()
        .with_simple_exporter(exporter)
        .with_config(sdk::Config {
            default_sampler: Box::new(sdk::Sampler::Always),
            ..Default::default()
        })
        .build();
    global::set_provider(provider);

    Ok(())
}

fn index() -> &'static str {
    let tracer = global::trace_provider().get_tracer("request");

    tracer.with_span("index", move |span| {
        span.set_attribute(Key::new("parameter").i64(10));
        "Index"
    })
}

fn main() -> thrift::Result<()> {
    init_tracer()?;

    HttpServer::new(|| {
        App::new()
            .wrap_fn(|req, srv| {
                let tracer = global::trace_provider().get_tracer("request");
                tracer.with_span("middleware", move |span| {
                    span.set_attribute(Key::new("path").string(req.path()));
                    srv.call(req).map(|res| res)
                })
            })
            .route("/", web::get().to(index))
    })
    .bind("127.0.0.1:8088")
    .unwrap()
    .run()?;

    Ok(())
}
