use hyper::{header::CONTENT_TYPE, rt::Future, service::service_fn_ok, Body, Response, Server};
use opentelemetry::api::metrics::{
    Counter, CounterHandle, Gauge, GaugeHandle, Measure, MeasureHandle, Meter, Options,
};
use opentelemetry::exporter::metrics::prometheus::{Encoder, TextEncoder};
use std::time::SystemTime;

fn main() {
    let addr = ([127, 0, 0, 1], 9898).into();
    println!("Listening address: {:?}", addr);
    let meter = opentelemetry::sdk::metrics::Meter::new("hyper");

    let common_key = opentelemetry::Key::new("handler");
    let common_labels = meter.labels(vec![common_key.string("all")]);

    let http_counter = meter
        .new_i64_counter(
            "example_http_requests_total",
            Options::default()
                .with_description("Total number of HTTP requests made.")
                .with_keys(vec![common_key.clone()]),
        )
        .acquire_handle(&common_labels);

    let http_req_histogram = meter
        .new_f64_measure(
            "example_http_request_duration_seconds",
            Options::default()
                .with_description("The HTTP request latencies in seconds.")
                .with_keys(vec![common_key.clone()]),
        )
        .acquire_handle(&common_labels);

    let http_body_gauge = meter
        .new_f64_gauge(
            "example_http_response_size_bytes",
            Options::default()
                .with_description("The HTTP response sizes in bytes.")
                .with_keys(vec![common_key]),
        )
        .acquire_handle(&common_labels);

    let new_service = move || {
        let encoder = TextEncoder::new();
        let http_counter = http_counter.clone();
        let http_body_gauge = http_body_gauge.clone();
        let http_req_histogram = http_req_histogram.clone();
        service_fn_ok(move |_request| {
            http_counter.add(1);
            let timer = SystemTime::now();

            let metric_families = prometheus::gather();
            let mut buffer = vec![];
            encoder.encode(&metric_families, &mut buffer).unwrap();
            http_body_gauge.set(buffer.len() as f64);

            let response = Response::builder()
                .status(200)
                .header(CONTENT_TYPE, encoder.format_type())
                .body(Body::from(buffer))
                .unwrap();

            http_req_histogram.record(timer.elapsed().unwrap().as_secs_f64());

            response
        })
    };

    let server = Server::bind(&addr)
        .serve(new_service)
        .map_err(|e| eprintln!("Server error: {}", e));

    hyper::rt::run(server);
}
