use std::{net::SocketAddr, sync::Arc};

use futures_util::future;
use opentelemetry::global::shutdown_tracer_provider;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

mod internal;
mod ports;

use crate::internal::metrics;
use crate::ports::{http_client, http_service};

#[tokio::main]
async fn main() {

    std::env::set_var("RUST_BACKTRACE", "1");

    /* Setup for Tracing */
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    
    /* Setup for OpenTelemetry add Tracer */
    let _ = metrics::init_tracer().unwrap();

    /* Setup for Prometheus add Http Metrics Builder */
    let metrics_handler = metrics::init_prometheus_metrics("svc_customer_api");
    let meter = metrics_handler.receive_meter();
    let http_metrics =
        metrics::HttpMetricsBuilder::new(Arc::new(metrics::HttpMetrics::new(meter.clone())));

    // ----------------------------------------------------
    // Prepare Http Client Ports and Host Services
    // ----------------------------------------------------
    let cli_routes = http_client::hello_ports(http_metrics.clone());
    let cli_host: &str = "127.0.0.1:3000";
    let cli_host: SocketAddr = cli_host
        .parse()
        .expect("Unable to parse the Socket Address for Client");
    let (_, http_client_ports) = warp::serve(cli_routes).bind_ephemeral(cli_host);
    
    // ----------------------------------------------------
    // Prepare Http Service Ports and Host Scraping Metrics
    // ----------------------------------------------------
    let svc_route = http_service::metrics_port(metrics_handler.clone());
    let svc_host: &str = "127.0.0.1:9080";
    let svc_host: SocketAddr = svc_host
        .parse()
        .expect("Unable to parse the Socket Address for Scraping");
    let (_, http_service_port) = warp::serve(svc_route).bind_ephemeral(svc_host);

    // -----------------------------------
    // Join All Resources for Microservice
    // -----------------------------------
    println!(
        "Starting API Client on: {} | Scraping Metrics on: {}",
        cli_host, svc_host
    );
    future::join(http_client_ports, http_service_port).await;

    /* Ending Tracer Provider */
    shutdown_tracer_provider();
}
