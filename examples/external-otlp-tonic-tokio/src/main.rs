//! This should show how to connect to a third party collector like
//! honeycomb or lightstep using tonic with tls and using tokio as reactor.
//! To run this you have to specify a few environment variables like in the example:
//! ```shell
//! OTLP_TONIC_ENDPOINT=https://api.honeycomb.io:443 \
//! OTLP_TONIC_X_HONEYCOMB_TEAM=token \
//! OTLP_TONIC_X_HONEYCOMB_DATASET=dataset \'
//! cargo run --bin external-otlp-tonic-tokio
//! ```
use opentelemetry::trace::TraceError;
use opentelemetry::{global, sdk::trace as sdktrace};
use opentelemetry::{
    trace::{TraceContextExt, Tracer},
    Key,
};
use tonic::{
    metadata::{MetadataKey, MetadataMap},
    transport::ClientTlsConfig,
};
use url::Url;

use opentelemetry::global::shutdown_tracer_provider;
use opentelemetry_otlp::WithExportConfig;
use std::{env::vars, str::FromStr, time::Duration};
use std::{
    env::{remove_var, var},
    error::Error,
};

// Use the variables to try and export the example to any external collector that accepts otlp
// like: oltp itself, honeycomb or lightstep
const ENDPOINT: &str = "OTLP_TONIC_ENDPOINT";
const HEADER_PREFIX: &str = "OTLP_TONIC_";

fn init_tracer() -> Result<sdktrace::Tracer, TraceError> {
    let endpoint = var(ENDPOINT).unwrap_or_else(|_| {
        panic!(
            "You must specify and endpoint to connect to with the variable {:?}.",
            ENDPOINT
        )
    });
    let endpoint = Url::parse(&endpoint).expect("endpoint is not a valid url");
    remove_var(ENDPOINT);
    let mut metadata = MetadataMap::new();
    for (key, value) in vars()
        .filter(|(name, _)| name.starts_with(HEADER_PREFIX))
        .map(|(name, value)| {
            let header_name = name
                .strip_prefix(HEADER_PREFIX)
                .map(|h| h.replace('_', "-"))
                .map(|h| h.to_ascii_lowercase())
                .unwrap();
            (header_name, value)
        })
    {
        metadata.insert(MetadataKey::from_str(&key).unwrap(), value.parse().unwrap());
    }

    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint.as_str())
                .with_metadata(dbg!(metadata))
                .with_tls_config(
                    ClientTlsConfig::new().domain_name(
                        endpoint
                            .host_str()
                            .expect("the specified endpoint should have a valid host"),
                    ),
                ),
        )
        .install_batch(opentelemetry::runtime::Tokio)
}

const LEMONS_KEY: Key = Key::from_static_str("ex.com/lemons");
const ANOTHER_KEY: Key = Key::from_static_str("ex.com/another");

#[tokio::main]
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

    // wait for 1 minutes so that we could see metrics being pushed via OTLP every 10 seconds.
    tokio::time::sleep(Duration::from_secs(60)).await;

    shutdown_tracer_provider();

    Ok(())
}
