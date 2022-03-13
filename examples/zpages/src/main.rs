use hyper::http::{Request, Response};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Server};
use opentelemetry::trace::{Span, Status};
use opentelemetry::{global, runtime::Tokio, sdk::trace, trace::Tracer};
use opentelemetry_zpages::{tracez, TracezError, TracezQuerier, TracezResponse};
use rand::Rng;
use std::str::FromStr;
use std::sync::Arc;
use std::{convert::Infallible, net::SocketAddr};
use tokio::time::Duration;

async fn handler(
    req: Request<Body>,
    querier: Arc<TracezQuerier>,
) -> Result<Response<Body>, Infallible> {
    Ok::<_, Infallible>(match req.uri().path() {
        uri if uri.starts_with("/tracez/api") => {
            // if it is api call
            let parts = uri
                .split('/')
                .filter(|x| !x.is_empty())
                .collect::<Vec<&str>>();
            if parts.len() < 3 {
                Response::builder().status(404).body(Body::empty()).unwrap()
            } else {
                let operation_name = *(parts.get(2).unwrap_or(&""));
                match operation_name {
                    "aggregations" => tracez_response_or_server_error(querier.aggregation().await),
                    "running" => {
                        if let Some(&span_name) = parts.get(3) {
                            tracez_response_or_server_error(querier.running(span_name.into()).await)
                        } else {
                            Response::builder().status(404).body(Body::empty()).unwrap()
                        }
                    }
                    "error" => {
                        if let Some(&span_name) = parts.get(3) {
                            tracez_response_or_server_error(querier.error(span_name.into()).await)
                        } else {
                            Response::builder().status(404).body(Body::empty()).unwrap()
                        }
                    }
                    "latency" => {
                        let bucket_index = parts.get(3);
                        let span_name = parts.get(4);
                        match (bucket_index, span_name) {
                            (Some(&bucket_index), Some(&span_name)) => {
                                if let Ok(bucket_index) = u32::from_str(bucket_index) {
                                    tracez_response_or_server_error(
                                        querier
                                            .latency(bucket_index as usize, span_name.into())
                                            .await,
                                    )
                                } else {
                                    Response::builder().status(404).body(Body::empty()).unwrap()
                                }
                            }
                            (_, _) => Response::builder().status(404).body(Body::empty()).unwrap(),
                        }
                    }
                    _ => Response::builder().status(404).body(Body::empty()).unwrap(),
                }
            }
        }
        "/running" => {
            let span_duration = Duration::from_millis(rand::thread_rng().gen_range(1..6000));
            let mut spans = global::tracer("zpages-test").start("running-spans");
            spans.set_status(Status::Ok);
            tokio::time::sleep(span_duration).await;
            println!("The span slept for {} ms", span_duration.as_millis());
            Response::new(Body::empty())
        }
        _ => Response::builder().status(404).body(Body::empty()).unwrap(),
    })
}

fn tracez_response_or_server_error(resp: Result<TracezResponse, TracezError>) -> Response<Body> {
    match resp {
        Ok(resp) => Response::new(Body::from(serde_json::to_string(&resp).unwrap())),
        Err(_) => Response::builder().status(500).body(Body::empty()).unwrap(),
    }
}

#[tokio::main]
async fn main() {
    let (processor, querier) = tracez(5, Tokio);
    let provider = trace::TracerProvider::builder()
        .with_span_processor(processor)
        .build();
    global::set_tracer_provider(provider);
    let querier = Arc::new(querier);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let server = Server::bind(&addr).serve(make_service_fn(move |_conn| {
        let inner = Arc::clone(&querier);
        async move { Ok::<_, Infallible>(service_fn(move |req| handler(req, Arc::clone(&inner)))) }
    }));

    println!("Listening on {}", addr);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
