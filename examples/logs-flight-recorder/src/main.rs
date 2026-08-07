use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::{TokioExecutor, TokioIo};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, Protocol, WithExportConfig};
use opentelemetry_sdk::logs::{
    BatchConfigBuilder, BatchLogProcessor, FlightRecorderLogProcessor, FlightRecorderTrigger,
    SdkLoggerProvider,
};
use opentelemetry_sdk::Resource;
use std::convert::Infallible;
use std::env;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};
use url::form_urlencoded;

const DEFAULT_LISTEN_ADDR: &str = "127.0.0.1:3000";
const DEFAULT_MAX_RECORDS: usize = 64;
const MAX_LOGS_PER_REQUEST: usize = 100;

type HttpBody = BoxBody<Bytes, Infallible>;

struct AppState {
    trigger: FlightRecorderTrigger,
    next_request_id: AtomicU64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Ok,
    Error,
}

struct WorkRequest {
    outcome: Outcome,
    log_count: usize,
    request_id: String,
}

fn init_logs(
    max_records: usize,
) -> Result<(SdkLoggerProvider, FlightRecorderTrigger), Box<dyn Error>> {
    let exporter = LogExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .build()?;
    let batch_processor = BatchLogProcessor::builder(exporter)
        .with_batch_config(
            BatchConfigBuilder::default()
                .with_max_queue_size(max_records)
                .with_max_export_batch_size(max_records)
                .build(),
        )
        .build();
    let (flight_recorder, trigger) = FlightRecorderLogProcessor::builder(batch_processor)
        .with_max_records(max_records)
        .build();
    let provider = SdkLoggerProvider::builder()
        .with_resource(
            Resource::builder_empty()
                .with_service_name("logs-flight-recorder")
                .build(),
        )
        .with_log_processor(flight_recorder)
        .build();

    Ok((provider, trigger))
}

fn response(status: StatusCode, body: impl Into<Bytes>) -> Response<HttpBody> {
    let mut response = Response::new(
        Full::new(body.into())
            .map_err(|never| match never {})
            .boxed(),
    );
    *response.status_mut() = status;
    response
}

fn parse_work_request(
    request: &Request<Incoming>,
    default_request_id: u64,
) -> Result<WorkRequest, &'static str> {
    let mut outcome = Outcome::Ok;
    let mut log_count = 5;
    let mut request_id = default_request_id.to_string();

    for (key, value) in form_urlencoded::parse(request.uri().query().unwrap_or_default().as_bytes())
    {
        match key.as_ref() {
            "result" => {
                outcome = match value.as_ref() {
                    "ok" => Outcome::Ok,
                    "error" => Outcome::Error,
                    _ => return Err("result must be 'ok' or 'error'"),
                };
            }
            "logs" => {
                log_count = value
                    .parse()
                    .map_err(|_| "logs must be an integer between 1 and 100")?;
                if !(1..=MAX_LOGS_PER_REQUEST).contains(&log_count) {
                    return Err("logs must be an integer between 1 and 100");
                }
            }
            "request_id" => request_id = value.into_owned(),
            _ => {}
        }
    }

    if request_id.is_empty() {
        return Err("request_id must not be empty");
    }

    Ok(WorkRequest {
        outcome,
        log_count,
        request_id,
    })
}

async fn handle_request(
    request: Request<Incoming>,
    state: Arc<AppState>,
) -> Result<Response<HttpBody>, Infallible> {
    if request.method() == Method::GET && request.uri().path() == "/health" {
        return Ok(response(StatusCode::OK, "healthy"));
    }
    if request.method() != Method::GET || request.uri().path() != "/work" {
        return Ok(response(StatusCode::NOT_FOUND, "not found"));
    }

    let request_number = state.next_request_id.fetch_add(1, Ordering::Relaxed);
    let work = match parse_work_request(&request, request_number) {
        Ok(work) => work,
        Err(message) => return Ok(response(StatusCode::BAD_REQUEST, message)),
    };
    let outcome = match work.outcome {
        Outcome::Ok => "ok",
        Outcome::Error => "error",
    };

    for step in 1..=work.log_count {
        info!(
            target: "flight_recorder_demo",
            event_kind = "work",
            request_id = work.request_id.as_str(),
            outcome,
            step,
            "processing request"
        );
    }

    if work.outcome == Outcome::Ok {
        return Ok(response(
            StatusCode::OK,
            format!(
                "request {} completed; logs remain in the flight recorder",
                work.request_id
            ),
        ));
    }

    error!(
        target: "flight_recorder_demo",
        event_kind = "failure",
        request_id = work.request_id.as_str(),
        outcome,
        "request failed; triggering the flight recorder"
    );
    match tokio::task::block_in_place(|| state.trigger.trigger()) {
        Ok(()) => Ok(response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!(
                "request {} failed; flight recorder snapshot exported",
                work.request_id
            ),
        )),
        Err(err) => Ok(response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("request {} failed; trigger failed: {err}", work.request_id),
        )),
    }
}

fn parse_env<T>(name: &str, default: T) -> Result<T, Box<dyn Error>>
where
    T: std::str::FromStr,
    T::Err: Error + 'static,
{
    match env::var(name) {
        Ok(value) => Ok(value.parse()?),
        Err(env::VarError::NotPresent) => Ok(default),
        Err(err) => Err(err.into()),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listen_addr: SocketAddr =
        parse_env("FLIGHT_RECORDER_LISTEN_ADDR", DEFAULT_LISTEN_ADDR.parse()?)?;
    let max_records = parse_env("FLIGHT_RECORDER_MAX_RECORDS", DEFAULT_MAX_RECORDS)?;
    if max_records == 0 {
        return Err("FLIGHT_RECORDER_MAX_RECORDS must be greater than zero".into());
    }

    let (logger_provider, trigger) = init_logs(max_records)?;
    let otel_layer = OpenTelemetryTracingBridge::new(&logger_provider)
        .with_filter(EnvFilter::new("off").add_directive("flight_recorder_demo=info".parse()?));
    tracing_subscriber::registry().with(otel_layer).try_init()?;

    let state = Arc::new(AppState {
        trigger,
        next_request_id: AtomicU64::new(1),
    });
    let listener = TcpListener::bind(listen_addr).await?;
    eprintln!("flight recorder demo listening on http://{listen_addr}");
    eprintln!("try: /work?result=ok&logs=5 and /work?result=error&logs=5");

    loop {
        tokio::select! {
            accepted = listener.accept() => {
                let (stream, _) = accepted?;
                let state = state.clone();
                tokio::spawn(async move {
                    if let Err(err) = hyper_util::server::conn::auto::Builder::new(
                        TokioExecutor::new(),
                    )
                    .serve_connection(
                        TokioIo::new(stream),
                        service_fn(move |request| handle_request(request, state.clone())),
                    )
                    .await
                    {
                        eprintln!("connection error: {err}");
                    }
                });
            }
            signal = tokio::signal::ctrl_c() => {
                signal?;
                break;
            }
        }
    }

    logger_provider.shutdown()?;
    Ok(())
}
