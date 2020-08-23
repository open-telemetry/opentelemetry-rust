use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;
use opentelemetry::api::{
    Context, TextMapFormat, KeyValue, TraceContextExt, TraceContextPropagator, Tracer,
};
use opentelemetry::sdk::Sampler;
use opentelemetry::{global, sdk};

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

    // For the demonstration, use `Sampler::AlwaysOn` sampler to sample all traces. In a production
    // application, use `Sampler::ParentOrElse` or `Sampler::Probability` with a desired probability.
    let provider = sdk::Provider::builder()
        .with_simple_exporter(exporter)
        .with_config(sdk::Config {
            default_sampler: Box::new(Sampler::AlwaysOn),
            ..Default::default()
        })
        .build();

    global::set_provider(provider);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_init()?;
    let mut client = GreeterClient::connect("http://[::1]:50051").await?;
    let propagator = TraceContextPropagator::new();
    let span = global::tracer("client").start("client-request");
    let cx = Context::current_with_span(span);

    let mut request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });
    propagator.inject_context(&cx, request.metadata_mut());

    let response = client.say_hello(request).await?;

    cx.span().add_event(
        "response-received".to_string(),
        vec![KeyValue::new("response", format!("{:?}", response))],
    );
    Ok(())
}
