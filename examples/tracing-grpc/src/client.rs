use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;
use opentelemetry::api::{HttpTextFormat, KeyValue, Provider, TraceContextPropagator};
use opentelemetry::sdk::Sampler;
use opentelemetry::{api, sdk};
use tracing::*;
use tracing_futures::Instrument;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::prelude::*;

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

fn tracing_init() -> Result<(), Box<dyn std::error::Error>> {
    let builder = opentelemetry_jaeger::Exporter::builder()
        .with_agent_endpoint("127.0.0.1:6831".parse().unwrap());

    let exporter = builder
        .with_process(opentelemetry_jaeger::Process {
            service_name: "grpc-client".to_string(),
            tags: vec![KeyValue::new("version", "0.1.0")],
        })
        .init()?;

    // For the demonstration, use `Sampler::Always` sampler to sample all traces. In a production
    // application, use `Sampler::Parent` or `Sampler::Probability` with a desired probability.
    let provider = sdk::Provider::builder()
        .with_simple_exporter(exporter)
        .with_config(sdk::Config {
            default_sampler: Box::new(Sampler::Always),
            ..Default::default()
        })
        .build();
    let tracer = provider.get_tracer("grpc-client");

    let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    tracing_subscriber::registry()
        .with(opentelemetry)
        .try_init()?;

    Ok(())
}

struct TonicMetadataMapCarrier<'a>(&'a mut tonic::metadata::MetadataMap);
impl<'a> api::Carrier for TonicMetadataMapCarrier<'a> {
    fn get(&self, key: &'static str) -> Option<&str> {
        self.0.get(key).and_then(|metadata| metadata.to_str().ok())
    }

    fn set(&mut self, key: &'static str, value: String) {
        if let Ok(key) = tonic::metadata::MetadataKey::from_bytes(key.to_lowercase().as_bytes()) {
            self.0.insert(
                key,
                tonic::metadata::MetadataValue::from_str(&value).unwrap(),
            );
        }
    }
}

#[instrument]
async fn greet() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GreeterClient::connect("http://[::1]:50051")
        .instrument(info_span!("client connect"))
        .await?;
    let propagator = TraceContextPropagator::new();
    let cx = tracing::Span::current().context();

    let mut request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });
    propagator.inject_context(&cx, &mut TonicMetadataMapCarrier(request.metadata_mut()));

    let response = client
        .say_hello(request)
        .instrument(info_span!("say_hello"))
        .await?;
    info!("Response received: {:?}", response);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_init()?;
    greet().await?;

    Ok(())
}
