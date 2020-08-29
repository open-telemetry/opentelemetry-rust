#[macro_use]
extern crate lazy_static;

use hyper::{
    header::CONTENT_TYPE,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use opentelemetry::{
    api::{
        metrics::{BoundCounter, BoundValueRecorder},
        KeyValue,
    },
    global,
};
use opentelemetry_prometheus::PrometheusExporter;
use prometheus::{Encoder, TextEncoder};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::SystemTime;

lazy_static! {
    static ref HANDLER_ALL: [KeyValue; 1] = [KeyValue::new("handler", "all")];
}

async fn serve_req(
    _req: Request<Body>,
    state: Arc<AppState>,
) -> Result<Response<Body>, hyper::Error> {
    let request_start = SystemTime::now();

    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    let metric_families = state.exporter.registry().gather();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    state.http_counter.add(1);
    state.http_body_gauge.record(buffer.len() as u64);

    let response = Response::builder()
        .status(200)
        .header(CONTENT_TYPE, encoder.format_type())
        .body(Body::from(buffer))
        .unwrap();

    state
        .http_req_histogram
        .record(request_start.elapsed().map_or(0.0, |d| d.as_secs_f64()));

    Ok(response)
}

struct AppState {
    exporter: PrometheusExporter,
    http_counter: BoundCounter<'static, u64>,
    http_body_gauge: BoundValueRecorder<'static, u64>,
    http_req_histogram: BoundValueRecorder<'static, f64>,
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let exporter = opentelemetry_prometheus::exporter().init();

    let meter = global::meter("ex.com/hyper");
    let state = Arc::new(AppState {
        exporter,
        http_counter: meter
            .u64_counter("example.http_requests_total")
            .with_description("Total number of HTTP requests made.")
            .init()
            .bind(HANDLER_ALL.as_ref()),
        http_body_gauge: meter
            .u64_value_recorder("example.http_response_size_bytes")
            .with_description("The HTTP response sizes in bytes.")
            .init()
            .bind(HANDLER_ALL.as_ref()),
        http_req_histogram: meter
            .f64_value_recorder("example.http_request_duration_seconds")
            .with_description("The HTTP request latencies in seconds.")
            .init()
            .bind(HANDLER_ALL.as_ref()),
    });

    // For every connection, we must make a `Service` to handle all
    // incoming HTTP requests on said connection.
    let make_svc = make_service_fn(move |_conn| {
        let state = state.clone();
        // This is the `Service` that will handle the connection.
        // `service_fn` is a helper to convert a function that
        // returns a Response into a `Service`.
        async move { Ok::<_, Infallible>(service_fn(move |req| serve_req(req, state.clone()))) }
    });

    let addr = ([127, 0, 0, 1], 3000).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}
