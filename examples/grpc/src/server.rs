use tonic::{transport::Server, Request, Response, Status};

use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{HelloReply, HelloRequest};
use opentelemetry::api::{self, KeyValue, Span, TextMapFormat, Tracer};
use opentelemetry::global;
use opentelemetry::sdk;
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
        let propagator = api::TraceContextPropagator::new();
        let parent_cx = propagator.extract(request.metadata());
        let span = global::tracer("greeter").start_from_context("Processing reply", &parent_cx);
        span.set_attribute(KeyValue::new("request", format!("{:?}", request)));

        // Return an instance of type HelloReply
        let reply = hello_world::HelloReply {
            message: format!("Hello {}!", request.into_inner().name), // We must use .into_inner() as the fields of gRPC requests and responses are private
        };

        Ok(Response::new(reply)) // Send back our formatted greeting
    }
}

fn tracing_init() -> Result<(sdk::Tracer, opentelemetry_jaeger::Uninstall), Box<dyn Error>> {
    opentelemetry_jaeger::new_pipeline()
        .with_service_name("grpc-server")
        .install()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _uninstall = tracing_init()?;
    let addr = "[::1]:50051".parse()?;
    let greeter = MyGreeter::default();

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
