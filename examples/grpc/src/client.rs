use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;
use opentelemetry::api::{
    trace::{TraceContextExt, Tracer},
    Context, KeyValue,
};
use opentelemetry::global;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use std::error::Error;

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

fn tracing_init(
) -> Result<(impl Tracer, opentelemetry_jaeger::Uninstall), Box<dyn Error + Send + Sync + 'static>>
{
    global::set_text_map_propagator(TraceContextPropagator::new());
    opentelemetry_jaeger::new_pipeline()
        .with_service_name("grpc-client")
        .install()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let (tracer, _uninstall) = tracing_init()?;
    let mut client = GreeterClient::connect("http://[::1]:50051").await?;
    let span = tracer.start("client-request");
    let cx = Context::current_with_span(span);

    let mut request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });
    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&cx, request.metadata_mut())
    });

    let response = client.say_hello(request).await?;

    cx.span().add_event(
        "response-received".to_string(),
        vec![KeyValue::new("response", format!("{:?}", response))],
    );
    Ok(())
}
