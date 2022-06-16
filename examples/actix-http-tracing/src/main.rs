use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use actix_web_opentelemetry::RequestTracing;
use opentelemetry::{global, sdk::propagation::TraceContextPropagator};
use std::io;
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;

async fn index(username: actix_web::web::Path<String>) -> String {
    greet_user(username.as_ref())
}

#[tracing::instrument]
fn greet_user(username: &str) -> String {
    tracing::info!("preparing to greet user");
    format!("Hello {}", username)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    // Start an (optional) otel prometheus metrics pipeline
    let metrics_exporter = opentelemetry_prometheus::exporter().init();
    let request_metrics = actix_web_opentelemetry::RequestMetrics::new(
        opentelemetry::global::meter("actix_http_tracing"),
        Some(|req: &actix_web::dev::ServiceRequest| {
            req.path() == "/metrics" && req.method() == actix_web::http::Method::GET
        }),
        Some(metrics_exporter),
    );

    // Start an otel jaeger trace pipeline
    global::set_text_map_propagator(TraceContextPropagator::new());
    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("app_name")
        .install_simple()
        .unwrap();

    // Initialize `tracing` using `opentelemetry-tracing` and configure logging
    Registry::default()
        .with(tracing_subscriber::EnvFilter::new("INFO"))
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();

    // Start actix web with otel and tracing middlewares
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(RequestTracing::new())
            .wrap(request_metrics.clone())
            .service(web::resource("/users/{username}").to(index))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
