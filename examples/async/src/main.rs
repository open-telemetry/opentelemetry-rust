//! Demonstrates using OpenTelemetry to instrument `async` functions.
//!
//! This is based on the [`hello_world`] example from `tokio`. and implements a
//! simple client that opens a TCP stream, writes "hello world\n", and closes
//! the connection.
//!
//! You can test this out by running:
//!
//!     ncat -l 6142
//!
//! And then in a second terminal run:
//!
//!     ncat -l 6143
//!
//! And then in a third terminal run:
//!
//!     cargo run --example async_fn
//!
//! [`hello_world`]: https://github.com/tokio-rs/tokio/blob/132e9f1da5965530b63554d7a1c59824c3de4e30/tokio/examples/hello_world.rs
use opentelemetry::trace::TraceError;
use opentelemetry::{
    global,
    sdk::trace as sdktrace,
    trace::{FutureExt, TraceContextExt, Tracer},
    Context,
};
use std::{error::Error, io, net::SocketAddr};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

async fn connect(addr: &SocketAddr) -> io::Result<TcpStream> {
    let tracer = global::tracer("connector");
    let span = tracer.start("Connecting");
    let cx = Context::current_with_value(span);

    TcpStream::connect(&addr).with_context(cx).await
}

async fn write(stream: &mut TcpStream) -> io::Result<usize> {
    let tracer = global::tracer("writer");
    let span = tracer.start("Writing");
    let cx = Context::current_with_span(span);

    stream.write(b"hello world\n").with_context(cx).await
}

async fn run(addr: &SocketAddr) -> io::Result<usize> {
    let tracer = global::tracer("runner");
    let span = tracer.start(format!("running: {}", addr));
    let cx = Context::current_with_span(span);

    let mut stream = connect(addr).with_context(cx.clone()).await?;
    write(&mut stream).with_context(cx).await
}

fn init_tracer() -> Result<sdktrace::Tracer, TraceError> {
    opentelemetry_jaeger::new_pipeline()
        .with_service_name("trace-demo")
        .install_batch(opentelemetry::runtime::Tokio)
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let tracer = init_tracer()?;
    let addr = "127.0.0.1:6142".parse()?;
    let addr2 = "127.0.0.1:6143".parse()?;
    let span = tracer.start("root");
    let cx = Context::current_with_span(span);

    let (run1, run2) = futures::future::join(run(&addr), run(&addr2))
        .with_context(cx)
        .await;
    run1?;
    run2?;

    global::shutdown_tracer_provider();
    Ok(())
}
