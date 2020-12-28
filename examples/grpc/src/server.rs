use tonic::{transport::Server, Request, Response, Status};

use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{HelloReply, HelloRequest};
use opentelemetry::global;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use opentelemetry::trace::TraceError;
use opentelemetry::{
    trace::{Span, Tracer},
    KeyValue,
};
use std::error::Error;

pub mod hello_world {
    tonic::include_proto!("helloworld"); // The string specified here must match the proto package name.
}

#[derive(Debug, Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>, // Accept request of type HelloRequest
    ) -> Result<Response<HelloReply>, Status> {
        let parent_cx = global::get_text_map_propagator(|prop| prop.extract(request.metadata()));
        let span = global::tracer("greeter").start_with_context("Processing reply", parent_cx);
        span.set_attribute(KeyValue::new("request", format!("{:?}", request)));

        // Return an instance of type HelloReply
        let reply = hello_world::HelloReply {
            message: format!("Hello {}!", request.into_inner().name), // We must use .into_inner() as the fields of gRPC requests and responses are private
        };

        Ok(Response::new(reply)) // Send back our formatted greeting
    }
}

fn tracing_init() -> Result<(impl Tracer, opentelemetry_jaeger::Uninstall), TraceError> {
    global::set_text_map_propagator(TraceContextPropagator::new());
    opentelemetry_jaeger::new_pipeline()
        .with_service_name("grpc-server")
        .install()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let _uninstall = tracing_init()?;
    let addr = "[::1]:50051".parse()?;
    let greeter = MyGreeter::default();

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
