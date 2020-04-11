use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;
use opentelemetry::api::{HttpTextFormat, KeyValue, Span, TraceContextPropagator, Tracer};
use opentelemetry::sdk::Sampler;
use opentelemetry::{api, global, sdk};

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

    global::set_provider(provider);

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_init()?;
    let mut client = GreeterClient::connect("http://[::1]:50051").await?;
    let propagator = TraceContextPropagator::new();
    let request_span = global::tracer("client").start("client-request", None);

    let mut request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });
    propagator.inject(
        request_span.get_context(),
        &mut TonicMetadataMapCarrier(request.metadata_mut()),
    );

    let response = client.say_hello(request).await?;

    request_span.add_event(
        "response-received".to_string(),
        vec![KeyValue::new("response", format!("{:?}", response))],
    );
    Ok(())
}
