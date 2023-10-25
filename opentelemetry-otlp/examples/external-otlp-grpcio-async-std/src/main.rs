//! This shows how to connect to a third party collector like
//! honeycomb or lightstep using grpcio with tls and using async-std as reactor.
//! To run this specify a few environment variables like in the example:
//! ```shell
//! OTLP_GRPCIO_ENDPOINT=https://api.honeycomb.io:443 \
//! OTLP_GRPCIO_X_HONEYCOMB_TEAM=token \
//! OTLP_GRPCIO_X_HONEYCOMB_DATASET=dataset \
//! cargo run --bin external-otlp-grpcio-async-std
//! ```
use opentelemetry::{
    global::{shutdown_tracer_provider, tracer},
    trace::TraceError,
    trace::{TraceContextExt, Tracer},
    Key,
};
use opentelemetry_otlp::WithExportConfig;
use std::{
    collections::HashMap,
    env::{remove_var, set_var, var, vars},
    error::Error,
};
use url::Url;

// Use the variables to try and export the example to any external collector that accepts otlp
// like: oltp itself, honeycomb or lightstep
const ENDPOINT: &str = "OTLP_GRPCIO_ENDPOINT";
const HEADER_PREFIX: &str = "OTLP_GRPCIO_";

fn init_tracer() -> Result<opentelemetry_sdk::trace::Tracer, TraceError> {
    let endpoint = var(ENDPOINT).unwrap_or_else(|_| {
        panic!(
            "You must specify and endpoint to connect to with the variable {:?}.",
            ENDPOINT
        )
    });
    let endpoint = Url::parse(&endpoint).expect("endpoint is not a valid url");
    remove_var(ENDPOINT);

    let headers: HashMap<_, _> = vars()
        .filter(|(name, _)| name.starts_with(HEADER_PREFIX))
        .map(|(name, value)| {
            let header_name = name
                .strip_prefix(HEADER_PREFIX)
                .unwrap()
                .replace('_', "-")
                .to_ascii_lowercase();
            (header_name, value)
        })
        .collect();

    let grpcio_endpoint = format!(
        "{}:{}",
        endpoint.host_str().unwrap(),
        endpoint.port_or_known_default().unwrap()
    );

    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .grpcio()
                .with_endpoint(grpcio_endpoint)
                .with_headers(headers)
                .with_tls(true),
        )
        .install_batch(opentelemetry_sdk::runtime::AsyncStd)
}

const LEMONS_KEY: Key = Key::from_static_str("ex.com/lemons");
const ANOTHER_KEY: Key = Key::from_static_str("ex.com/another");

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    if let Err(std::env::VarError::NotPresent) = var("RUST_LOG") {
        set_var("RUST_LOG", "debug")
    };
    env_logger::init();
    let _ = init_tracer()?;

    let tracer = tracer("ex.com/basic");

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

    shutdown_tracer_provider();

    Ok(())
}
