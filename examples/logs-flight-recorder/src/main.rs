use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::{TokioExecutor, TokioIo};
use opentelemetry::logs::Severity;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, Protocol, WithExportConfig};
use opentelemetry_sdk::logs::{
    BatchConfigBuilder, BatchLogProcessor, ScopedFlightRecorder, ScopedFlightRecorderLogProcessor,
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
use tracing::{error, info, warn};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};
use url::form_urlencoded;

const DEFAULT_LISTEN_ADDR: &str = "127.0.0.1:3000";
const DEFAULT_MAX_RECORDS: usize = 64;
const DEFAULT_MAX_ACTIVE_SCOPES: usize = 1_024;
const MAX_LOGS_PER_REQUEST: usize = 100;

type HttpBody = BoxBody<Bytes, Infallible>;

struct AppState {
    recorder: ScopedFlightRecorder,
    next_request_id: AtomicU64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Ok,
    Warn,
    Error,
}

struct WorkRequest {
    outcome: Outcome,
    log_count: usize,
    delay_ms: u64,
    request_id: String,
}

fn init_logs(
    max_records: usize,
    max_active_scopes: usize,
) -> Result<(SdkLoggerProvider, ScopedFlightRecorder), Box<dyn Error>> {
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
    let (flight_recorder, recorder) = ScopedFlightRecorderLogProcessor::builder(batch_processor)
        .with_max_records_per_scope(max_records)
        .with_max_active_scopes(max_active_scopes)
        .with_max_buffered_severity(Severity::Info4)
        .build();
    let provider = SdkLoggerProvider::builder()
        .with_resource(
            Resource::builder_empty()
                .with_service_name("logs-flight-recorder")
                .build(),
        )
        .with_log_processor(flight_recorder)
        .build();

    Ok((provider, recorder))
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
    let mut delay_ms = 0;
    let mut request_id = default_request_id.to_string();

    for (key, value) in form_urlencoded::parse(request.uri().query().unwrap_or_default().as_bytes())
    {
        match key.as_ref() {
            "result" => {
                outcome = match value.as_ref() {
                    "ok" => Outcome::Ok,
                    "warn" => Outcome::Warn,
                    "error" => Outcome::Error,
                    _ => return Err("result must be 'ok', 'warn', or 'error'"),
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
            "delay_ms" => {
                delay_ms = value
                    .parse()
                    .map_err(|_| "delay_ms must be an integer between 0 and 1000")?;
                if delay_ms > 1_000 {
                    return Err("delay_ms must be an integer between 0 and 1000");
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
        delay_ms,
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
        Outcome::Warn => "warn",
        Outcome::Error => "error",
    };
    let recording_scope = match state.recorder.try_start() {
        Some(scope) => scope,
        None => {
            return Ok(response(
                StatusCode::SERVICE_UNAVAILABLE,
                "flight recorder active-scope limit reached",
            ));
        }
    };

    recording_scope
        .with_context(async {
            for step in 1..=work.log_count {
                info!(
                    target: "flight_recorder_demo",
                    event_kind = "work",
                    request_id = work.request_id.as_str(),
                    outcome,
                    step,
                    "processing request"
                );
                if work.delay_ms > 0 {
                    tokio::time::sleep(std::time::Duration::from_millis(work.delay_ms)).await;
                }
            }

            if work.outcome == Outcome::Warn {
                warn!(
                    target: "flight_recorder_demo",
                    event_kind = "warning",
                    request_id = work.request_id.as_str(),
                    outcome,
                    "request completed with a warning; bypassing the flight recorder"
                );
            } else if work.outcome == Outcome::Error {
                error!(
                    target: "flight_recorder_demo",
                    event_kind = "failure",
                    request_id = work.request_id.as_str(),
                    outcome,
                    "request failed; triggering the flight recorder"
                );
            }
        })
        .await;

    if work.outcome == Outcome::Ok {
        recording_scope.discard();
        return Ok(response(
            StatusCode::OK,
            format!(
                "request {} completed; scoped logs discarded",
                work.request_id
            ),
        ));
    }

    if work.outcome == Outcome::Warn {
        recording_scope.discard();
        return Ok(response(
            StatusCode::OK,
            format!(
                "request {} completed; warning followed the normal export path",
                work.request_id
            ),
        ));
    }

    match tokio::task::block_in_place(|| recording_scope.trigger()) {
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
    let max_active_scopes = parse_env(
        "FLIGHT_RECORDER_MAX_ACTIVE_SCOPES",
        DEFAULT_MAX_ACTIVE_SCOPES,
    )?;
    if max_active_scopes == 0 {
        return Err("FLIGHT_RECORDER_MAX_ACTIVE_SCOPES must be greater than zero".into());
    }

    let (logger_provider, recorder) = init_logs(max_records, max_active_scopes)?;
    let otel_layer = OpenTelemetryTracingBridge::new(&logger_provider)
        .with_filter(EnvFilter::new("off").add_directive("flight_recorder_demo=info".parse()?));
    tracing_subscriber::registry().with(otel_layer).try_init()?;

    let state = Arc::new(AppState {
        recorder,
        next_request_id: AtomicU64::new(1),
    });
    let listener = TcpListener::bind(listen_addr).await?;
    eprintln!("flight recorder demo listening on http://{listen_addr}");
    eprintln!("try: /work?result=ok|warn|error&logs=5&delay_ms=0");

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
