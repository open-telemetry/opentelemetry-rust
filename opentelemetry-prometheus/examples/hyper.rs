use http_body_util::Full;
use hyper::{
    body::{Bytes, Incoming},
    header::CONTENT_TYPE,
    service::service_fn,
    Method, Request, Response,
};
use hyper_util::rt::{TokioExecutor, TokioIo};
use once_cell::sync::Lazy;
use opentelemetry::time::now;
use opentelemetry::{
    metrics::{Counter, Histogram, MeterProvider as _},
    KeyValue,
};
use opentelemetry_prometheus::PrometheusExporter;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

static HANDLER_ALL: Lazy<[KeyValue; 1]> = Lazy::new(|| [KeyValue::new("handler", "all")]);

async fn serve_req(
    req: Request<Incoming>,
    state: Arc<AppState>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    println!("Receiving request at path {}", req.uri());
    let request_start = now();

    state.http_counter.add(1, HANDLER_ALL.as_ref());

    let response = match (req.method(), req.uri().path()) {
        (&Method::GET, "/metrics") => {
            // Export metrics in Prometheus exposition format
            let exported = state.exporter.export().unwrap_or_else(|e| {
                eprintln!("Failed to export metrics: {}", e);
                String::from("# Error exporting metrics\n")
            });
            let buffer = exported.into_bytes();

            state
                .http_body_gauge
                .record(buffer.len() as u64, HANDLER_ALL.as_ref());

            Response::builder()
                .status(200)
                .header(CONTENT_TYPE, "text/plain; version=0.0.4")
                .body(Full::new(Bytes::from(buffer)))
                .unwrap()
        }
        (&Method::GET, "/") => Response::builder()
            .status(200)
            .body(Full::new("Hello World".into()))
            .unwrap(),
        _ => Response::builder()
            .status(404)
            .body(Full::new("Missing Page".into()))
            .unwrap(),
    };

    state.http_req_histogram.record(
        request_start.elapsed().map_or(0.0, |d| d.as_secs_f64()),
        &[],
    );
    Ok(response)
}

struct AppState {
    exporter: PrometheusExporter,
    http_counter: Counter<u64>,
    http_body_gauge: Histogram<u64>,
    http_req_histogram: Histogram<f64>,
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use hyper_util::server::conn::auto::Builder;

    let exporter = opentelemetry_prometheus::exporter().build()?;
    let provider = SdkMeterProvider::builder()
        .with_reader(exporter.clone())
        .build();

    let meter = provider.meter("hyper-example");
    let state = Arc::new(AppState {
        exporter,
        http_counter: meter
            .u64_counter("http_requests_total")
            .with_description("Total number of HTTP requests made.")
            .build(),
        http_body_gauge: meter
            .u64_histogram("example.http_response_size")
            .with_unit("By")
            .with_description("The metrics HTTP response sizes in bytes.")
            .build(),
        http_req_histogram: meter
            .f64_histogram("example.http_request_duration")
            .with_unit("ms")
            .with_description("The HTTP request latencies in milliseconds.")
            .build(),
    });

    let addr: SocketAddr = ([127, 0, 0, 1], 3000).into();
    let listener = TcpListener::bind(addr).await.unwrap();

    println!("Listening on http://{addr}");

    while let Ok((stream, _addr)) = listener.accept().await {
        if let Err(err) = Builder::new(TokioExecutor::new())
            .serve_connection(
                TokioIo::new(stream),
                service_fn(|req| serve_req(req, state.clone())),
            )
            .await
        {
            eprintln!("{err}");
        }
    }

    Ok(())
}
