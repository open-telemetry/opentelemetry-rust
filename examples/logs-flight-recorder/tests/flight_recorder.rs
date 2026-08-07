use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::{TokioExecutor, TokioIo};
use opentelemetry_proto::tonic::collector::logs::v1::{
    ExportLogsServiceRequest, ExportLogsServiceResponse,
};
use opentelemetry_proto::tonic::common::v1::any_value::Value;
use opentelemetry_proto::tonic::logs::v1::LogRecord;
use prost::Message;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

async fn collect_otlp_request(
    request: Request<Incoming>,
    requests: Arc<Mutex<Vec<ExportLogsServiceRequest>>>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    if request.method() != Method::POST || request.uri().path() != "/v1/logs" {
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Full::new(Bytes::new()))
            .unwrap());
    }
    assert_eq!(
        request
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("application/x-protobuf")
    );

    let body = request.into_body().collect().await.unwrap().to_bytes();
    requests
        .lock()
        .unwrap()
        .push(ExportLogsServiceRequest::decode(body).unwrap());
    let response = ExportLogsServiceResponse::default().encode_to_vec();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/x-protobuf")
        .body(Full::new(Bytes::from(response)))
        .unwrap())
}

async fn start_fake_collector() -> (
    SocketAddr,
    Arc<Mutex<Vec<ExportLogsServiceRequest>>>,
    tokio::task::JoinHandle<()>,
) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let requests = Arc::new(Mutex::new(Vec::new()));
    let stored_requests = requests.clone();
    let task = tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let requests = stored_requests.clone();
            tokio::spawn(async move {
                hyper_util::server::conn::auto::Builder::new(TokioExecutor::new())
                    .serve_connection(
                        TokioIo::new(stream),
                        service_fn(move |request| collect_otlp_request(request, requests.clone())),
                    )
                    .await
                    .unwrap();
            });
        }
    });
    (address, requests, task)
}

async fn get_without_timeout(address: SocketAddr, path: &str) -> u16 {
    let mut stream = TcpStream::connect(address).await.unwrap();
    stream
        .write_all(
            format!("GET {path} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n")
                .as_bytes(),
        )
        .await
        .unwrap();
    let mut response = Vec::new();
    stream.read_to_end(&mut response).await.unwrap();
    let status_line = std::str::from_utf8(&response)
        .unwrap()
        .lines()
        .next()
        .unwrap();
    status_line
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse()
        .unwrap()
}

async fn get(address: SocketAddr, path: &str) -> u16 {
    tokio::time::timeout(Duration::from_secs(2), get_without_timeout(address, path))
        .await
        .expect("HTTP request timed out")
}

async fn wait_until_ready(address: SocketAddr) {
    for _ in 0..100 {
        if tokio::time::timeout(Duration::from_millis(100), TcpStream::connect(address))
            .await
            .is_ok_and(|result| result.is_ok())
            && get(address, "/health").await == 200
        {
            return;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("demo application did not become ready");
}

fn all_log_records(requests: &[ExportLogsServiceRequest]) -> Vec<&LogRecord> {
    requests
        .iter()
        .flat_map(|request| &request.resource_logs)
        .flat_map(|resource| &resource.scope_logs)
        .flat_map(|scope| &scope.log_records)
        .collect()
}

fn string_attribute<'a>(record: &'a LogRecord, key: &str) -> Option<&'a str> {
    record
        .attributes
        .iter()
        .find(|attribute| attribute.key == key)
        .and_then(|attribute| attribute.value.as_ref())
        .and_then(|value| value.value.as_ref())
        .and_then(|value| match value {
            Value::StringValue(value) => Some(value.as_str()),
            _ => None,
        })
}

fn int_attribute(record: &LogRecord, key: &str) -> Option<i64> {
    record
        .attributes
        .iter()
        .find(|attribute| attribute.key == key)
        .and_then(|attribute| attribute.value.as_ref())
        .and_then(|value| value.value.as_ref())
        .and_then(|value| match value {
            Value::IntValue(value) => Some(*value),
            _ => None,
        })
}

fn string_body(record: &LogRecord) -> Option<&str> {
    record
        .body
        .as_ref()
        .and_then(|body| body.value.as_ref())
        .and_then(|value| match value {
            Value::StringValue(value) => Some(value.as_str()),
            _ => None,
        })
}

async fn wait_for_records(
    requests: &Mutex<Vec<ExportLogsServiceRequest>>,
    expected_count: usize,
) -> Vec<LogRecord> {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let records = {
                let requests = requests.lock().unwrap();
                all_log_records(&requests)
                    .into_iter()
                    .cloned()
                    .collect::<Vec<_>>()
            };
            if records.len() >= expected_count {
                break records;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    })
    .await
    .expect("collector did not receive the expected logs")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn routes_high_severity_logs_and_exports_triggered_snapshots_to_otlp() {
    let (collector_address, requests, collector_task) = start_fake_collector().await;
    let app_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let app_address = app_listener.local_addr().unwrap();
    drop(app_listener);

    let child = Command::new(env!("CARGO_BIN_EXE_logs-flight-recorder"))
        .env("FLIGHT_RECORDER_LISTEN_ADDR", app_address.to_string())
        .env("FLIGHT_RECORDER_MAX_RECORDS", "5")
        .env(
            "OTEL_EXPORTER_OTLP_LOGS_ENDPOINT",
            format!("http://{collector_address}/v1/logs"),
        )
        .env("OTEL_EXPORTER_OTLP_LOGS_PROTOCOL", "http/protobuf")
        .env_remove("OTEL_EXPORTER_OTLP_LOGS_COMPRESSION")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let _child = ChildGuard(child);
    wait_until_ready(app_address).await;

    assert_eq!(
        get(app_address, "/work?result=ok&logs=4&request_id=successful").await,
        200
    );
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert!(requests.lock().unwrap().is_empty());

    assert_eq!(
        get(app_address, "/work?result=warn&logs=1&request_id=warning").await,
        200
    );
    let records = wait_for_records(&requests, 1).await;
    assert_eq!(records.len(), 1);
    assert_eq!(string_attribute(&records[0], "request_id"), Some("warning"));
    assert_eq!(string_attribute(&records[0], "event_kind"), Some("warning"));
    assert_eq!(records[0].severity_number, 13);
    assert_eq!(
        string_body(&records[0]),
        Some("request completed with a warning; bypassing the flight recorder")
    );

    assert_eq!(
        get(app_address, "/work?result=error&logs=2&request_id=failing").await,
        500
    );

    let records = wait_for_records(&requests, 7).await;
    assert_eq!(records.len(), 7);

    let identities = records
        .iter()
        .map(|record| {
            assert!(matches!(
                string_body(record),
                Some(
                    "processing request"
                        | "request completed with a warning; bypassing the flight recorder"
                        | "request failed; triggering the flight recorder"
                )
            ));
            (
                string_attribute(record, "request_id").unwrap().to_string(),
                string_attribute(record, "event_kind").unwrap().to_string(),
                int_attribute(record, "step"),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        identities,
        [
            ("warning".into(), "warning".into(), None),
            ("failing".into(), "failure".into(), None),
            ("successful".into(), "work".into(), Some(3)),
            ("successful".into(), "work".into(), Some(4)),
            ("warning".into(), "work".into(), Some(1)),
            ("failing".into(), "work".into(), Some(1)),
            ("failing".into(), "work".into(), Some(2)),
        ]
    );
    assert_eq!(records[1].severity_number, 17);
    assert!(records[2..]
        .iter()
        .all(|record| record.severity_number == 9));

    assert_eq!(
        get(
            app_address,
            "/work?result=ok&logs=1&request_id=after-trigger"
        )
        .await,
        200
    );
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(all_log_records(&requests.lock().unwrap()).len(), 7);

    assert_eq!(
        get(
            app_address,
            "/work?result=error&logs=1&request_id=second-failure"
        )
        .await,
        500
    );
    let records = wait_for_records(&requests, 10).await;
    assert_eq!(records.len(), 10);
    let second_snapshot = records[7..]
        .iter()
        .map(|record| {
            (
                string_attribute(record, "request_id").unwrap(),
                string_attribute(record, "event_kind").unwrap(),
                int_attribute(record, "step"),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        second_snapshot,
        [
            ("second-failure", "failure", None),
            ("after-trigger", "work", Some(1)),
            ("second-failure", "work", Some(1)),
        ]
    );

    collector_task.abort();
}
