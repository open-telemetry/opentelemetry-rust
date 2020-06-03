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
use opentelemetry::{
    api::{trace::futures::FutureExt, Context, TraceContextExt, Tracer},
    global, sdk,
};
use std::time::Duration;
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
    let span = tracer.start(&format!("running: {}", addr));
    let cx = Context::current_with_span(span);

    let mut stream = connect(addr).with_context(cx.clone()).await?;
    write(&mut stream).with_context(cx).await
}

fn init_tracer() -> thrift::Result<()> {
    let exporter = opentelemetry_jaeger::Exporter::builder()
        .with_agent_endpoint("127.0.0.1:6831".parse().unwrap())
        .with_process(opentelemetry_jaeger::Process {
            service_name: "trace-demo".to_string(),
            tags: vec![],
        })
        .init()?;

    // Configure your async library of choice. E.g. `async_std::spawn` or similar function can be
    // used here in place of `tokio::spawn`, etc.
    let batch = sdk::BatchSpanProcessor::builder(exporter, tokio::spawn, tokio::time::interval)
        .with_scheduled_delay(Duration::from_millis(100))
        .build();

    // For the demonstration, use `Sampler::AlwaysOn` sampler to sample all traces. In a production
    // application, use `Sampler::ParentOrElse` or `Sampler::Probability` with a desired probability.
    let provider = sdk::Provider::builder()
        .with_batch_exporter(batch)
        .with_config(sdk::Config {
            default_sampler: Box::new(sdk::Sampler::AlwaysOn),
            ..Default::default()
        })
        .build();
    global::set_provider(provider);

    Ok(())
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    init_tracer()?;
    let addr = "127.0.0.1:6142".parse()?;
    let addr2 = "127.0.0.1:6143".parse()?;
    let tracer = global::tracer("async_example");
    let span = tracer.start("root");
    let cx = Context::current_with_span(span);

    let (run1, run2) = futures::future::join(run(&addr), run(&addr2))
        .with_context(cx)
        .await;
    run1?;
    run2?;

    tokio::time::delay_for(Duration::from_millis(250)).await;
    // or async_std::task::sleep(Duration::from_millis(250)).await;

    Ok(())
}
