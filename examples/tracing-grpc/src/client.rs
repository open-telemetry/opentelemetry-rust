use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;
use opentelemetry::{global, propagation::Injector};
use opentelemetry_sdk::{
    propagation::TraceContextPropagator, runtime::Tokio, trace::TracerProvider,
};
use opentelemetry_stdout::SpanExporter;

use opentelemetry::{
    trace::{SpanKind, TraceContextExt, Tracer},
    Context, KeyValue,
};

fn init_tracer() {
    global::set_text_map_propagator(TraceContextPropagator::new());
    // Install stdout exporter pipeline to be able to retrieve the collected spans.
    let provider = TracerProvider::builder()
        .with_batch_exporter(SpanExporter::default(), Tokio)
        .build();

    global::set_tracer_provider(provider);
}

struct MetadataMap<'a>(&'a mut tonic::metadata::MetadataMap);

impl<'a> Injector for MetadataMap<'a> {
    /// Set a key and value in the MetadataMap.  Does nothing if the key or value are not valid inputs
    fn set(&mut self, key: &str, value: String) {
        if let Ok(key) = tonic::metadata::MetadataKey::from_bytes(key.as_bytes()) {
            if let Ok(val) = tonic::metadata::MetadataValue::try_from(&value) {
                self.0.insert(key, val);
            }
        }
    }
}

#[allow(clippy::derive_partial_eq_without_eq)] // tonic don't derive Eq for generated types. We shouldn't manually change it.
pub mod hello_world {
    tonic::include_proto!("helloworld");
}

async fn greet() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let tracer = global::tracer("example/client");
    let span = tracer
        .span_builder(String::from("Greeter/client"))
        .with_kind(SpanKind::Client)
        .with_attributes(vec![KeyValue::new("component", "grpc")])
        .start(&tracer);
    let cx = Context::current_with_span(span);
    let mut client = GreeterClient::connect("http://[::1]:50051").await?;

    let mut request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&cx, &mut MetadataMap(request.metadata_mut()))
    });

    let response = client.say_hello(request).await;

    let status = match response {
        Ok(_res) => "OK".to_string(),
        Err(status) => {
            // Access the status code
            let status_code = status.code();
            status_code.to_string()
        }
    };
    cx.span().add_event(
        "Got response!".to_string(),
        vec![KeyValue::new("status", status)],
    );

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    init_tracer();
    greet().await?;
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}
