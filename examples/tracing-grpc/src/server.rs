use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{HelloReply, HelloRequest};
use opentelemetry::{
    global,
    propagation::Extractor,
    trace::{Span, SpanKind, Tracer},
};
use opentelemetry_sdk::{
    propagation::TraceContextPropagator, runtime::Tokio, trace::TracerProvider,
};
use opentelemetry_stdout::SpanExporterBuilder;
use tonic::{transport::Server, Request, Response, Status};

fn init_tracer() {
    global::set_text_map_propagator(TraceContextPropagator::new());
    // Install stdout exporter pipeline to be able to retrieve the collected spans.
    let provider = TracerProvider::builder()
        .with_batch_exporter(
            SpanExporterBuilder::default()
                .with_encoder(|writer, data| {
                    Ok(serde_json::to_writer_pretty(writer, &data).unwrap())
                })
                .build(),
            Tokio,
        )
        .build();

    global::set_tracer_provider(provider);
}

#[allow(clippy::derive_partial_eq_without_eq)] // tonic don't derive Eq for generated types. We shouldn't manually change it.
pub mod hello_world {
    tonic::include_proto!("helloworld"); // The string specified here must match the proto package name
}

struct MetadataMap<'a>(&'a tonic::metadata::MetadataMap);

impl<'a> Extractor for MetadataMap<'a> {
    /// Get a value for a key from the MetadataMap.  If the value can't be converted to &str, returns None
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|metadata| metadata.to_str().ok())
    }

    /// Collect all the keys from the MetadataMap.
    fn keys(&self) -> Vec<&str> {
        self.0
            .keys()
            .map(|key| match key {
                tonic::metadata::KeyRef::Ascii(v) => v.as_str(),
                tonic::metadata::KeyRef::Binary(v) => v.as_str(),
            })
            .collect::<Vec<_>>()
    }
}

fn expensive_fn<S: Span>(to_print: String, span: &mut S) {
    std::thread::sleep(std::time::Duration::from_millis(20));
    span.add_event(to_print, vec![]);
}

#[derive(Debug, Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>, // Accept request of type HelloRequest
    ) -> Result<Response<HelloReply>, Status> {
        let parent_cx =
            global::get_text_map_propagator(|prop| prop.extract(&MetadataMap(request.metadata())));
        let tracer = global::tracer("example/server");
        let mut span = tracer
            .span_builder("Greeter/server")
            .with_kind(SpanKind::Server)
            .start_with_context(&tracer, &parent_cx);

        let name = request.into_inner().name;
        expensive_fn(format!("Got name: {name:?}"), &mut span);

        // Return an instance of type HelloReply
        let reply = hello_world::HelloReply {
            message: format!("Hello {name}!"), // We must use .into_inner() as the fields of gRPC requests and responses are private
        };

        Ok(Response::new(reply)) // Send back our formatted greeting
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    init_tracer();

    let addr = "[::1]:50051".parse()?;
    let greeter = MyGreeter::default();

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}
