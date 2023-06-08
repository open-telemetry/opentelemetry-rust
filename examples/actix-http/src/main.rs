use actix_service::Service;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use opentelemetry::global::shutdown_tracer_provider;
use opentelemetry::trace::TraceError;
use opentelemetry::{global, sdk::trace as sdktrace};
use opentelemetry::{
    trace::{FutureExt, TraceContextExt, Tracer},
    Key,
};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;

fn init_tracer() -> Result<sdktrace::Tracer, TraceError> {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .with_trace_config(sdktrace::config().with_resource(Resource::new(vec![
            opentelemetry::KeyValue::new("service.name", "trace-http-demo"),
        ])))
        .install_batch(opentelemetry::runtime::TokioCurrentThread)
}

async fn index() -> &'static str {
    let tracer = global::tracer("request");
    tracer.in_span("index", |ctx| {
        if let Some(span) = ctx.span() {
            span.set_attribute(Key::new("parameter").i64(10));
        }
        "Index"
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let tracer = init_tracer().expect("Failed to initialise tracer.");

    HttpServer::new(move || {
        let tracer = tracer.clone();
        App::new()
            .wrap(Logger::default())
            .wrap_fn(move |req, srv| {
                tracer.in_span("middleware", move |cx| {
                    if let Some(span) = cx.span() {
                        span.set_attribute(Key::new("path").string(req.path().to_string()));
                    }
                    srv.call(req).with_context(cx)
                })
            })
            .route("/", web::get().to(index))
    })
    .bind("127.0.0.1:8088")
    .unwrap()
    .run()
    .await?;

    // wait until all pending spans get exported.
    shutdown_tracer_provider();

    Ok(())
}
