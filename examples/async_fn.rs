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
    api::{trace::futures::Instrument, Tracer},
    global, sdk,
};
use std::time::Duration;
use std::{error::Error, io, net::SocketAddr, thread};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

async fn connect(addr: &SocketAddr) -> io::Result<TcpStream> {
    let tracer = global::tracer("connector");
    let span = tracer.start("Connecting", None);

    TcpStream::connect(&addr).instrument(span).await
}

async fn write(stream: &mut TcpStream) -> io::Result<usize> {
    let tracer = global::tracer("writer");
    let span = tracer.start("Writing", None);

    stream.write(b"hello world\n").instrument(span).await
}

async fn run(addr: &SocketAddr) -> io::Result<usize> {
    let tracer = global::tracer("runner");
    let span = tracer.start(&format!("running: {}", addr), None);

    let mut stream = connect(addr).instrument(tracer.clone_span(&span)).await?;
    write(&mut stream).instrument(span).await
}

fn init_tracer() -> thrift::Result<()> {
    let exporter = opentelemetry_jaeger::Exporter::builder()
        .with_agent_endpoint("127.0.0.1:6831".parse().unwrap())
        .with_process(opentelemetry_jaeger::Process {
            service_name: "trace-demo".to_string(),
            tags: vec![],
        })
        .init()?;
    let batch = sdk::BatchSpanProcessor::builder(exporter, tokio::spawn, tokio::time::interval)
        .with_scheduled_delay(Duration::from_millis(100))
        .build();
    let provider = sdk::Provider::builder()
        .with_batch_exporter(batch)
        .with_config(sdk::Config {
            default_sampler: Box::new(sdk::Sampler::Always),
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
    let span = tracer.start("root", None);

    let (run1, run2) = futures::future::join(run(&addr), run(&addr2))
        .instrument(span)
        .await;
    run1?;
    run2?;

    thread::sleep(Duration::from_millis(250));

    Ok(())
}
