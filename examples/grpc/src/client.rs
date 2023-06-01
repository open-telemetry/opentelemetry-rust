use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;
use opentelemetry::global;
use opentelemetry::global::shutdown_tracer_provider;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use opentelemetry::trace::TraceResult;
use opentelemetry::{
    propagation::Injector,
    sdk::trace::Tracer,
    trace::{TraceContextExt, Tracer as _},
    Context, KeyValue,
};

struct MetadataMap<'a>(&'a mut tonic::metadata::MetadataMap);

impl<'a> Injector for MetadataMap<'a> {
    /// Set a key and value in the MetadataMap.  Does nothing if the key or value are not valid inputs
    fn set(&mut self, key: &str, value: String) {
        if let Ok(key) = tonic::metadata::MetadataKey::from_bytes(key.as_bytes()) {
            if let Ok(val) = tonic::metadata::MetadataValue::from_str(&value) {
                self.0.insert(key, val);
            }
        }
    }
}

#[allow(clippy::derive_partial_eq_without_eq)] // tonic don't derive Eq for generated types. We shouldn't manually change it.
pub mod hello_world {
    tonic::include_proto!("helloworld");
}

fn tracing_init() -> TraceResult<Tracer> {
    global::set_text_map_propagator(TraceContextPropagator::new());
    opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name("grpc-client")
        .install_simple()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let tracer = tracing_init()?;
    let mut client = GreeterClient::connect("http://[::1]:50051").await?;
    let span = tracer.start("client-request");
    let cx = Context::current_with_span(span);

    let mut request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });
    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&cx, &mut MetadataMap(request.metadata_mut()))
    });

    let response = client.say_hello(request).await?;

    // `cx` initialized with span above, so unwrapping is safe
    cx.span().unwrap().add_event(
        "response-received".to_string(),
        vec![KeyValue::new("response", format!("{response:?}"))],
    );

    shutdown_tracer_provider();

    Ok(())
}
