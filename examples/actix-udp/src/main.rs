use actix_service::Service;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use opentelemetry::trace::TraceError;
use opentelemetry::{global, sdk::trace as sdktrace};
use opentelemetry::{
    trace::{FutureExt, TraceContextExt, Tracer},
    Key,
};

fn init_tracer() -> Result<sdktrace::Tracer, TraceError> {
    opentelemetry_jaeger::new_pipeline()
        .with_agent_endpoint("localhost:6831")
        .with_service_name("trace-udp-demo")
        .with_trace_config(opentelemetry::sdk::trace::config().with_resource(
            opentelemetry::sdk::Resource::new(vec![
                opentelemetry::KeyValue::new("service.name", "my-service"), // this will not override the trace-udp-demo
                opentelemetry::KeyValue::new("service.namespace", "my-namespace"),
                opentelemetry::KeyValue::new("exporter", "jaeger"),
            ]),
        ))
        .install_simple()
}

async fn index() -> &'static str {
    let tracer = global::tracer("request");
    tracer.in_span("index", |ctx| {
        ctx.span().set_attribute(Key::new("parameter").i64(10));
        "Index"
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let _tracer = init_tracer().expect("Failed to initialise tracer.");

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap_fn(|req, srv| {
                let tracer = global::tracer("request");
                tracer.in_span("middleware", move |cx| {
                    cx.span()
                        .set_attribute(Key::new("path").string(req.path().to_string()));
                    srv.call(req).with_context(cx)
                })
            })
            .route("/", web::get().to(index))
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .run()
    .await
}
