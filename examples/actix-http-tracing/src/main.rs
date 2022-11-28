use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use actix_web_opentelemetry::{PrometheusMetricsHandler, RequestMetricsBuilder, RequestTracing};
use opentelemetry::sdk::export::metrics::aggregation;
use opentelemetry::sdk::metrics::{controllers, processors, selectors};
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
    let controller = controllers::basic(
        processors::factory(
            selectors::simple::histogram([1.0, 2.0, 5.0, 10.0, 20.0, 50.0]),
            aggregation::cumulative_temporality_selector(),
        )
        .with_memory(true),
    )
    .build();
    let exporter = opentelemetry_prometheus::exporter(controller).init();
    let meter = global::meter("actix_web");

    // Request metrics middleware
    let request_metrics = RequestMetricsBuilder::new().build(meter);

    // Start an otel jaeger trace pipeline
    global::set_text_map_propagator(TraceContextPropagator::new());
    let tracer = opentelemetry_jaeger::new_agent_pipeline()
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
            .route(
                "/metrics",
                web::get().to(PrometheusMetricsHandler::new(exporter.clone())),
            )
            .service(web::resource("/users/{username}").to(index))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
