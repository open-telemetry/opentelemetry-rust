use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::{body::Incoming, service::service_fn, Request, Response, StatusCode};
use hyper_util::rt::{TokioExecutor, TokioIo};
use opentelemetry::{
    baggage::BaggageExt,
    global::{self, BoxedTracer},
    logs::LogRecord,
    propagation::TextMapCompositePropagator,
    trace::{Span, SpanKind, TraceContextExt, Tracer, FutureContextExt},
    Context, InstrumentationScope, KeyValue,
};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_http::{Bytes, HeaderExtractor};
use opentelemetry_sdk::{
    error::OTelSdkResult,
    logs::{LogProcessor, SdkLogRecord, SdkLoggerProvider},
    propagation::{BaggagePropagator, TraceContextPropagator},
    trace::{SdkTracerProvider, SpanProcessor},
};
use opentelemetry_semantic_conventions::trace;
use opentelemetry_stdout::{LogExporter, SpanExporter};
use std::time::Duration;
use std::{convert::Infallible, net::SocketAddr, sync::OnceLock};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn get_tracer() -> &'static BoxedTracer {
    static TRACER: OnceLock<BoxedTracer> = OnceLock::new();
    TRACER.get_or_init(|| global::tracer("example/server"))
}

// Utility function to extract the context from the incoming request headers
fn extract_context_from_request(req: &Request<Incoming>) -> Context {
    global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(req.headers()))
    })
}

// Separate async function for the handle endpoint
async fn handle_health_check(
    _req: Request<Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, Infallible> {
    let tracer = get_tracer();
    let _span = tracer
        .span_builder("health_check")
        .with_kind(SpanKind::Internal)
        .start(tracer);
    info!(name: "health_check", message = "Health check endpoint hit");

    let res = Response::new(
        Full::new(Bytes::from_static(b"Server is up and running!"))
            .map_err(|err| match err {})
            .boxed(),
    );

    Ok(res)
}

// Separate async function for the echo endpoint
async fn handle_echo(
    req: Request<Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, Infallible> {
    let tracer = get_tracer();
    let _span = tracer
        .span_builder("echo")
        .with_kind(SpanKind::Internal)
        .start(tracer);
    info!(name = "echo", message = "Echo endpoint hit");

    let res = Response::new(req.into_body().boxed());

    Ok(res)
}

async fn router(
    req: Request<Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, Infallible> {
    // Extract the context from the incoming request headers
    let parent_cx = extract_context_from_request(&req);
    let response = {
        // Create a span parenting the remote client span.
        let tracer = get_tracer();
        let span = tracer
            .span_builder("router")
            .with_kind(SpanKind::Server)
            .start_with_context(tracer, &parent_cx);

        info!(name = "router", message = "Dispatching request");

        let cx = parent_cx.with_span(span);
        match (req.method(), req.uri().path()) {
            (&hyper::Method::GET, "/health") => handle_health_check(req).with_context(cx).await,
            (&hyper::Method::GET, "/echo") => handle_echo(req).with_context(cx).await,
            _ => {
                cx.span()
                    .set_attribute(KeyValue::new(trace::HTTP_RESPONSE_STATUS_CODE, 404));
                let mut not_found = Response::new(BoxBody::default());
                *not_found.status_mut() = StatusCode::NOT_FOUND;
                Ok(not_found)
            }
        }
    };

    response
}

/// A custom log processor that enriches LogRecords with baggage attributes.
/// Baggage information is not added automatically without this processor.
#[derive(Debug)]
struct EnrichWithBaggageLogProcessor;
impl LogProcessor for EnrichWithBaggageLogProcessor {
    fn emit(&self, data: &mut SdkLogRecord, _instrumentation: &InstrumentationScope) {
        Context::map_current(|cx| {
            for (kk, vv) in cx.baggage().iter() {
                data.add_attribute(kk.clone(), vv.0.clone());
            }
        });
    }

    fn force_flush(&self) -> OTelSdkResult {
        Ok(())
    }

    fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
        Ok(())
    }
}

/// A custom span processor that enriches spans with baggage attributes. Baggage
/// information is not added automatically without this processor.
#[derive(Debug)]
struct EnrichWithBaggageSpanProcessor;
impl SpanProcessor for EnrichWithBaggageSpanProcessor {
    fn force_flush(&self) -> OTelSdkResult {
        Ok(())
    }

    fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
        Ok(())
    }

    fn on_start(&self, span: &mut opentelemetry_sdk::trace::Span, cx: &Context) {
        for (kk, vv) in cx.baggage().iter() {
            span.set_attribute(KeyValue::new(kk.clone(), vv.0.clone()));
        }
    }

    fn on_end(&self, _span: opentelemetry_sdk::trace::SpanData) {}
}

fn init_tracer() -> SdkTracerProvider {
    let baggage_propagator = BaggagePropagator::new();
    let trace_context_propagator = TraceContextPropagator::new();
    let composite_propagator = TextMapCompositePropagator::new(vec![
        Box::new(baggage_propagator),
        Box::new(trace_context_propagator),
    ]);

    global::set_text_map_propagator(composite_propagator);

    // Setup tracerprovider with stdout exporter
    // that prints the spans to stdout.
    let provider = SdkTracerProvider::builder()
        .with_span_processor(EnrichWithBaggageSpanProcessor)
        .with_simple_exporter(SpanExporter::default())
        .build();

    global::set_tracer_provider(provider.clone());
    provider
}

fn init_logs() -> SdkLoggerProvider {
    // Setup tracerprovider with stdout exporter
    // that prints the spans to stdout.
    let logger_provider = SdkLoggerProvider::builder()
        .with_log_processor(EnrichWithBaggageLogProcessor)
        .with_simple_exporter(LogExporter::default())
        .build();
    let otel_layer = OpenTelemetryTracingBridge::new(&logger_provider);
    tracing_subscriber::registry().with(otel_layer).init();

    logger_provider
}

#[tokio::main]
async fn main() {
    use hyper_util::server::conn::auto::Builder;

    let provider = init_tracer();
    let logger_provider = init_logs();
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await.unwrap();

    while let Ok((stream, _addr)) = listener.accept().await {
        if let Err(err) = Builder::new(TokioExecutor::new())
            .serve_connection(TokioIo::new(stream), service_fn(router))
            .await
        {
            eprintln!("{err}");
        }
    }

    provider.shutdown().expect("Shutdown provider failed");
    logger_provider
        .shutdown()
        .expect("Shutdown provider failed");
}
