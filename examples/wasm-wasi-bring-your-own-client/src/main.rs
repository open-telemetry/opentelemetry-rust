use opentelemetry::global::shutdown_tracer_provider;
use opentelemetry::runtime;
use opentelemetry::sdk::Resource;
use opentelemetry::trace::TraceError;
use opentelemetry::{global, sdk::trace as sdktrace};
use opentelemetry::{
    metrics,
    trace::{TraceContextExt, Tracer},
    Context, Key, KeyValue,
};
use opentelemetry_http::{Bytes, HttpError, Response, Request};
use opentelemetry_otlp::{ExportConfig, WithExportConfig};
use std::error::Error;
use std::time::Duration;

#[derive(Debug)]
struct MyOwnHttpClient;

#[async_trait::async_trait]
impl opentelemetry_http::HttpClient for MyOwnHttpClient {
    async fn send(&self, request: Request<Vec<u8>>) -> Result<Response<Bytes>, HttpError> {
        todo!()
    }
}

fn init_tracer() -> Result<sdktrace::Tracer, TraceError> {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::HttpExporterBuilder::default().with_http_client(MyOwnHttpClient),
        )
        .with_trace_config(
            sdktrace::config().with_resource(Resource::new(vec![KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                "trace-demo",
            )])),
        )
        .install_simple()
}

const LEMONS_KEY: Key = Key::from_static_str("lemons");
const ANOTHER_KEY: Key = Key::from_static_str("ex.com/another");

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let _ = init_tracer()?;
    let tracer = global::tracer("ex.com/basic");

    tracer.in_span("operation", |cx| {
        let span = cx.span();
        span.add_event(
            "Nice operation!".to_string(),
            vec![Key::new("bogons").i64(100)],
        );
        span.set_attribute(ANOTHER_KEY.string("yes"));

        tracer.in_span("Sub operation...", |cx| {
            let span = cx.span();
            span.set_attribute(LEMONS_KEY.string("five"));

            span.add_event("Sub span event", vec![]);
        });
    });

    // // wait for 1 minutes so that we could see metrics being pushed via OTLP every 10 seconds.
    // tokio::time::sleep(Duration::from_secs(60)).await;

    shutdown_tracer_provider();

    Ok(())
}
