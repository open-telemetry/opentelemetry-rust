use http_body_util::Full;
use hyper_util::{client::legacy::Client, rt::TokioExecutor};
use log::{info, Level};
use opentelemetry::{
    global,
    trace::{SpanKind, TraceContextExt, Tracer},
    Context,
};
use opentelemetry_appender_log::OpenTelemetryLogBridge;
use opentelemetry_http::{Bytes, HeaderInjector};
use opentelemetry_sdk::{
    logs::SdkLoggerProvider, propagation::TraceContextPropagator, trace::SdkTracerProvider,
};
use opentelemetry_stdout::{LogExporter, SpanExporter};

fn init_tracer() -> SdkTracerProvider {
    global::set_text_map_propagator(TraceContextPropagator::new());
    // Install stdout exporter pipeline to be able to retrieve the collected spans.
    // For the demonstration, use `Sampler::AlwaysOn` sampler to sample all traces.
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(SpanExporter::default())
        .build();

    global::set_tracer_provider(provider.clone());
    provider
}

fn init_logs() -> SdkLoggerProvider {
    // Setup tracerprovider with stdout exporter
    // that prints the spans to stdout.
    let logger_provider = SdkLoggerProvider::builder()
        .with_simple_exporter(LogExporter::default())
        .build();
    // Setup Log Appender for the log crate.
    let otel_log_appender = OpenTelemetryLogBridge::new(&logger_provider);
    log::set_boxed_logger(Box::new(otel_log_appender)).unwrap();
    log::set_max_level(Level::Info.to_level_filter());

    logger_provider
}

async fn send_request(
    url: &str,
    body_content: &str,
    span_name: &str,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let client = Client::builder(TokioExecutor::new()).build_http();
    let tracer = global::tracer("example/client");
    let span = tracer
        .span_builder(String::from(span_name))
        .with_kind(SpanKind::Client)
        .start(&tracer);
    let cx = Context::current_with_span(span);

    let mut req = hyper::Request::builder().uri(url);
    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&cx, &mut HeaderInjector(req.headers_mut().unwrap()))
    });
    req.headers_mut()
        .unwrap()
        .insert("baggage", "is_synthetic=true".parse().unwrap());
    let res = client
        .request(req.body(Full::new(Bytes::from(body_content.to_string())))?)
        .await?;

    info!(name = "ResponseReceived", status:% = res.status(); "Response received");

    Ok(())
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let tracer_provider = init_tracer();
    let logger_provider = init_logs();

    send_request(
        "http://127.0.0.1:3000/health",
        "Health Request!",
        "server_health_check",
    )
    .await?;
    send_request(
        "http://127.0.0.1:3000/echo",
        "Echo Request!",
        "server_echo_check",
    )
    .await?;

    tracer_provider
        .shutdown()
        .expect("Shutdown provider failed");
    logger_provider
        .shutdown()
        .expect("Shutdown provider failed");
    Ok(())
}
